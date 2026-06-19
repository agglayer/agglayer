use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use agglayer_config::settlement_service::{SettlementServiceConfig, SettlementTransactionConfig};
use agglayer_storage::stores::{SettlementReader, SettlementWriter};
use agglayer_types::{SettlementJob, SettlementJobId, SettlementJobResult};
use alloy::providers::{Provider, WalletProvider};
use educe::Educe;
use eyre::Context as _;
use tokio::sync::{mpsc, watch, Mutex};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::settlement_task::{
    SettlementTask, SettlementTaskRunResult, TaskAdminCommand, TaskControlHandle,
};

const ADMIN_CHANNEL_BUFFER_SIZE: usize = 10;

type ResultWatchersMap =
    Arc<Mutex<HashMap<SettlementJobId, watch::Receiver<Option<SettlementJobResult>>>>>;
type TaskControlsMap = Arc<Mutex<HashMap<SettlementJobId, TaskControlHandle>>>;

async fn remove_job_from_active_tracking(
    job_id: SettlementJobId,
    result_watchers: &ResultWatchersMap,
    task_controls: &TaskControlsMap,
) {
    result_watchers.lock().await.remove(&job_id);
    task_controls.lock().await.remove(&job_id);
}

/// Ensures in-memory job tracking is removed when a spawned settlement task
/// ends abnormally (e.g. panics) before explicit cleanup can run.
struct JobTrackingGuard {
    job_id: SettlementJobId,
    result_watchers: ResultWatchersMap,
    task_controls: TaskControlsMap,
    armed: bool,
}

impl JobTrackingGuard {
    fn new(
        job_id: SettlementJobId,
        result_watchers: ResultWatchersMap,
        task_controls: TaskControlsMap,
    ) -> Self {
        Self {
            job_id,
            result_watchers,
            task_controls,
            armed: true,
        }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for JobTrackingGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }

        let job_id = self.job_id;
        let result_watchers = self.result_watchers.clone();
        let task_controls = self.task_controls.clone();
        tokio::spawn(async move {
            remove_job_from_active_tracking(job_id, &result_watchers, &task_controls).await;
        });
    }
}

/// The Settlement Service is responsible for managing settlement jobs and
/// answering settlement result requests.
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
}

pub enum RetrievedSettlementResult {
    Pending(SettlementJobWatcher),
    Completed(SettlementJobResult),
}

impl<
        L1Provider: Provider + WalletProvider + 'static,
        SettlementStore: SettlementReader + SettlementWriter + Send + Sync + 'static,
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
        // TODO: load all pending settlements from rocksdb and run them
        Ok(this)
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
        job: SettlementJob,
    ) -> eyre::Result<SettlementJobWatcher> {
        let (admin_sender, admin_receiver) = mpsc::channel(ADMIN_CHANNEL_BUFFER_SIZE);
        let (result_sender, result_receiver) = watch::channel(None);
        let (task_control_handle, task_control) =
            TaskControlHandle::new(&self.cancellation_token, admin_sender, admin_receiver);
        let (job_id, mut task) = SettlementTask::create(
            job,
            self.tx_config.clone(),
            self.provider.clone(),
            self.store.clone(),
            task_control,
        )
        .await?;
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
            let mut cleanup_guard =
                JobTrackingGuard::new(job_id, result_watchers.clone(), task_controls.clone());

            let run_result = task.run().await;

            match run_result {
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
                }
                SettlementTaskRunResult::ReloadAndRestart => {
                    cleanup_guard.disarm();
                    remove_job_from_active_tracking(job_id, &result_watchers, &task_controls).await;
                    todo!("Reload and restart settlement tasks");
                }
            }

            remove_job_from_active_tracking(job_id, &result_watchers, &task_controls).await;
            cleanup_guard.disarm();
        });
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

        // TODO: Spawn a settlement task for a recovered pending job.
        // XREF: https://github.com/agglayer/agglayer/issues/1230
        todo!()
    }
}

pub struct RequestNewSettlement(pub SettlementJob);

impl<
        L1Provider: Provider + WalletProvider + 'static,
        SettlementStore: SettlementReader + SettlementWriter + Send + Sync + 'static,
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
        Box::pin(async move { this.request_new_settlement(req.0).await })
    }
}

pub struct RetrieveSettlementResult(pub SettlementJobId);

impl<
        L1Provider: Provider + WalletProvider + 'static,
        SettlementStore: SettlementReader + SettlementWriter + Send + Sync + 'static,
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
        SettlementStore: SettlementReader + SettlementWriter + Send + Sync + 'static,
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
mod tests {
    use std::sync::Arc;

    use agglayer_storage::tests::mocks::MockStateStore;
    use agglayer_types::{
        ContractCallOutcome, ContractCallResult, Digest, Nonce, SettlementAttemptNumber,
        SettlementJobId, SettlementJobResult, SettlementTxHash, B256,
    };
    use alloy::{
        network::EthereumWallet, providers::ProviderBuilder, signers::local::PrivateKeySigner,
    };

