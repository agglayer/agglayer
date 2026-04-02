use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use agglayer_config::settlement_service::SettlementServiceConfig;
use alloy::providers::Provider;
use eyre::Context as _;
use tokio::sync::{mpsc, watch, Mutex};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use ulid::Ulid;

use crate::settlement_task::{
    SettlementJob, SettlementJobResult, SettlementTask, SettlementTaskRunResult, TaskAdminCommand,
    TaskControlHandle,
};

const ADMIN_CHANNEL_BUFFER_SIZE: usize = 10;

/// The Settlement Service is responsible for managing settlement jobs and
/// answering settlement result requests.
#[derive(educe::Educe)]
#[educe(Clone)]
pub struct SettlementService<P> {
    provider: Arc<P>,
    cancellation_token: CancellationToken,
    task_controls: Arc<Mutex<HashMap<Ulid, TaskControlHandle>>>,
    result_watchers: Arc<Mutex<HashMap<Ulid, watch::Receiver<Option<SettlementJobResult>>>>>,
}

pub struct SettlementJobWatcher {
    watcher: watch::Receiver<Option<SettlementJobResult>>,
    job_id: Ulid,
}

impl SettlementJobWatcher {
    pub fn watcher(&mut self) -> &mut watch::Receiver<Option<SettlementJobResult>> {
        &mut self.watcher
    }

    pub fn job_id(&self) -> Ulid {
        self.job_id
    }
}

pub enum RetrievedSettlementResult {
    Pending(SettlementJobWatcher),
    Completed(SettlementJobResult),
}

impl<P: Provider + 'static> SettlementService<P> {
    pub async fn start(
        _config: SettlementServiceConfig,
        provider: Arc<P>,
        cancellation_token: CancellationToken,
    ) -> eyre::Result<Self> {
        let this = Self {
            provider,
            cancellation_token,
            task_controls: Arc::new(Mutex::new(HashMap::new())),
            result_watchers: Arc::new(Mutex::new(HashMap::new())),
        };
        // TODO: load all pending settlements from rocksdb and run them
        Ok(this)
    }

    #[tracing::instrument(skip_all)]
    async fn task_control(&self, job_id: Ulid) -> eyre::Result<TaskControlHandle> {
        let task_controls = self.task_controls.lock().await;
        let Some(task_control) = task_controls.get(&job_id) else {
            eyre::bail!("No task control found for settlement task {job_id}");
        };
        Ok(task_control.clone())
    }

    #[tracing::instrument(skip_all)]
    async fn admin_task(&self, job_id: Ulid, command: TaskAdminCommand) -> eyre::Result<()> {
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
    pub async fn admin_abort_task(&self, job_id: Ulid) -> eyre::Result<()> {
        self.task_control(job_id).await?.cancel();
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn admin_reload_and_restart_task(&self, job_id: Ulid) -> eyre::Result<()> {
        self.admin_task(job_id, TaskAdminCommand::ReloadAndRestart)
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn request_new_settlement(
        &self,
        job: SettlementJob,
    ) -> eyre::Result<SettlementJobWatcher> {
        let (admin_sender, admin_receiver) = mpsc::channel(ADMIN_CHANNEL_BUFFER_SIZE);
        let (result_sender, result_receiver) = watch::channel(None);
        let (task_control_handle, task_control) =
            TaskControlHandle::new(&self.cancellation_token, admin_sender, admin_receiver);
        let (job_id, mut task) =
            SettlementTask::create(job, self.provider.clone(), task_control).await?;
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
        tokio::task::spawn(async move {
            match task.run().await {
                SettlementTaskRunResult::Completed(result) => {
                    if let Err(error) = result_sender.send(Some(result)) {
                        error!(
                            ?error,
                            ?job_id,
                            "Failed to send settlement job result to watchers"
                        );
                    }
                }
                SettlementTaskRunResult::Cancelled => {
                    info!(?job_id, "Settlement task cancelled");
                    result_watchers.lock().await.remove(&job_id);
                }
                SettlementTaskRunResult::ReloadAndRestart => {
                    result_watchers.lock().await.remove(&job_id);
                    task_controls.lock().await.remove(&job_id);
                    todo!("Reload and restart settlement tasks");
                }
            }
            task_controls.lock().await.remove(&job_id);
        });
        Ok(SettlementJobWatcher {
            watcher: result_receiver,
            job_id,
        })
    }

    #[tracing::instrument(skip(self))]
    pub async fn retrieve_settlement_result(
        &self,
        job_id: Ulid,
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
        // TODO: check rocksdb for completed settlement job results
        todo!()
    }
}

pub struct RequestNewSettlement(pub SettlementJob);

impl<P: Provider + 'static> tower::Service<RequestNewSettlement> for SettlementService<P> {
    type Response = SettlementJobWatcher;
    type Error = eyre::Error;
    type Future = Pin<Box<dyn Future<Output = eyre::Result<Self::Response>>>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: RequestNewSettlement) -> Self::Future {
        let this = self.clone();
        Box::pin(async move { this.request_new_settlement(req.0).await })
    }
}

pub struct RetrieveSettlementResult(pub Ulid);

impl<P: Provider + 'static> tower::Service<RetrieveSettlementResult> for SettlementService<P> {
    type Response = RetrievedSettlementResult;
    type Error = eyre::Error;
    type Future = Pin<Box<dyn Future<Output = eyre::Result<Self::Response>>>>;

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
    AbortTask(Ulid),
    ReloadAndRestartTask(Ulid),
}

impl<P: Provider + 'static> tower::Service<AdminCommand> for SettlementService<P> {
    type Response = ();
    type Error = eyre::Error;
    type Future = Pin<Box<dyn Future<Output = eyre::Result<Self::Response>>>>;

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
