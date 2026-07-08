use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc, time::SystemTime};

use agglayer_config::settlement_service::{SettlementServiceConfig, SettlementTransactionConfig};
use agglayer_storage::stores::{
    EditEvenIfCompleted, SettlementReader, SettlementWriter, StateReader, StateWriter,
};
use agglayer_types::{
    Address, CertificateId, ClientError, Nonce, SettlementAttempt, SettlementAttemptResult,
    SettlementJob, SettlementJobId, SettlementJobResult, SettlementTxHash,
};
use alloy::{
    consensus::Transaction as _,
    network::TransactionResponse as _,
    providers::{Provider, WalletProvider},
};
use educe::Educe;
use eyre::Context as _;
use tokio::sync::{watch, Mutex};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::settlement_task::{
    SettlementTask, SettlementTaskRunResult, StoredSettlementJob, TaskAdminCommand,
    TaskControlHandle,
};

/// The Settlement Service is responsible for managing settlement jobs and
/// answering settlement result requests.
///
/// Once startup recovery completes, every persisted settlement job without a
/// terminal result is expected to have a running task and in-memory result
/// watcher. The admin abort escape hatch is the current exception: it can stop
/// a task without recording a terminal result until the admin API grows an
/// explicit aborted result.
#[derive(Educe)]
#[educe(Clone)]
pub struct SettlementService<L1Provider, SettlementStore> {
    tx_config: Arc<SettlementTransactionConfig>,
    provider: Arc<L1Provider>,
    store: Arc<SettlementStore>,
    cancellation_token: CancellationToken,
    task_controls: Arc<Mutex<HashMap<SettlementJobId, TaskControlHandle>>>,
    result_watchers:
        Arc<Mutex<HashMap<SettlementJobId, watch::Receiver<Option<SettlementJobResult>>>>>,
}

pub struct SettlementJobWatcher {
    watcher: watch::Receiver<Option<SettlementJobResult>>,
    job_id: SettlementJobId,
}

impl SettlementJobWatcher {
    pub fn watcher(&mut self) -> &mut watch::Receiver<Option<SettlementJobResult>> {
        &mut self.watcher
    }

    pub fn job_id(&self) -> SettlementJobId {
        self.job_id
    }

    /// Wait until the job produces a result, then return it.
    ///
    /// Uses `wait_for(Option::is_some)` rather than `changed()` so a result
    /// that landed before this call is not missed.
    pub async fn wait_for_result(&mut self) -> eyre::Result<SettlementJobResult> {
        let result = self
            .watcher
            .wait_for(|value| value.is_some())
            .await
            .map_err(|_| eyre::eyre!("settlement job watcher closed before producing a result"))?
            .clone();
        result.ok_or_else(|| eyre::eyre!("settlement job completed with no result"))
    }
}

pub enum RetrievedSettlementResult {
    Pending(SettlementJobWatcher),
    Completed(SettlementJobResult),
}