    use super::*;

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

    async fn mk_service(
        store: Arc<MockStateStore>,
    ) -> SettlementService<impl Provider + WalletProvider + 'static, MockStateStore> {
        SettlementService::start(
            SettlementServiceConfig::default(),
            Arc::new(SettlementTransactionConfig::default()),
            Arc::new(mk_provider()),
            store,
            CancellationToken::new(),
        )
        .await
        .expect("settlement service should start")
    }

    fn mk_job_id(seed: u128) -> SettlementJobId {
        SettlementJobId::from(ulid::Ulid::from(seed))
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

    async fn insert_active_job_tracking(
        service: &SettlementService<impl Provider + WalletProvider + 'static, MockStateStore>,
        job_id: SettlementJobId,
        watcher: watch::Receiver<Option<SettlementJobResult>>,
    ) {
        let (admin_sender, admin_receiver) = mpsc::channel(ADMIN_CHANNEL_BUFFER_SIZE);
        let (handle, _control) =
            TaskControlHandle::new(&CancellationToken::new(), admin_sender, admin_receiver);
        service.task_controls.lock().await.insert(job_id, handle);
        service.result_watchers.lock().await.insert(job_id, watcher);
    }

    #[tokio::test]
    async fn remove_job_from_active_tracking_clears_both_maps() {
        let store = Arc::new(MockStateStore::new());
        let service = mk_service(store).await;
        let job_id = mk_job_id(10);
        let (_sender, watcher) = watch::channel(None);

        insert_active_job_tracking(&service, job_id, watcher).await;

        remove_job_from_active_tracking(job_id, &service.result_watchers, &service.task_controls)
            .await;

        assert!(service.result_watchers.lock().await.is_empty());
        assert!(service.task_controls.lock().await.is_empty());
    }

    #[tokio::test]
    async fn retrieve_uses_storage_after_active_tracking_cleanup() {
        let mut store = MockStateStore::new();
        let job_id = mk_job_id(11);
        let stored_result = mk_result(11, ContractCallOutcome::Success);
        let stored_result_for_store = stored_result.clone();

        store
            .expect_get_settlement_job_result()
            .once()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .return_once(move |_| Ok(Some(stored_result_for_store)));

        let service = mk_service(Arc::new(store)).await;
        let (_sender, watcher) = watch::channel(Some(stored_result.clone()));
        insert_active_job_tracking(&service, job_id, watcher).await;

        remove_job_from_active_tracking(job_id, &service.result_watchers, &service.task_controls)
            .await;

        let retrieved = service
            .retrieve_settlement_result(job_id)
            .await
            .expect("retrieval should succeed");

        match retrieved {
            RetrievedSettlementResult::Completed(result) => assert_eq!(result, stored_result),
            RetrievedSettlementResult::Pending(_) => {
                panic!("expected completed result from storage after map cleanup")
            }
        }
    }

    #[tokio::test]
    async fn caller_watcher_retains_result_after_active_tracking_cleanup() {
        let store = Arc::new(MockStateStore::new());
        let service = mk_service(store).await;
        let job_id = mk_job_id(12);
        let result = mk_result(12, ContractCallOutcome::Success);

        let (sender, caller_watcher) = watch::channel(None);
        sender
            .send(Some(result.clone()))
            .expect("result should be sent to watcher");
        insert_active_job_tracking(&service, job_id, caller_watcher.clone()).await;

        remove_job_from_active_tracking(job_id, &service.result_watchers, &service.task_controls)
            .await;

        assert_eq!(
            caller_watcher.borrow().as_ref(),
            Some(&result),
            "caller watcher should retain the result after service map cleanup"
        );
    }

    #[tokio::test]
    async fn job_tracking_guard_cleans_up_on_drop_without_disarm() {
        use std::time::Duration;

        let store = Arc::new(MockStateStore::new());
        let service = mk_service(store).await;
        let job_id = mk_job_id(13);
        let (_sender, watcher) = watch::channel(None);
        insert_active_job_tracking(&service, job_id, watcher).await;

        let guard = JobTrackingGuard::new(
            job_id,
            service.result_watchers.clone(),
            service.task_controls.clone(),
        );
        drop(guard);

        tokio::time::sleep(Duration::from_millis(10)).await;

        assert!(service.result_watchers.lock().await.is_empty());
        assert!(service.task_controls.lock().await.is_empty());
    }

    #[tokio::test]
    async fn retrieve_uses_in_memory_watcher_before_storage() {
        let store = Arc::new(MockStateStore::new());
        let service = mk_service(store.clone()).await;
        let job_id = mk_job_id(1);
        let in_memory_result = mk_result(2, ContractCallOutcome::Revert);

        let (_sender, watcher) = watch::channel(Some(in_memory_result.clone()));
        service.result_watchers.lock().await.insert(job_id, watcher);

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
}
