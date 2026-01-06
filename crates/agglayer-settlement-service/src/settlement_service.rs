use std::{collections::HashMap, sync::Arc};

use agglayer_config::settlement_service::SettlementServiceConfig;
use tokio::sync::{mpsc, watch, Mutex};
use tokio_util::sync::CancellationToken;
use tracing::error;
use ulid::Ulid;

use crate::settlement_task::{
    SettlementJob, SettlementJobResult, SettlementTask, TaskAdminCommand,
};

pub enum ServiceAdminCommand {
    TaskCommand {
        job_id: Ulid,
        command: TaskAdminCommand,
    },
}

pub struct SettlementService {
    admin_command_senders: Arc<Mutex<HashMap<Ulid, mpsc::Sender<TaskAdminCommand>>>>,
    result_watchers: Arc<Mutex<HashMap<Ulid, SettlementJobWatcher>>>,
}

pub type SettlementJobWatcher = watch::Receiver<Option<SettlementJobResult>>;

pub enum RetrievedSettlementResult {
    Pending(SettlementJobWatcher),
    Completed(SettlementJobResult),
}

impl SettlementService {
    pub async fn start(
        _config: SettlementServiceConfig,
        admin_commands: mpsc::Receiver<ServiceAdminCommand>,
        cancellation_token: CancellationToken,
    ) -> eyre::Result<Self> {
        let this = Self {
            admin_command_senders: Arc::new(Mutex::new(HashMap::new())),
            result_watchers: Arc::new(Mutex::new(HashMap::new())),
        };
        tokio::task::spawn(Self::admin_command_proxy(
            admin_commands,
            this.admin_command_senders.clone(),
        ));
        tokio::task::spawn(Self::cancellation_token_proxy(
            cancellation_token,
            this.admin_command_senders.clone(),
        ));
        // TODO: load all pending settlements from rocksdb and run them
        Ok(this)
    }

    #[tracing::instrument(skip_all)]
    async fn admin_command_proxy(
        mut admin_commands: mpsc::Receiver<ServiceAdminCommand>,
        senders: Arc<Mutex<HashMap<Ulid, mpsc::Sender<TaskAdminCommand>>>>,
    ) {
        while let Some(command) = admin_commands.recv().await {
            match command {
                ServiceAdminCommand::TaskCommand { job_id, command } => {
                    let senders = senders.lock().await;
                    let sender = match senders.get(&job_id) {
                        Some(sender) => sender,
                        None => {
                            error!(?job_id, "No admin command sender found for settlement task");
                            continue;
                        }
                    };
                    if let Err(error) = sender.try_send(command) {
                        error!(
                            ?error,
                            ?job_id,
                            "Failed to forward admin command to settlement task, did it already \
                             complete?"
                        );
                    }
                }
            }
        }
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

    #[tracing::instrument(skip(self))]
    pub async fn request_new_settlement(
        &self,
        job: SettlementJob,
    ) -> eyre::Result<(Ulid, SettlementJobWatcher)> {
        let (admin_sender, admin_receiver) = mpsc::channel(10);
        let (result_sender, result_receiver) = watch::channel(None);
        let (job_id, mut task) = SettlementTask::create(job, admin_receiver).await?;
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
        Ok((job_id, result_receiver))
    }

    #[tracing::instrument(skip(self))]
    pub async fn retrieve_settlement_result(
        &self,
        job_id: Ulid,
    ) -> eyre::Result<RetrievedSettlementResult> {
        if let Some(watcher) = self.result_watchers.lock().await.get(&job_id) {
            return match watcher.borrow().as_ref() {
                None => Ok(RetrievedSettlementResult::Pending(watcher.clone())),
                Some(result) => Ok(RetrievedSettlementResult::Completed(result.clone())),
            };
        }
        // TODO: check rocksdb for completed settlement job results
        todo!()
    }
}
