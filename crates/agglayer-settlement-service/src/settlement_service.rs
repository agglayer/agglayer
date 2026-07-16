use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use agglayer_config::settlement_service::{SettlementServiceConfig, SettlementTransactionConfig};
use agglayer_storage::stores::{SettlementReader, SettlementWriter, StateReader, StateWriter};
use agglayer_types::{CertificateId, SettlementJob, SettlementJobId, SettlementJobResult};
use alloy::providers::{Provider, WalletProvider};
use educe::Educe;
use eyre::Context as _;
use tokio::sync::{watch, Mutex};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::{
    error::SettlementAdminError,
    settlement_task::{
        SettlementTask, SettlementTaskRunResult, StoredSettlementJob, TaskAdminCommand,
        TaskControlHandle,
    },
    wallet_nonce_locks::WalletNonceLocks,
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
    /// Per-job result channel senders. Holding the sender (rather than a
    /// receiver) keeps the channel alive across a task respawn: an admin
    /// reload replaces the task but reuses the registered sender, so
    /// certificates already awaiting through a `subscribe()`d receiver keep
    /// receiving instead of seeing the channel close.
    /// XREF: bot r3589571032 on PR #1681.
    result_senders:
        Arc<Mutex<HashMap<SettlementJobId, watch::Sender<Option<SettlementJobResult>>>>>,
    /// Per-wallet locks serializing the nonce read-to-save window across
    /// concurrent settlement tasks.
    /// XREF: https://github.com/agglayer/agglayer/issues/1597
    wallet_nonce_locks: Arc<WalletNonceLocks>,
    /// Serializes admin respawn operations so two concurrent
    /// reload-and-restart calls cannot spawn two tasks for one job.
    admin_operation_lock: Arc<Mutex<()>>,
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
            result_senders: Arc::new(Mutex::new(HashMap::new())),
            wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
            admin_operation_lock: Arc::new(Mutex::new(())),
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
                self.wallet_nonce_locks.clone(),
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
        self.task_controls
            .lock()
            .await
            .insert(job_id, task_control_handle);
        // Reuse the registered sender on respawn so waiters that subscribed
        // before the respawn keep receiving; only create a fresh channel for
        // a job with no sender yet.
        let result_sender = {
            let mut result_senders = self.result_senders.lock().await;
            result_senders
                .entry(job_id)
                .or_insert_with(|| watch::channel(None).0)
                .clone()
        };
        let result_receiver = result_sender.subscribe();
        let task_controls = self.task_controls.clone();
        let result_senders = self.result_senders.clone();
        let tx_config = self.tx_config.clone();
        let provider = self.provider.clone();
        let store = self.store.clone();
        let wallet_nonce_locks = self.wallet_nonce_locks.clone();
        tokio::task::spawn(async move {
            loop {
                match task.run().await {
                    SettlementTaskRunResult::Completed(result) => {
                        // Publish the result, then drop the registered sender.
                        // `send_replace` sets the value unconditionally (even
                        // with zero receivers, as for a job resumed at startup
                        // whose spawn receiver was discarded), unlike `send`
                        // which errors and leaves the value `None`. A current
                        // or late-but-pre-removal subscriber observes the value
                        // set here; a subscriber after removal finds no sender
                        // and falls through to the completed result in storage.
                        // XREF: bot r3589964568 on PR #1681.
                        let _ = result_sender.send_replace(Some(result));
                        result_senders.lock().await.remove(&job_id);
                        task_controls.lock().await.remove(&job_id);
                        break;
                    }
                    SettlementTaskRunResult::Cancelled => {
                        info!(?job_id, "Settlement task cancelled");
                        result_senders.lock().await.remove(&job_id);
                        task_controls.lock().await.remove(&job_id);
                        break;
                    }
                    SettlementTaskRunResult::ReloadAndRestart => {
                        info!(?job_id, "Reloading and restarting settlement task");
                        // Parent the replacement token on the OLD task's token
                        // rather than the service token. During the load the
                        // OLD handle stays registered on purpose, so an
                        // `admin_abort_task` in that window cancels the OLD
                        // token; a child of it is then immediately cancelled,
                        // making the reloaded task observe cancellation at its
                        // first control check and exit cleanly. Parenting on
                        // the service token would lose that abort.
                        // XREF: bot r3589631493 on PR #1681.
                        let (task_control_handle, task_control) =
                            TaskControlHandle::new(task.cancellation_token());
                        // The old handle stays in the map until the reloaded
                        // task is ready: its receiver lives in the current
                        // `task` binding, so it reads as open and a concurrent
                        // admin reload queues a command instead of mistaking
                        // this task for a panicked one and spawning a
                        // duplicate. On a failed load the old handle is
                        // removed below, never leaving a closed handle
                        // registered by a live loop.
                        match SettlementTask::load(
                            job_id,
                            tx_config.clone(),
                            provider.clone(),
                            store.clone(),
                            wallet_nonce_locks.clone(),
                            task_control,
                        )
                        .await
                        {
                            Ok(StoredSettlementJob::Pending(reloaded_task)) => {
                                // If an abort landed on the OLD token while the
                                // task was hydrating, prefer exiting over
                                // installing the reloaded task. The child-token
                                // parenting above already guarantees the
                                // reloaded task would exit at its first control
                                // check, but skipping the swap tears the task
                                // down here without another loop iteration.
                                if task.cancellation_token().is_cancelled() {
                                    info!(
                                        ?job_id,
                                        "Abort observed during in-task reload; exiting instead of \
                                         installing the reloaded task"
                                    );
                                    result_senders.lock().await.remove(&job_id);
                                    task_controls.lock().await.remove(&job_id);
                                    break;
                                }
                                task_controls
                                    .lock()
                                    .await
                                    .insert(job_id, task_control_handle);
                                task = reloaded_task;
                            }
                            Ok(StoredSettlementJob::Completed(_, result)) => {
                                // `send_replace` sets the value unconditionally
                                // (even with zero receivers), unlike `send`
                                // which errors and leaves the value `None`. A
                                // current or late-but-pre-removal subscriber
                                // observes the value; a subscriber after removal
                                // falls through to the completed result in
                                // storage.
                                // XREF: bot r3589964568 on PR #1681.
                                let _ = result_sender.send_replace(Some(result));
                                result_senders.lock().await.remove(&job_id);
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
                                result_senders.lock().await.remove(&job_id);
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

    /// Classify why no live task exists for `job_id` by consulting
    /// storage: completed job, pending job with a dead task, or no such
    /// job at all.
    async fn no_live_task_error(&self, job_id: SettlementJobId) -> SettlementAdminError {
        match self.store.get_settlement_job_result(&job_id) {
            Err(source) => SettlementAdminError::Storage { job_id, source },
            Ok(Some(_)) => SettlementAdminError::JobCompleted(job_id),
            Ok(None) => match self.store.get_settlement_job(&job_id) {
                Err(source) => SettlementAdminError::Storage { job_id, source },
                Ok(Some(_)) => SettlementAdminError::NoLiveTask(job_id),
                Ok(None) => SettlementAdminError::JobNotFound(job_id),
            },
        }
    }

    /// Request cancellation of the in-memory task of `job_id`; the job stays
    /// pending in storage and no terminal result is recorded.
    ///
    /// Serializes against reload/respawn via `admin_operation_lock`: an abort
    /// either runs fully before an [`Self::admin_reload_and_restart_task`]
    /// starts or fully after it completes. This closes the respawn's
    /// load+spawn window, where the map briefly holds the stale closed handle
    /// of a panicked task: without the lock an abort in that window would
    /// cancel the dead token (a no-op) and then the respawn would install a
    /// fresh, uncancelled handle, silently losing the abort. Under the lock a
    /// post-respawn abort instead finds the freshly spawned live handle and
    /// cancels the real token. Taking the lock cannot expose abort to an
    /// unbounded block because gas estimation — the only previously-unbounded
    /// await — now runs outside the lock (XREF: Codex bot r3590141585 on PR
    /// #1681); what remains under the lock is bounded storage reads, map
    /// inserts, and a non-awaiting spawn.
    ///
    /// Returns before the task observes the cancellation; watch the actual
    /// teardown through [`Self::has_live_task`]. `Ok(())` can race with task
    /// completion, where the cancel is a no-op on a lingering handle. An
    /// abort chained with a reload-and-restart can drop the queued reload if
    /// the task exits on the cancellation first; retry once
    /// [`Self::has_live_task`] is `false`. Aborting the stale registration of
    /// a panicked task is an accepted no-op; the next
    /// [`Self::admin_reload_and_restart_task`] clears it. The storage
    /// classification behind the returned errors is a best-effort snapshot.
    #[tracing::instrument(skip(self))]
    pub async fn admin_abort_task(
        &self,
        job_id: SettlementJobId,
    ) -> Result<(), SettlementAdminError> {
        // Serialize against reload/respawn so an abort cannot interleave with
        // a respawn's load+spawn window and get lost on the stale closed
        // handle. Same lock-ordering as `admin_reload_and_restart_task`
        // (admin_operation_lock, then task_controls), so no new cycle.
        // XREF: Codex bot r3594135624 on PR #1681.
        let _admin_op = self.admin_operation_lock.lock().await;

        let control = self.task_controls.lock().await.get(&job_id).cloned();
        match control {
            Some(control) => {
                control.cancel();
                Ok(())
            }
            None => Err(self.no_live_task_error(job_id).await),
        }
    }

    /// Whether an in-memory task is currently registered for `job_id`.
    ///
    /// Advisory: the answer can race with task completion. A `pending`
    /// job without a live task is wedged and needs
    /// [`Self::admin_reload_and_restart_task`].
    pub async fn has_live_task(&self, job_id: SettlementJobId) -> bool {
        self.task_controls.lock().await.contains_key(&job_id)
    }

    /// Make the task of `job_id` drop its in-memory state and reload
    /// from storage. A live task gets the reload command; a pending job
    /// without a live task (after an admin abort, a failed in-task
    /// reload, or a task panic) gets a fresh task spawned from storage:
    /// this is the recovery step after [`Self::admin_abort_task`].
    ///
    /// A panicked task exits without deregistering, leaving a closed
    /// control handle in the map; the reload detects the closed handle
    /// and respawns over the stale entry.
    ///
    /// The reload command is queued, not immediate: a task that is
    /// concurrently cancelled can exit without draining it, a reload
    /// already in flight drops commands queued to the pre-reload
    /// handle when it completes, and the channel can close between
    /// the liveness check and the send, failing with
    /// `TaskNotResponding`. Retrying is safe: a retry reaches the
    /// live task, respawns over a dead or panicked one, or reports
    /// the job's storage state.
    #[tracing::instrument(skip(self))]
    pub async fn admin_reload_and_restart_task(
        &self,
        job_id: SettlementJobId,
    ) -> Result<(), SettlementAdminError> {
        let _admin_op = self.admin_operation_lock.lock().await;

        // Fast path: a live task processes the reload itself. A closed
        // handle still registered in the map means the owning task
        // panicked without deregistering: fall through and respawn over
        // the stale entries.
        let control = self.task_controls.lock().await.get(&job_id).cloned();
        if let Some(control) = control {
            if !control.is_closed() {
                return control
                    .try_send(TaskAdminCommand::ReloadAndRestart)
                    .map_err(|error| SettlementAdminError::TaskNotResponding {
                        job_id,
                        reason: error.to_string(),
                    });
            }
            warn!(
                ?job_id,
                "Settlement task control channel is closed but still registered; respawning over \
                 the stale entry"
            );
        }

        // No live task: respawn from storage if the job is still pending.
        let (task_control_handle, task_control) = TaskControlHandle::new(&self.cancellation_token);
        match SettlementTask::load(
            job_id,
            self.tx_config.clone(),
            self.provider.clone(),
            self.store.clone(),
            self.wallet_nonce_locks.clone(),
            task_control,
        )
        .await
        {
            Ok(StoredSettlementJob::Pending(task)) => {
                self.spawn_settlement_task(job_id, task, task_control_handle)
                    .await;
                info!(
                    ?job_id,
                    "Respawned settlement task via admin reload-and-restart"
                );
                Ok(())
            }
            Ok(StoredSettlementJob::Completed(_, result)) => {
                // A stale control/sender can survive here only when the
                // owning task panicked after persisting the terminal
                // result but before publishing it to the watcher. That
                // stale sender can still have a waiter (a certificate in
                // `wait_for_settlement`). Publish the loaded terminal
                // result to that waiter with `send_replace` (which sets
                // the value even with zero receivers) before dropping the
                // sender, so the waiter resolves instead of erroring; a
                // later retrieve finds no sender and reads the completed
                // result from storage. `send_replace` is sync, so a single
                // lock scope covering send_replace + remove is fine and
                // avoids a re-lock.
                // XREF: bot r3589964560 on PR #1681.
                self.task_controls.lock().await.remove(&job_id);
                {
                    let mut result_senders = self.result_senders.lock().await;
                    if let Some(sender) = result_senders.get(&job_id) {
                        sender.send_replace(Some(result));
                    }
                    result_senders.remove(&job_id);
                }
                Err(SettlementAdminError::JobCompleted(job_id))
            }
            Err(error) => match self.store.get_settlement_job(&job_id) {
                Ok(None) => Err(SettlementAdminError::JobNotFound(job_id)),
                _ => Err(SettlementAdminError::ReloadFailed {
                    job_id,
                    reason: format!("{error:#}"),
                }),
            },
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn request_new_settlement(
        &self,
        certificate_id: Option<CertificateId>,
        job: SettlementJob,
    ) -> eyre::Result<SettlementJobWatcher> {
        let (task_control_handle, task_control) = TaskControlHandle::new(&self.cancellation_token);

        // Resolve the gas limit BEFORE acquiring `admin_operation_lock`. Gas
        // estimation is an L1 `eth_estimateGas` with retry-on-transient-failure
        // that writes nothing and touches none of the shared maps; during an
        // L1/RPC outage it can retry for a long time. Holding the admin lock
        // across it would let a new submission stuck in estimation retries
        // block an operator from respawning an unrelated wedged job via
        // `admin_reload_and_restart_task` (which takes the same lock) — the
        // worst time for it. Cancellation is preserved via the task's token.
        // XREF: Codex bot r3590141585 on PR #1681.
        let job = SettlementTask::<L1Provider, SettlementStore>::resolve_gas_limit(
            self.provider.as_ref(),
            self.tx_config.as_ref(),
            job,
            task_control.cancellation_token(),
        )
        .await?;

        // Serialize creation against admin respawn. `create` makes the job
        // visible in storage as pending (reserve/save) before
        // `spawn_settlement_task` registers the task; without this lock a
        // concurrent `admin_reload_and_restart_task` (which takes the same
        // lock) could observe the pending job with no live task in that window
        // and spawn a second task from storage, racing two tasks over one job.
        // Only the reserve/save/spawn window needs the lock — gas estimation
        // above is intentionally outside it (see the comment above).
        // XREF: bot r3589631500 on PR #1681.
        //
        // No deadlock: `request_new_settlement` calls no admin method that
        // re-takes this lock, and `spawn_settlement_task` never takes it.
        let _admin_op = self.admin_operation_lock.lock().await;

        let (job_id, task) = SettlementTask::create(
            certificate_id,
            job,
            self.tx_config.clone(),
            self.provider.clone(),
            self.store.clone(),
            self.wallet_nonce_locks.clone(),
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
        if let Some(sender) = self.result_senders.lock().await.get(&job_id) {
            return match sender.borrow().as_ref() {
                None => Ok(RetrievedSettlementResult::Pending(SettlementJobWatcher {
                    job_id,
                    watcher: sender.subscribe(),
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
                AdminCommand::AbortTask(job_id) => this
                    .admin_abort_task(job_id)
                    .await
                    .map_err(eyre::Report::new),
                AdminCommand::ReloadAndRestartTask(job_id) => this
                    .admin_reload_and_restart_task(job_id)
                    .await
                    .map_err(eyre::Report::new),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use agglayer_storage::tests::mocks::MockStateStore;
    use agglayer_types::{
        CertificateId, ContractCallOutcome, ContractCallResult, Digest, Nonce,
        SettlementAttemptNumber, SettlementJob, SettlementJobId, SettlementJobResult,
        SettlementTxHash, B256, U256,
    };
    use alloy::{
        network::EthereumWallet,
        primitives::U64,
        providers::{mock::Asserter, ProviderBuilder},
        signers::local::PrivateKeySigner,
    };

    use super::*;
    use crate::settlement_task::{
        SettlementTask, StoredSettlementJob, TaskAdminCommand, TaskControlHandle,
    };

    fn mk_provider() -> impl Provider + WalletProvider + 'static {
        ProviderBuilder::new()
            .wallet(EthereumWallet::from(
                PrivateKeySigner::from_slice(&[0x11; 32]).expect("valid test signing key"),
            ))
            .connect_http(
                "http://127.0.0.1:0"
                    .parse()
                    .expect("test provider URL should parse"),
            )
    }

    fn mk_provider_with_gas_estimate(
        gas_estimate: u64,
    ) -> impl Provider + WalletProvider + 'static {
        mk_provider_and_asserter_with_gas_estimate(gas_estimate).0
    }

    /// Like [`mk_provider_with_gas_estimate`] but also returns the backing
    /// [`Asserter`]. The asserter's response queue is drained as the provider
    /// answers RPC calls, so a test can observe whether `eth_estimateGas` has
    /// run yet by inspecting `asserter.read_q().len()`.
    fn mk_provider_and_asserter_with_gas_estimate(
        gas_estimate: u64,
    ) -> (impl Provider + WalletProvider + 'static, Asserter) {
        let asserter = Asserter::new();
        asserter.push_success(&U64::from(gas_estimate));
        let provider = ProviderBuilder::new()
            .wallet(EthereumWallet::from(
                PrivateKeySigner::from_slice(&[0x11; 32]).expect("valid test signing key"),
            ))
            .connect_mocked_client(asserter.clone());
        (provider, asserter)
    }

    fn expect_empty_startup_recovery(store: &mut MockStateStore) {
        store
            .expect_list_settlement_job_ids()
            .once()
            .return_once(|| Ok(Vec::new()));
    }

    async fn mk_service(
        store: Arc<MockStateStore>,
    ) -> SettlementService<impl Provider + WalletProvider + 'static, MockStateStore> {
        mk_service_with_token(store, CancellationToken::new()).await
    }

    async fn mk_service_with_token(
        store: Arc<MockStateStore>,
        cancellation_token: CancellationToken,
    ) -> SettlementService<impl Provider + WalletProvider + 'static, MockStateStore> {
        SettlementService::start(
            SettlementServiceConfig::default(),
            Arc::new(SettlementTransactionConfig::default()),
            Arc::new(mk_provider()),
            store,
            cancellation_token,
        )
        .await
        .expect("settlement service should start")
    }

    fn mk_job_id(seed: u128) -> SettlementJobId {
        SettlementJobId::from(ulid::Ulid::from(seed))
    }

    fn mk_job(seed: u8) -> SettlementJob {
        SettlementJob {
            contract_address: agglayer_types::Address::from([seed; 20]),
            calldata: vec![seed, seed.wrapping_add(1)].into(),
            eth_value: U256::from(seed),
            gas_limit: seed as u128 + 100_000,
        }
    }

    fn mk_result(seed: u8, outcome: ContractCallOutcome) -> SettlementJobResult {
        SettlementJobResult {
            wallet: agglayer_types::Address::from([seed.wrapping_add(3); 20]),
            nonce: Nonce(seed as u64 + 200),
            attempt_number: SettlementAttemptNumber(seed as u64 + 300),
            contract_call_result: ContractCallResult {
                outcome,
                metadata: vec![seed, seed.wrapping_add(1)].into(),
                block_hash: B256::from([seed; 32]),
                block_number: seed as u64,
                tx_hash: SettlementTxHash::new(Digest::from([seed.wrapping_add(2); 32])),
            },
        }
    }

    #[tokio::test]
    async fn start_scans_jobs_and_skips_completed_ones() {
        let mut store = MockStateStore::new();
        let job_id = mk_job_id(9);
        let job = mk_job(9);
        let result = mk_result(9, ContractCallOutcome::Success);

        store
            .expect_list_settlement_job_ids()
            .once()
            .return_once(move || Ok(vec![job_id]));
        store
            .expect_get_settlement_job()
            .once()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .return_once(move |_| Ok(Some(job)));
        store
            .expect_get_settlement_job_result()
            .once()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .return_once(move |_| Ok(Some(result)));
        store.expect_list_settlement_attempts().never();
        store.expect_list_settlement_attempt_results().never();

        let service = mk_service(Arc::new(store)).await;

        assert!(service.task_controls.lock().await.is_empty());
        assert!(service.result_senders.lock().await.is_empty());
    }

    #[tokio::test]
    async fn retrieve_uses_in_memory_watcher_before_storage() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let service = mk_service(Arc::new(store)).await;
        let job_id = mk_job_id(1);
        let in_memory_result = mk_result(2, ContractCallOutcome::Revert);

        let (sender, _watcher) = watch::channel(Some(in_memory_result.clone()));
        service.result_senders.lock().await.insert(job_id, sender);

        let retrieved = service
            .retrieve_settlement_result(job_id)
            .await
            .expect("retrieval should succeed");

        match retrieved {
            RetrievedSettlementResult::Completed(result) => assert_eq!(result, in_memory_result),
            RetrievedSettlementResult::Pending(_) => panic!("expected completed result"),
        }
    }

    #[tokio::test]
    async fn retrieve_uses_stored_terminal_result_without_watcher() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(2);
        let stored_result = mk_result(3, ContractCallOutcome::Success);
        let stored_result_for_store = stored_result.clone();

        store
            .expect_get_settlement_job_result()
            .once()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .return_once(move |_| Ok(Some(stored_result_for_store)));

        let service = mk_service(Arc::new(store)).await;

        let retrieved = service
            .retrieve_settlement_result(job_id)
            .await
            .expect("retrieval should succeed");

        match retrieved {
            RetrievedSettlementResult::Completed(result) => assert_eq!(result, stored_result),
            RetrievedSettlementResult::Pending(_) => panic!("expected completed result"),
        }
    }

    #[tokio::test]
    async fn retrieve_fails_for_unknown_job_id() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(4);

        store
            .expect_get_settlement_job_result()
            .once()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .return_once(|_| Ok(None));
        store
            .expect_get_settlement_job()
            .once()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .return_once(|_| Ok(None));

        let service = mk_service(Arc::new(store)).await;

        let result = service.retrieve_settlement_result(job_id).await;
        assert!(result.is_err(), "unknown job should fail");
        let error = result.err().expect("result should be an error");

        assert!(error.to_string().contains("No settlement job found for id"));
    }

    #[tokio::test]
    async fn retrieve_fails_when_pending_job_has_no_running_task() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(5);
        let job = mk_job(5);

        store
            .expect_get_settlement_job_result()
            .once()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .return_once(|_| Ok(None));
        store
            .expect_get_settlement_job()
            .once()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .return_once(move |_| Ok(Some(job)));

        let service = mk_service(Arc::new(store)).await;

        let result = service.retrieve_settlement_result(job_id).await;
        assert!(
            result.is_err(),
            "pending job without a watcher should fail as an invariant break"
        );
        let error = result.err().expect("result should be an error");

        assert!(error.to_string().contains("exists without a running task"));
    }

    #[tokio::test]
    async fn reload_and_restart_preserves_watcher_when_reload_finds_completed_job() {
        let mut store = MockStateStore::new();
        let job_id = mk_job_id(6);
        let job = mk_job(6);
        let completed_result = mk_result(6, ContractCallOutcome::Success);
        let completed_result_for_store = completed_result.clone();
        let result_reads = Arc::new(Mutex::new(0usize));

        store
            .expect_list_settlement_job_ids()
            .once()
            .return_once(|| Ok(Vec::new()));
        store
            .expect_get_settlement_job()
            .times(2)
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .returning({
                let job = job.clone();
                move |_| Ok(Some(job.clone()))
            });
        store
            .expect_get_settlement_job_result()
            .times(2)
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .returning(move |_| {
                let mut result_reads = result_reads.lock().unwrap();
                *result_reads += 1;
                if *result_reads == 1 {
                    Ok(None)
                } else {
                    Ok(Some(completed_result_for_store.clone()))
                }
            });
        store
            .expect_list_settlement_attempt_results()
            .once()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .return_once(|_| Ok(Vec::new()));
        store
            .expect_list_settlement_attempts()
            .once()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .return_once(|_| Ok(Vec::new()));

        let store = Arc::new(store);
        let service = mk_service(store).await;
        let (task_control_handle, task_control) =
            TaskControlHandle::new(&service.cancellation_token);
        task_control_handle
            .try_send(TaskAdminCommand::ReloadAndRestart)
            .expect("reload command should fit in admin channel");
        let task = match SettlementTask::load(
            job_id,
            service.tx_config.clone(),
            service.provider.clone(),
            service.store.clone(),
            service.wallet_nonce_locks.clone(),
            task_control,
        )
        .await
        .expect("settlement task should load")
        {
            StoredSettlementJob::Pending(task) => task,
            StoredSettlementJob::Completed(_, _) => panic!("initial load should be pending"),
        };

        let mut result_receiver = service
            .spawn_settlement_task(job_id, task, task_control_handle)
            .await;

        result_receiver
            .changed()
            .await
            .expect("reload should publish the stored terminal result");

        assert_eq!(result_receiver.borrow().as_ref(), Some(&completed_result));
        assert!(service.task_controls.lock().await.is_empty());
        // In the sender model the completed result is delivered to the
        // already-subscribed receiver above, then the registered sender is
        // dropped. A later `retrieve_settlement_result` falls through to the
        // completed result in storage rather than a lingering in-memory entry.
        assert!(!service.result_senders.lock().await.contains_key(&job_id));
    }

    /// Regression for bot r3589964568 (PR #1681): the run loop must publish
    /// the terminal result even when the job has zero live receivers, as
    /// happens for a job resumed at startup whose `spawn_settlement_task`
    /// receiver was discarded by `resume_pending_settlement_jobs`.
    ///
    /// The run loop publishes with `send_replace`, which sets the value
    /// unconditionally, rather than `send`, which errors and leaves the value
    /// `None` when there are no receivers. Here the only handle kept alive is a
    /// `Sender` clone (which is NOT a receiver, so it does not mask the bug):
    /// with the buggy `send` the published value would stay `None`, and a
    /// retrieve subscribing in the window before the sender is removed would
    /// observe a pending watcher that then closes with no value even though
    /// storage holds the result. `send_replace` makes the value observable via
    /// the retained sender before removal.
    #[tokio::test]
    async fn completion_publish_sets_value_with_zero_receivers() {
        let mut store = MockStateStore::new();
        let job_id = mk_job_id(37);
        let job = mk_job(37);
        let completed_result = mk_result(37, ContractCallOutcome::Success);
        let completed_result_for_store = completed_result.clone();
        let result_reads = Arc::new(Mutex::new(0usize));

        store
            .expect_list_settlement_job_ids()
            .once()
            .return_once(|| Ok(Vec::new()));
        store
            .expect_get_settlement_job()
            .times(2)
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .returning({
                let job = job.clone();
                move |_| Ok(Some(job.clone()))
            });
        store
            .expect_get_settlement_job_result()
            .times(2)
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .returning(move |_| {
                let mut result_reads = result_reads.lock().unwrap();
                *result_reads += 1;
                if *result_reads == 1 {
                    Ok(None)
                } else {
                    Ok(Some(completed_result_for_store.clone()))
                }
            });
        store
            .expect_list_settlement_attempt_results()
            .once()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .return_once(|_| Ok(Vec::new()));
        store
            .expect_list_settlement_attempts()
            .once()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .return_once(|_| Ok(Vec::new()));

        let store = Arc::new(store);
        let service = mk_service(store).await;

        // Retain a `Sender` clone so we can observe the published value while
        // keeping zero live receivers: this models a startup-resumed job whose
        // spawn receiver was discarded. Register it before spawn so the run
        // loop reuses it and publishes into it on completion.
        let retained_sender = watch::channel(None).0;
        service
            .result_senders
            .lock()
            .await
            .insert(job_id, retained_sender.clone());

        let (task_control_handle, task_control) =
            TaskControlHandle::new(&service.cancellation_token);
        task_control_handle
            .try_send(TaskAdminCommand::ReloadAndRestart)
            .expect("reload command should fit in admin channel");
        let task = match SettlementTask::load(
            job_id,
            service.tx_config.clone(),
            service.provider.clone(),
            service.store.clone(),
            service.wallet_nonce_locks.clone(),
            task_control,
        )
        .await
        .expect("settlement task should load")
        {
            StoredSettlementJob::Pending(task) => task,
            StoredSettlementJob::Completed(_, _) => panic!("initial load should be pending"),
        };

        // Drop the receiver `spawn_settlement_task` hands back, leaving the
        // retained `Sender` clone as the only live handle (zero receivers).
        let spawn_receiver = service
            .spawn_settlement_task(job_id, task, task_control_handle)
            .await;
        drop(spawn_receiver);

        // Wait for the run loop to publish and deregister the sender. Once the
        // sender is removed from the map, publishing has happened.
        tokio::time::timeout(std::time::Duration::from_secs(10), async {
            while service.result_senders.lock().await.contains_key(&job_id) {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("run loop should publish and deregister the sender");

        // The retained sender observed the value despite having zero receivers
        // at publish time. With the buggy `send` this would still be `None`.
        assert_eq!(retained_sender.borrow().as_ref(), Some(&completed_result));
        // A fresh subscriber (as `retrieve_settlement_result` would create)
        // sees the value too, so it reports `Completed` instead of a pending
        // watcher that closes empty.
        assert_eq!(
            retained_sender.subscribe().borrow().as_ref(),
            Some(&completed_result)
        );
    }

    #[tokio::test]
    async fn request_new_settlement_records_certificate_link_before_job() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let certificate_id = CertificateId::new(Digest::from([7; 32]));
        let job = mk_job(7);
        // `create` resolves the gas limit via estimateGas (mock returns 200_000).
        let mut expected_job = job.clone();
        expected_job.gas_limit = 200_000;
        let recorded_job_id = Arc::new(Mutex::new(None));
        let ordering = Arc::new(Mutex::new(Vec::new()));

        store
            .expect_insert_certificate_settlement_job_id()
            .once()
            .withf(move |recorded_certificate_id, _| recorded_certificate_id == &certificate_id)
            .return_once({
                let ordering = ordering.clone();
                let recorded_job_id = recorded_job_id.clone();
                move |_, settlement_job_id| {
                    ordering.lock().unwrap().push("write_link");
                    *recorded_job_id.lock().unwrap() = Some(*settlement_job_id);
                    Ok(())
                }
            });

        store
            .expect_insert_settlement_job()
            .once()
            .withf(move |_, recorded_job| recorded_job == &expected_job)
            .return_once({
                let ordering = ordering.clone();
                let recorded_job_id = recorded_job_id.clone();
                move |settlement_job_id, _| {
                    ordering.lock().unwrap().push("write_job");
                    assert_eq!(*recorded_job_id.lock().unwrap(), Some(*settlement_job_id));
                    Ok(())
                }
            });

        // `create` runs `estimateGas` before persisting; answer it above the
        // ceiling so the stored limit is unchanged. Live token for estimation,
        // then cancel to stop the spawned task.
        let cancellation_token = CancellationToken::new();
        let service = SettlementService::start(
            SettlementServiceConfig::default(),
            Arc::new(SettlementTransactionConfig::default()),
            Arc::new(mk_provider_with_gas_estimate(200_000)),
            Arc::new(store),
            cancellation_token.clone(),
        )
        .await
        .expect("settlement service should start");

        let watcher = service
            .request_new_settlement(Some(certificate_id), job)
            .await
            .expect("settlement request should be accepted");
        cancellation_token.cancel();

        assert_eq!(*recorded_job_id.lock().unwrap(), Some(watcher.job_id()));
        assert_eq!(
            ordering.lock().unwrap().as_slice(),
            ["write_link", "write_job"]
        );
    }

    mod admin;
    mod same_wallet_nonce_race;
}
