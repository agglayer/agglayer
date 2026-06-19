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

use crate::nonce_allocator::NonceAllocatorRegistry;
use crate::settlement_task::{
    SettlementTask, SettlementTaskRunResult, TaskAdminCommand, TaskControlHandle,
};

const ADMIN_CHANNEL_BUFFER_SIZE: usize = 10;

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
    nonce_allocators: Arc<NonceAllocatorRegistry>,
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
            nonce_allocators: Arc::new(NonceAllocatorRegistry::new()),
        };
        // TODO: load all pending settlements from rocksdb and run them
        // When spawning recovered tasks, pass `nonce_allocators` and seed from DB + RPC.
        // XREF: https://github.com/agglayer/agglayer/issues/1230
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
            self.nonce_allocators.clone(),
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

    mod nonce_allocator_integration {
        use std::{
            sync::{Arc, Mutex},
            time::Duration,
        };

        use agglayer_storage::tests::mocks::MockStateStore;
        use agglayer_types::{Nonce, SettlementJob};
        use alloy::{
            node_bindings::Anvil,
            primitives::U256,
            providers::{Provider, WalletProvider},
            rpc::types::TransactionRequest,
        };

        use super::*;
        use crate::utils::build_provider;

        fn mk_settlement_job() -> SettlementJob {
            SettlementJob {
                contract_address: agglayer_types::Address::from([1; 20]),
                calldata: vec![2, 3].into(),
                eth_value: U256::from(0),
                gas_limit: 100_000,
            }
        }

        fn mock_store_recording_attempts(
            job_count: usize,
            attempt_count: usize,
            captured_nonces: Arc<Mutex<Vec<u64>>>,
        ) -> MockStateStore {
            let mut store = MockStateStore::new();
            store
                .expect_insert_settlement_job()
                .times(job_count)
                .returning(|_, _| Ok(()));
            store
                .expect_insert_settlement_attempt()
                .times(attempt_count)
                .returning(move |_, _, attempt| {
                    captured_nonces.lock().unwrap().push(attempt.nonce.0);
                    Ok(())
                });
            store
        }

        async fn mk_anvil_service(
            provider: Arc<impl Provider + WalletProvider + 'static>,
            store: Arc<MockStateStore>,
        ) -> SettlementService<impl Provider + WalletProvider + 'static, MockStateStore> {
            SettlementService::start(
                SettlementServiceConfig::default(),
                Arc::new(SettlementTransactionConfig::default()),
                provider,
                store,
                CancellationToken::new(),
            )
            .await
            .expect("settlement service should start")
        }

        async fn wait_for_recorded_nonces(captured_nonces: &Arc<Mutex<Vec<u64>>>, count: usize) {
            tokio::time::timeout(Duration::from_secs(10), async {
                loop {
                    if captured_nonces.lock().unwrap().len() >= count {
                        return;
                    }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            })
            .await
            .expect("timed out waiting for settlement attempts to be recorded");
        }

        #[tokio::test]
        async fn concurrent_jobs_receive_distinct_nonces() {
            let anvil = Anvil::new().spawn();
            let provider = Arc::new(build_provider(&anvil));
            let captured_nonces = Arc::new(Mutex::new(Vec::new()));
            let store = Arc::new(mock_store_recording_attempts(
                2,
                2,
                captured_nonces.clone(),
            ));
            let service = Arc::new(mk_anvil_service(provider, store).await);

            let service_a = service.clone();
            let service_b = service.clone();
            let (watcher_a, watcher_b) = tokio::join!(
                async {
                    service_a
                        .request_new_settlement(mk_settlement_job())
                        .await
                        .expect("job a")
                },
                async {
                    service_b
                        .request_new_settlement(mk_settlement_job())
                        .await
                        .expect("job b")
                },
            );

            wait_for_recorded_nonces(&captured_nonces, 2).await;

            service
                .admin_abort_task(watcher_a.job_id())
                .await
                .expect("abort job a");
            service
                .admin_abort_task(watcher_b.job_id())
                .await
                .expect("abort job b");

            let mut nonces = captured_nonces.lock().unwrap().clone();
            nonces.sort_unstable();
            assert_eq!(nonces.len(), 2, "expected two recorded attempts");
            assert_ne!(nonces[0], nonces[1], "concurrent jobs must not share a nonce");
            assert_eq!(
                nonces[1],
                nonces[0] + 1,
                "concurrent jobs should receive gap-free nonces"
            );
        }

        #[tokio::test]
        async fn sequential_jobs_increment_nonce() {
            let anvil = Anvil::new().spawn();
            let provider = Arc::new(build_provider(&anvil));
            let captured_nonces = Arc::new(Mutex::new(Vec::new()));
            let store = Arc::new(mock_store_recording_attempts(
                2,
                2,
                captured_nonces.clone(),
            ));
            let service = Arc::new(mk_anvil_service(provider, store).await);

            let watcher_a = service
                .request_new_settlement(mk_settlement_job())
                .await
                .expect("job a");
            wait_for_recorded_nonces(&captured_nonces, 1).await;
            service
                .admin_abort_task(watcher_a.job_id())
                .await
                .expect("abort job a");

            let watcher_b = service
                .request_new_settlement(mk_settlement_job())
                .await
                .expect("job b");
            wait_for_recorded_nonces(&captured_nonces, 2).await;
            service
                .admin_abort_task(watcher_b.job_id())
                .await
                .expect("abort job b");

            let nonces = captured_nonces.lock().unwrap().clone();
            assert_eq!(nonces.len(), 2);
            assert_eq!(nonces[1], nonces[0] + 1);
        }

        #[tokio::test]
        async fn external_nonce_use_advances_allocator() {
            let anvil = Anvil::new().spawn();
            let provider = Arc::new(build_provider(&anvil));
            let store = Arc::new(MockStateStore::new());
            let service = mk_anvil_service(provider.clone(), store).await;
            let wallet = provider.default_signer_address();

            let chain_pending = provider
                .get_transaction_count(wallet)
                .pending()
                .await
                .expect("read pending nonce for seed");
            service
                .nonce_allocators
                .seed_if_unseeded(wallet, chain_pending)
                .await;
            let reserved = service.nonce_allocators.handout(wallet).await;
            assert_eq!(reserved, 0);

            provider
                .send_transaction(
                    TransactionRequest::default()
                        .to(anvil.addresses()[1])
                        .value(U256::from(1))
                        .nonce(reserved),
                )
                .await
                .expect("send external transaction")
                .get_receipt()
                .await
                .expect("mine external transaction");

            let chain_pending = provider
                .get_transaction_count(wallet)
                .pending()
                .await
                .expect("read pending nonce");
            service
                .nonce_allocators
                .reconcile_next_pending(wallet, chain_pending)
                .await;

            let next = service.nonce_allocators.handout(wallet).await;
            assert_eq!(next, reserved + 1);
            assert_eq!(next, Nonce(1).0);
        }
    }
}
