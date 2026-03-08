use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use agglayer_config::settlement_service::SettlementServiceConfig;
use agglayer_storage::stores::{SettlementReader, SettlementWriter};
use eyre::Context as _;
use tokio::sync::{mpsc, watch, Mutex};
use tokio_util::sync::CancellationToken;
use tracing::error;
use ulid::Ulid;

use crate::settlement_task::{
    derive_terminal_result_from_storage_records, SettlementJob, SettlementJobResult,
    SettlementTask, TaskAdminCommand,
};

const ADMIN_CHANNEL_BUFFER_SIZE: usize = 10;

/// The Settlement Service is responsible for managing settlement jobs and
/// answering settlement result requests.
pub struct SettlementService<S> {
    store: Arc<S>,
    admin_command_senders: Arc<Mutex<HashMap<Ulid, mpsc::Sender<TaskAdminCommand>>>>,
    result_watchers: Arc<Mutex<HashMap<Ulid, watch::Receiver<Option<SettlementJobResult>>>>>,
}

impl<S> Clone for SettlementService<S> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            admin_command_senders: self.admin_command_senders.clone(),
            result_watchers: self.result_watchers.clone(),
        }
    }
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

impl<S> SettlementService<S>
where
    S: SettlementReader + SettlementWriter + 'static,
{
    pub async fn start(
        _config: SettlementServiceConfig,
        cancellation_token: CancellationToken,
        store: Arc<S>,
    ) -> eyre::Result<Self> {
        let this = Self {
            store,
            admin_command_senders: Arc::new(Mutex::new(HashMap::new())),
            result_watchers: Arc::new(Mutex::new(HashMap::new())),
        };
        tokio::task::spawn(Self::cancellation_token_proxy(
            cancellation_token,
            this.admin_command_senders.clone(),
        ));
        // TODO: load all pending settlements from rocksdb and run them
        Ok(this)
    }

    #[tracing::instrument(skip_all)]
    async fn cancellation_token_proxy(
        cancellation_token: CancellationToken,
        senders: Arc<Mutex<HashMap<Ulid, mpsc::Sender<TaskAdminCommand>>>>,
    ) {
        cancellation_token.cancelled().await;
        let senders = senders.lock().await;
        for (job_id, sender) in senders.iter() {
            if let Err(error) = sender.try_send(TaskAdminCommand::Abort) {
                error!(
                    ?error,
                    ?job_id,
                    "Failed to forward abort command to settlement task during service shutdown"
                );
            }
        }
    }

    #[tracing::instrument(skip_all)]
    async fn admin_task(&self, job_id: Ulid, command: TaskAdminCommand) -> eyre::Result<()> {
        let senders = self.admin_command_senders.lock().await;
        let Some(sender) = senders.get(&job_id) else {
            return Err(eyre::eyre!(
                "No admin command sender found for settlement task {job_id}"
            ));
        };
        sender.try_send(command).wrap_err_with(|| {
            format!(
                "Failed to forward admin command to settlement task {job_id}, did it already \
                 complete?"
            )
        })?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn admin_abort_task(&self, job_id: Ulid) -> eyre::Result<()> {
        self.admin_task(job_id, TaskAdminCommand::Abort).await
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
        let (job_id, mut task) =
            SettlementTask::create(self.store.clone(), job, admin_receiver).await?;
        self.admin_command_senders
            .lock()
            .await
            .insert(job_id, admin_sender);
        self.result_watchers
            .lock()
            .await
            .insert(job_id, result_receiver.clone());
        tokio::task::spawn(async move {
            let result = task.run().await;
            if let Err(error) = result_sender.send(Some(result)) {
                error!(
                    ?error,
                    ?job_id,
                    "Failed to send settlement job result to watchers"
                );
            }
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

        if self.store.get_settlement_job(&job_id)?.is_none() {
            return Err(eyre::eyre!(
                "No settlement job found for id {job_id} in memory or storage"
            ));
        }

        let attempts = self.store.list_settlement_attempts(&job_id)?;
        let attempt_results = self.store.list_settlement_attempt_results(&job_id)?;

        if let Some(result) =
            derive_terminal_result_from_storage_records(&attempts, &attempt_results)?
        {
            return Ok(RetrievedSettlementResult::Completed(result));
        }

        Err(eyre::eyre!(
            "Settlement job {job_id} has no active watcher and no terminal result in storage"
        ))
    }
}

pub struct RequestNewSettlement(pub SettlementJob);

impl<S> tower::Service<RequestNewSettlement> for SettlementService<S>
where
    S: SettlementReader + SettlementWriter + 'static,
{
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

impl<S> tower::Service<RetrieveSettlementResult> for SettlementService<S>
where
    S: SettlementReader + SettlementWriter + 'static,
{
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

impl<S> tower::Service<AdminCommand> for SettlementService<S>
where
    S: SettlementReader + SettlementWriter + 'static,
{
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