impl<
        L1Provider: Provider + WalletProvider + 'static,
        SettlementStore: SettlementReader + SettlementWriter + StateReader + StateWriter + Send + Sync + 'static,
    > SettlementService<L1Provider, SettlementStore>
{
    pub async fn start(
        _config: SettlementServiceConfig,
        tx_config: Arc<SettlementTransactionConfig>,
        provider: Arc<L1Provider>,
        store: Arc<SettlementStore>,
        cancellation_token: CancellationToken,
    ) -> eyre::Result<Self> {
        let this = Self {
            tx_config,
            provider,
            store,
            cancellation_token,
            task_controls: Arc::new(Mutex::new(HashMap::new())),
            result_watchers: Arc::new(Mutex::new(HashMap::new())),
        };
        this.resume_pending_settlement_jobs().await?;
        Ok(this)
    }

    #[tracing::instrument(skip_all)]
    async fn resume_pending_settlement_jobs(&self) -> eyre::Result<()> {
        // TODO: Avoid scanning the whole settlement jobs CF on every startup.
        // Record the latest ULID before which all settlement job ids are known
        // to be fully complete in the metadata CF, then start future scans from
        // that point.
        let job_ids = self
            .store
            .list_settlement_job_ids()
            .wrap_err("Failed to scan settlement job ids during startup recovery")?;

        let mut completed_jobs = 0usize;
        let mut resumed_jobs = 0usize;
        for job_id in job_ids {
            let (task_control_handle, task_control) =
                TaskControlHandle::new(&self.cancellation_token);
            match SettlementTask::load(
                job_id,
                self.tx_config.clone(),
                self.provider.clone(),
                self.store.clone(),
                task_control,
            )
            .await
            .wrap_err_with(|| {
                format!("Failed to load settlement job {job_id} during startup recovery")
            })? {
                StoredSettlementJob::Completed(_, _) => {
                    completed_jobs += 1;
                }
                StoredSettlementJob::Pending(task) => {
                    self.spawn_settlement_task(job_id, task, task_control_handle)
                        .await;
                    resumed_jobs += 1;
                }
            }
        }

        info!(
            completed_jobs,
            resumed_jobs, "Settlement service startup recovery scan completed"
        );
        Ok(())
    }

    async fn spawn_settlement_task(
        &self,
        job_id: SettlementJobId,
        mut task: SettlementTask<L1Provider, SettlementStore>,
        task_control_handle: TaskControlHandle,
    ) -> watch::Receiver<Option<SettlementJobResult>> {
        let (result_sender, result_receiver) = watch::channel(None);
        self.task_controls
            .lock()
            .await
            .insert(job_id, task_control_handle);
        self.result_watchers
            .lock()
            .await
            .insert(job_id, result_receiver.clone());
        let task_controls = self.task_controls.clone();
        let result_watchers = self.result_watchers.clone();
        let tx_config = self.tx_config.clone();
        let provider = self.provider.clone();
        let store = self.store.clone();
        let cancellation_token = self.cancellation_token.clone();
        tokio::task::spawn(async move {
            loop {
                match task.run().await {
                    SettlementTaskRunResult::Completed(result) => {
                        if let Err(error) = result_sender.send(Some(result)) {
                            error!(
                                ?error,
                                ?job_id,
                                "Failed to send settlement job result to watchers"
                            );
                        }
                        task_controls.lock().await.remove(&job_id);
                        break;
                    }
                    SettlementTaskRunResult::Cancelled => {
                        info!(?job_id, "Settlement task cancelled");
                        result_watchers.lock().await.remove(&job_id);
                        task_controls.lock().await.remove(&job_id);
                        break;
                    }
                    SettlementTaskRunResult::ReloadAndRestart => {
                        info!(?job_id, "Reloading and restarting settlement task");
                        let (task_control_handle, task_control) =
                            TaskControlHandle::new(&cancellation_token);
                        task_controls
                            .lock()
                            .await
                            .insert(job_id, task_control_handle);
                        match SettlementTask::load(
                            job_id,
                            tx_config.clone(),
                            provider.clone(),
                            store.clone(),
                            task_control,
                        )
                        .await
                        {
                            Ok(StoredSettlementJob::Pending(reloaded_task)) => {
                                task = reloaded_task;
                            }
                            Ok(StoredSettlementJob::Completed(_, result)) => {
                                if let Err(error) = result_sender.send(Some(result)) {
                                    error!(
                                        ?error,
                                        ?job_id,
                                        "Failed to send settlement job result to watchers"
                                    );
                                }
                                task_controls.lock().await.remove(&job_id);
                                break;
                            }
                            Err(error) => {
                                error!(
                                    ?error,
                                    ?job_id,
                                    "Failed to reload settlement task; dropping in-memory task \
                                     state"
                                );
                                result_watchers.lock().await.remove(&job_id);
                                task_controls.lock().await.remove(&job_id);
                                break;
                            }
                        }
                    }
                }
            }
        });
        result_receiver
    }

    #[tracing::instrument(skip_all)]
    async fn task_control(&self, job_id: SettlementJobId) -> eyre::Result<TaskControlHandle> {
        let task_controls = self.task_controls.lock().await;
        let Some(task_control) = task_controls.get(&job_id) else {
            eyre::bail!("No task control found for settlement task {job_id}");
        };
        Ok(task_control.clone())
    }

    #[tracing::instrument(skip_all)]
    async fn admin_task(
        &self,
        job_id: SettlementJobId,
        command: TaskAdminCommand,
    ) -> eyre::Result<()> {
        self.task_control(job_id)
            .await?
            .try_send(command)
            .wrap_err_with(|| {
                format!(
                    "Failed to forward admin command to settlement task {job_id}, did it already \
                     complete?"
                )
            })?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn admin_abort_task(&self, job_id: SettlementJobId) -> eyre::Result<()> {
        self.task_control(job_id).await?.cancel();
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn admin_reload_and_restart_task(&self, job_id: SettlementJobId) -> eyre::Result<()> {
        self.admin_task(job_id, TaskAdminCommand::ReloadAndRestart)
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn request_new_settlement(
        &self,
        certificate_id: Option<CertificateId>,
        job: SettlementJob,
    ) -> eyre::Result<SettlementJobWatcher> {
        let (task_control_handle, task_control) = TaskControlHandle::new(&self.cancellation_token);
        let (job_id, task) = SettlementTask::create(
            certificate_id,
            job,
            self.tx_config.clone(),
            self.provider.clone(),
            self.store.clone(),
            task_control,
        )
        .await?;
        let result_receiver = self
            .spawn_settlement_task(job_id, task, task_control_handle)
            .await;
        Ok(SettlementJobWatcher {
            watcher: result_receiver,
            job_id,
        })
    }

    #[tracing::instrument(skip(self))]
    pub async fn retrieve_settlement_result(
        &self,
        job_id: SettlementJobId,
    ) -> eyre::Result<RetrievedSettlementResult> {
        if let Some(watcher) = self.result_watchers.lock().await.get(&job_id) {
            return match watcher.borrow().as_ref() {
                None => Ok(RetrievedSettlementResult::Pending(SettlementJobWatcher {
                    job_id,
                    watcher: watcher.clone(),
                })),
                Some(result) => Ok(RetrievedSettlementResult::Completed(result.clone())),
            };
        }

        if let Some(result) = self
            .store
            .get_settlement_job_result(&job_id)
            .wrap_err_with(|| {
                format!("Failed to read settlement job terminal result for id {job_id}")
            })?
        {
            return Ok(RetrievedSettlementResult::Completed(result));
        }

        if self
            .store
            .get_settlement_job(&job_id)
            .wrap_err_with(|| format!("Failed to check settlement job existence for id {job_id}"))?
            .is_none()
        {
            eyre::bail!("No settlement job found for id {job_id}");
        }

        error!(
            ?job_id,
            "Settlement service invariant broken: pending job exists without running task"
        );
        eyre::bail!("Pending settlement job {job_id} exists without a running task");
    }
}

/// How the live task for a job (if any) was told about an admin mutation.
///
/// Admin mutations are declarative edits of stored state; a running task only
/// picks them up by reloading from storage. Anything but [`Notified`] means
/// the operator should check the job before relying on the edit being live.
///
/// Serializes as `notified` / `absent` / `notify-failed` in admin RPC
/// responses.
///
/// [`Notified`]: LiveTaskNotification::Notified
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LiveTaskNotification {
    /// The running task was told to reload from storage and restart.
    Notified,
    /// No live task exists for this job. The edit persists and is picked up
    /// whenever a task is started for the job (e.g. on startup recovery).
    Absent,
    /// A live task exists but could not be notified; it keeps acting on its
    /// stale in-memory view until it reloads. Retry with an explicit task
    /// reload, or abort the task.
    NotifyFailed,
}

/// A settlement attempt to register through the admin surface.
///
/// Only the transaction hash is mandatory. A missing sender or nonce is
/// resolved by fetching the transaction from L1 by hash; missing fees fall
/// back to the fetched transaction's fees, or 0 when the transaction was not
/// fetched. A missing submission time defaults to now.
#[derive(Clone, Debug)]
pub struct NewSettlementAttempt {
    pub tx_hash: SettlementTxHash,
    pub sender_wallet: Option<Address>,
    pub nonce: Option<Nonce>,
    pub submission_time: Option<SystemTime>,
    pub max_fee_per_gas: Option<u128>,
    pub max_priority_fee_per_gas: Option<u128>,
}

/// Admin surface: mutations of stored settlement state.
impl<
        L1Provider: Provider + WalletProvider + 'static,
        SettlementStore: SettlementReader + SettlementWriter + StateReader + StateWriter + Send + Sync + 'static,
    > SettlementService<L1Provider, SettlementStore>
{
    /// Tells the live task for `job_id`, if any, to drop its in-memory state
    /// and reload from storage, so it observes an admin edit.
    ///
    /// Best-effort: the edit is already persisted when this runs, and a task
    /// that cannot be notified will still observe it on its next reload.
    async fn notify_live_task_of_admin_edit(
        &self,
        job_id: SettlementJobId,
    ) -> LiveTaskNotification {
        let task_controls = self.task_controls.lock().await;
        let Some(task_control) = task_controls.get(&job_id) else {
            return LiveTaskNotification::Absent;
        };

        match task_control.try_send(TaskAdminCommand::ReloadAndRestart) {
            Ok(()) => LiveTaskNotification::Notified,
            Err(error) => {
                warn!(
                    ?job_id,
                    ?error,
                    "Failed to notify live settlement task of an admin edit; the task acts on \
                     stale in-memory state until it reloads"
                );
                LiveTaskNotification::NotifyFailed
            }
        }
    }

    /// Resolves an admin-provided attempt into a full [`SettlementAttempt`],
    /// fetching the transaction from L1 when the sender or nonce is missing.
    async fn resolve_new_settlement_attempt(
        &self,
        attempt: NewSettlementAttempt,
    ) -> eyre::Result<SettlementAttempt> {
        let tx_hash = attempt.tx_hash;
        let fetched_tx = match (&attempt.sender_wallet, &attempt.nonce) {
            (Some(_), Some(_)) => None,
            _ => Some(
                self.provider
                    .get_transaction_by_hash(tx_hash.into())
                    .await
                    .wrap_err_with(|| {
                        format!("Failed to fetch settlement transaction {tx_hash} from L1")
                    })?
                    .ok_or_else(|| {
                        eyre::eyre!(
                            "Settlement transaction {tx_hash} is not known to the L1 RPC; provide \
                             sender_wallet and nonce explicitly"
                        )
                    })?,
            ),
        };

        Ok(SettlementAttempt {
            sender_wallet: attempt
                .sender_wallet
                .or_else(|| fetched_tx.as_ref().map(|tx| tx.from().into()))
                .expect("transaction is fetched when the sender is missing"),
            nonce: attempt
                .nonce
                .or_else(|| fetched_tx.as_ref().map(|tx| Nonce(tx.nonce())))
                .expect("transaction is fetched when the nonce is missing"),
            hash: tx_hash,
            submission_time: attempt.submission_time.unwrap_or_else(SystemTime::now),
            max_fee_per_gas: attempt
                .max_fee_per_gas
                // Fully qualified: the rpc transaction type also offers
                // `TransactionResponse::max_fee_per_gas`.
                .or_else(|| {
                    fetched_tx
                        .as_ref()
                        .map(alloy::consensus::Transaction::max_fee_per_gas)
                })
                .unwrap_or(0),
            max_priority_fee_per_gas: attempt
                .max_priority_fee_per_gas
                .or_else(|| {
                    fetched_tx
                        .as_ref()
                        .and_then(|tx| tx.max_priority_fee_per_gas())
                })
                .unwrap_or(0),
        })
    }

    /// Appends a new settlement attempt to `job_id` and returns its assigned
    /// attempt number.
    ///
    /// This always adds one new attempt (it never overwrites an existing
    /// one), so it is safe for porting an externally-submitted settlement
    /// transaction into the job.
    #[tracing::instrument(skip(self))]
    pub async fn admin_insert_settlement_attempt(
        &self,
        job_id: SettlementJobId,
        attempt: NewSettlementAttempt,
        edit_even_if_completed: EditEvenIfCompleted,
    ) -> eyre::Result<(u64, LiveTaskNotification)> {
        let attempt = self.resolve_new_settlement_attempt(attempt).await?;
        let attempt_number = self
            .store
            .admin_insert_settlement_attempt(&job_id, &attempt, edit_even_if_completed)
            .wrap_err_with(|| format!("Failed to insert settlement attempt for job {job_id}"))?;
        let live_task = self.notify_live_task_of_admin_edit(job_id).await;
        Ok((attempt_number, live_task))
    }

    /// Records that an administrator asserts the attempt will never land on
    /// L1, overwriting any previously recorded result for it.
    ///
    /// Terminal for the attempt, never for the job: the reloaded task no
    /// longer waits on this attempt and drives the settlement elsewhere.
    #[tracing::instrument(skip(self))]
    pub async fn admin_mark_attempt_definitely_failed(
        &self,
        job_id: SettlementJobId,
        attempt_number: u64,
        reason: &str,
        edit_even_if_completed: EditEvenIfCompleted,
    ) -> eyre::Result<LiveTaskNotification> {
        let result = SettlementAttemptResult::ClientError(ClientError::abandoned_by_admin(reason));
        self.store
            .admin_override_settlement_attempt_result(
                &job_id,
                attempt_number,
                &result,
                edit_even_if_completed,
            )
            .wrap_err_with(|| {
                format!(
                    "Failed to mark settlement attempt {attempt_number} of job {job_id} as \
                     definitely failed"
                )
            })?;
        Ok(self.notify_live_task_of_admin_edit(job_id).await)
    }

    /// Removes the recorded result of an attempt, handing the attempt back to
    /// the settlement task as pending.
    #[tracing::instrument(skip(self))]
    pub async fn admin_remove_attempt_result(
        &self,
        job_id: SettlementJobId,
        attempt_number: u64,
        edit_even_if_completed: EditEvenIfCompleted,
    ) -> eyre::Result<LiveTaskNotification> {
        self.store
            .admin_remove_settlement_attempt_result(&job_id, attempt_number, edit_even_if_completed)
            .wrap_err_with(|| {
                format!(
                    "Failed to remove result of settlement attempt {attempt_number} of job \
                     {job_id}"
                )
            })?;
        Ok(self.notify_live_task_of_admin_edit(job_id).await)
    }

    /// Removes the terminal result of a completed job and spawns a fresh task
    /// for it, so the job is re-driven from its stored state.
    ///
    /// Force operation: if the removed result was real, only the settlement
    /// contract's replay protection stands between the re-driven job and a
    /// double settlement.
    #[tracing::instrument(skip(self))]
    pub async fn admin_force_remove_settlement_job_result(
        &self,
        job_id: SettlementJobId,
    ) -> eyre::Result<()> {
        // A completed job has no live task. Refusing while one is still
        // registered (e.g. mid-completion, or the job is simply not done)
        // keeps the old task and the fresh one below from racing.
        if self.task_controls.lock().await.contains_key(&job_id) {
            eyre::bail!(
                "A settlement task is still live for job {job_id}; its terminal result can only \
                 be removed once it has completed"
            );
        }

        self.store
            .admin_force_remove_settlement_job_result(&job_id)
            .wrap_err_with(|| {
                format!("Failed to remove terminal result of settlement job {job_id}")
            })?;

        // Drop the in-memory watcher that still broadcasts the removed
        // result, then bring the job back to life.
        self.result_watchers.lock().await.remove(&job_id);

        let (task_control_handle, task_control) = TaskControlHandle::new(&self.cancellation_token);
        match SettlementTask::load(
            job_id,
            self.tx_config.clone(),
            self.provider.clone(),
            self.store.clone(),
            task_control,
        )
        .await
        .wrap_err_with(|| {
            format!(
                "Removed the terminal result of settlement job {job_id} but failed to reload the \
                 job; it will be re-driven after the next node restart"
            )
        })? {
            StoredSettlementJob::Pending(task) => {
                self.spawn_settlement_task(job_id, task, task_control_handle)
                    .await;
                Ok(())
            }
            StoredSettlementJob::Completed(_, _) => {
                eyre::bail!(
                    "Settlement job {job_id} still has a terminal result right after its removal; \
                     was one re-recorded concurrently?"
                )
            }
        }
    }
}

#[derive(Debug)]
pub struct RequestNewSettlement {
    pub certificate_id: Option<CertificateId>,
    pub job: SettlementJob,
}

impl<
        L1Provider: Provider + WalletProvider + 'static,
        SettlementStore: SettlementReader + SettlementWriter + StateReader + StateWriter + Send + Sync + 'static,
    > tower::Service<RequestNewSettlement> for SettlementService<L1Provider, SettlementStore>
{
    type Response = SettlementJobWatcher;
    type Error = eyre::Error;
    type Future = Pin<Box<dyn Future<Output = eyre::Result<Self::Response>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: RequestNewSettlement) -> Self::Future {
        let this = self.clone();
        Box::pin(async move {
            this.request_new_settlement(req.certificate_id, req.job)
                .await
        })
    }
}

pub struct RetrieveSettlementResult(pub SettlementJobId);

impl<
        L1Provider: Provider + WalletProvider + 'static,
        SettlementStore: SettlementReader + SettlementWriter + StateReader + StateWriter + Send + Sync + 'static,
    > tower::Service<RetrieveSettlementResult> for SettlementService<L1Provider, SettlementStore>
{
    type Response = RetrievedSettlementResult;
    type Error = eyre::Error;
    type Future = Pin<Box<dyn Future<Output = eyre::Result<Self::Response>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: RetrieveSettlementResult) -> Self::Future {
        let this = self.clone();
        Box::pin(async move { this.retrieve_settlement_result(req.0).await })
    }
}

pub enum AdminCommand {
    AbortTask(SettlementJobId),
    ReloadAndRestartTask(SettlementJobId),
}

impl<
        L1Provider: Provider + WalletProvider + 'static,
        SettlementStore: SettlementReader + SettlementWriter + StateReader + StateWriter + Send + Sync + 'static,
    > tower::Service<AdminCommand> for SettlementService<L1Provider, SettlementStore>
{
    type Response = ();
    type Error = eyre::Error;
    type Future = Pin<Box<dyn Future<Output = eyre::Result<Self::Response>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: AdminCommand) -> Self::Future {
        let this = self.clone();
        Box::pin(async move {
            match req {
                AdminCommand::AbortTask(job_id) => this.admin_abort_task(job_id).await,
                AdminCommand::ReloadAndRestartTask(job_id) => {
                    this.admin_reload_and_restart_task(job_id).await
                }
            }
        })
    }
}

#[cfg(test)]
mod tests;
