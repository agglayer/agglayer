use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use agglayer_config::settlement_service::{SettlementServiceConfig, SettlementTransactionConfig};
use agglayer_storage::stores::{SettlementReader, SettlementWriter, StateReader, StateWriter};
use agglayer_types::{
    CertificateId, RpcErrorCode, SettlementJob, SettlementJobId, SettlementJobResult,
};
use alloy::providers::{Provider, WalletProvider};
use educe::Educe;
use eyre::Context as _;
use tokio::sync::{mpsc, watch, Mutex};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::{
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
    result_watchers:
        Arc<Mutex<HashMap<SettlementJobId, watch::Receiver<Option<SettlementJobResult>>>>>,
    /// Per-wallet locks serializing the nonce read-to-save window across
    /// concurrent settlement tasks.
    /// XREF: https://github.com/agglayer/agglayer/issues/1597
    wallet_nonce_locks: Arc<WalletNonceLocks>,
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
            wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
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
        let wallet_nonce_locks = self.wallet_nonce_locks.clone();
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
                            wallet_nonce_locks.clone(),
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

    /// Classifies why no in-memory task control exists for `job_id`, by
    /// reading storage. Only called on error paths (no task control entry, or
    /// a TOCTOU race against a just-finished task), so the extra storage
    /// reads are irrelevant for cost.
    async fn classify_missing_task(&self, job_id: SettlementJobId) -> eyre::Report {
        let job = match self.store.get_settlement_job(&job_id) {
            Ok(job) => job,
            Err(error) => {
                return eyre::Report::new(error).wrap_err(format!(
                    "Failed to check settlement job existence for id {job_id}"
                ));
            }
        };

        if job.is_none() {
            return eyre::eyre!("no settlement job found for id {job_id}")
                .wrap_err(RpcErrorCode::NotFound);
        }

        match self.store.get_settlement_job_result(&job_id) {
            Ok(Some(_)) => eyre::eyre!("settlement job {job_id} already completed")
                .wrap_err(RpcErrorCode::AlreadyCompleted),
            Ok(None) => eyre::eyre!("no live settlement task for pending job {job_id}")
                .wrap_err(RpcErrorCode::NoLiveTask),
            Err(error) => eyre::Report::new(error).wrap_err(format!(
                "Failed to read settlement job terminal result for id {job_id}"
            )),
        }
    }

    #[tracing::instrument(skip_all)]
    async fn task_control(&self, job_id: SettlementJobId) -> eyre::Result<TaskControlHandle> {
        let task_control = self.task_controls.lock().await.get(&job_id).cloned();
        match task_control {
            Some(task_control) => Ok(task_control),
            None => Err(self.classify_missing_task(job_id).await),
        }
    }

    #[tracing::instrument(skip_all)]
    async fn admin_task(
        &self,
        job_id: SettlementJobId,
        command: TaskAdminCommand,
    ) -> eyre::Result<()> {
        let task_control = self.task_control(job_id).await?;
        match task_control.try_send(command) {
            Ok(()) => Ok(()),
            Err(error @ mpsc::error::TrySendError::Full(_)) => Err(eyre::Report::new(error)
                .wrap_err(format!(
                    "Failed to forward admin command to settlement task {job_id}: admin command \
                     queue full"
                ))
                .wrap_err(RpcErrorCode::Unavailable)),
            // The task completed or died between the `task_control` lookup above and this
            // `try_send` call; classify the same way as a missing task control entry.
            Err(mpsc::error::TrySendError::Closed(_)) => {
                Err(self.classify_missing_task(job_id).await)
            }
        }
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
        Err(
            eyre::eyre!("Pending settlement job {job_id} exists without a running task")
                .wrap_err(RpcErrorCode::NoLiveTask),
        )
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
