use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use agglayer_config::settlement_service::SettlementServiceConfig;
use agglayer_types::{SettlementJob, SettlementJobResult};
use eyre::Context as _;
use tokio::sync::{mpsc, watch, Mutex};
use tokio_util::sync::CancellationToken;
use tracing::error;
use ulid::Ulid;

use crate::settlement_task::{SettlementStore, SettlementTask, TaskAdminCommand};

const ADMIN_CHANNEL_BUFFER_SIZE: usize = 10;

/// The Settlement Service is responsible for managing settlement jobs and
/// answering settlement result requests.
pub struct SettlementService<S>
where
    S: SettlementStore,
{
    admin_command_senders: Arc<Mutex<HashMap<Ulid, mpsc::Sender<TaskAdminCommand>>>>,
    result_watchers: Arc<Mutex<HashMap<Ulid, watch::Receiver<Option<SettlementJobResult>>>>>,
    settlement_store: Arc<S>,
}

impl<S> Clone for SettlementService<S>
where
    S: SettlementStore,
{
    fn clone(&self) -> Self {
        Self {
            admin_command_senders: self.admin_command_senders.clone(),
            result_watchers: self.result_watchers.clone(),
            settlement_store: self.settlement_store.clone(),
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
    S: SettlementStore + 'static,
{
    pub async fn start(
        _config: SettlementServiceConfig,
        settlement_store: Arc<S>,
        cancellation_token: CancellationToken,
    ) -> eyre::Result<Self> {
        let this = Self {
            admin_command_senders: Arc::new(Mutex::new(HashMap::new())),
            result_watchers: Arc::new(Mutex::new(HashMap::new())),
            settlement_store,
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
            SettlementTask::create(job, admin_receiver, self.settlement_store.clone()).await?;
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
        if let Some(result) = self
            .settlement_store
            .get_settlement_job_result(&job_id)
            .wrap_err_with(|| {
                format!("Failed to read settlement job terminal result for id {job_id}")
            })?
        {
            return Ok(RetrievedSettlementResult::Completed(result));
        }

        if let Some(watcher) = self.result_watchers.lock().await.get(&job_id) {
            return match watcher.borrow().as_ref() {
                None => Ok(RetrievedSettlementResult::Pending(SettlementJobWatcher {
                    job_id,
                    watcher: watcher.clone(),
                })),
                Some(result) => Ok(RetrievedSettlementResult::Completed(result.clone())),
            };
        }

        if self
            .settlement_store
            .get_settlement_job(&job_id)
            .wrap_err_with(|| format!("Failed to check settlement job existence for id {job_id}"))?
            .is_none()
        {
            return Err(eyre::eyre!("No settlement job found for id {job_id}"));
        }

        let watcher = {
            let mut watchers = self.result_watchers.lock().await;

            watchers
                .entry(job_id)
                .or_insert_with(|| {
                    let (_sender, watcher) = watch::channel(None);
                    watcher
                })
                .clone()
        };

        Ok(RetrievedSettlementResult::Pending(SettlementJobWatcher {
            watcher,
            job_id,
        }))
    }
}

pub struct RequestNewSettlement(pub SettlementJob);

impl<S> tower::Service<RequestNewSettlement> for SettlementService<S>
where
    S: SettlementStore + 'static,
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
    S: SettlementStore + 'static,
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
    S: SettlementStore + 'static,
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

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Arc};

    use agglayer_storage::{
        backup::BackupClient,
        stores::{state::StateStore, SettlementWriter as _},
    };
    use agglayer_types::{
        Address, ContractCallOutcome, ContractCallResult, Digest, SettlementJob,
        SettlementJobResult, SettlementTxHash, B256, U256,
    };

    use super::*;

    struct TempStateDbDir(PathBuf);

    impl TempStateDbDir {
        fn new() -> Self {
            let path = std::env::temp_dir()
                .join(format!("agglayer-settlement-service-tests-{}", Ulid::new()));

            std::fs::create_dir_all(&path).expect("temporary test directory should be created");

            Self(path)
        }

        fn path(&self) -> &std::path::Path {
            self.0.as_path()
        }
    }

    impl Drop for TempStateDbDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn mk_stored_job(seed: u8) -> SettlementJob {
        SettlementJob {
            contract_address: Address::from([seed; 20]),
            calldata: vec![seed, seed.wrapping_add(1)].into(),
            eth_value: U256::from(seed as u64),
            gas_limit: 21_000,
            max_fee_per_gas_ceiling: 1_000,
            max_fee_per_gas_floor: 500,
            max_fee_per_gas_increase_percents: 125,
            max_priority_fee_per_gas_ceiling: 1_000,
            max_priority_fee_per_gas_floor: 500,
            max_priority_fee_per_gas_increase_percents: 125,
        }
    }

    fn mk_stored_result(seed: u8) -> SettlementJobResult {
        SettlementJobResult::ContractCall(ContractCallResult {
            outcome: ContractCallOutcome::Success,
            metadata: vec![seed, seed.wrapping_add(1)].into(),
            block_hash: B256::from([seed; 32]),
            block_number: seed as u64,
            tx_hash: SettlementTxHash::new(Digest::from([seed.wrapping_add(2); 32])),
        })
    }

    fn mk_in_memory_result(seed: u8) -> SettlementJobResult {
        SettlementJobResult::ContractCall(ContractCallResult {
            outcome: ContractCallOutcome::Revert,
            metadata: vec![seed, seed.wrapping_add(10)].into(),
            block_hash: B256::from([seed.wrapping_add(11); 32]),
            block_number: seed as u64 + 1_000,
            tx_hash: SettlementTxHash::new(Digest::from([seed.wrapping_add(12); 32])),
        })
    }

    fn mk_state_store(path: &std::path::Path) -> Arc<StateStore> {
        Arc::new(
            StateStore::new_with_path(path, BackupClient::noop()).expect("state store should open"),
        )
    }

    #[tokio::test]
    async fn retrieve_uses_stored_terminal_result_before_watcher() {
        let tmp = TempStateDbDir::new();
        let state_store = mk_state_store(tmp.path());
        let cancellation_token = CancellationToken::new();
        let service = SettlementService::start(
            SettlementServiceConfig::default(),
            state_store.clone(),
            cancellation_token.clone(),
        )
        .await
        .expect("settlement service should start");

        let job_id = Ulid::new();
        let stored_result = mk_stored_result(1);

        state_store
            .insert_settlement_job(&job_id, &mk_stored_job(1))
            .expect("job insert should succeed");
        state_store
            .insert_settlement_job_result(&job_id, &stored_result)
            .expect("job result insert should succeed");

        let (_sender, watcher) = watch::channel(Some(mk_in_memory_result(2)));
        service.result_watchers.lock().await.insert(job_id, watcher);

        let retrieved = service
            .retrieve_settlement_result(job_id)
            .await
            .expect("retrieval should succeed");

        match retrieved {
            RetrievedSettlementResult::Completed(result) => {
                assert_eq!(result, stored_result);
            }
            RetrievedSettlementResult::Pending(_) => panic!("expected completed result"),
        }

        cancellation_token.cancel();
    }

    #[tokio::test]
    async fn retrieve_returns_pending_when_job_exists_without_terminal_result() {
        let tmp = TempStateDbDir::new();
        let state_store = mk_state_store(tmp.path());
        let cancellation_token = CancellationToken::new();
        let service = SettlementService::start(
            SettlementServiceConfig::default(),
            state_store.clone(),
            cancellation_token.clone(),
        )
        .await
        .expect("settlement service should start");

        let job_id = Ulid::new();

        state_store
            .insert_settlement_job(&job_id, &mk_stored_job(3))
            .expect("job insert should succeed");

        let retrieved = service
            .retrieve_settlement_result(job_id)
            .await
            .expect("retrieval should succeed");

        match retrieved {
            RetrievedSettlementResult::Pending(mut watcher) => {
                assert!(watcher.watcher().borrow().is_none());
            }
            RetrievedSettlementResult::Completed(_) => panic!("expected pending result"),
        }

        cancellation_token.cancel();
    }

    #[tokio::test]
    async fn retrieve_fails_for_unknown_job_id() {
        let tmp = TempStateDbDir::new();
        let state_store = mk_state_store(tmp.path());
        let cancellation_token = CancellationToken::new();
        let service = SettlementService::start(
            SettlementServiceConfig::default(),
            state_store,
            cancellation_token.clone(),
        )
        .await
        .expect("settlement service should start");

        let result = service.retrieve_settlement_result(Ulid::new()).await;
        assert!(result.is_err(), "unknown job should fail");
        let error = result.err().expect("result should be an error");

        assert!(error.to_string().contains("No settlement job found for id"));

        cancellation_token.cancel();
    }
}
