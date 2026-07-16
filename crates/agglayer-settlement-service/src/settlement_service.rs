use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use agglayer_config::settlement_service::{SettlementServiceConfig, SettlementTransactionConfig};
use agglayer_storage::stores::{SettlementReader, SettlementWriter, StateReader, StateWriter};
use agglayer_types::{CertificateId, SettlementJob, SettlementJobId, SettlementJobResult};
use alloy::providers::{Provider, WalletProvider};
use educe::Educe;
use eyre::Context as _;
use tokio::sync::{watch, Mutex};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

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
    recovery_skipped_jobs: u64,
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
        let mut this = Self {
            tx_config,
            provider,
            store,
            cancellation_token,
            task_controls: Arc::new(Mutex::new(HashMap::new())),
            result_watchers: Arc::new(Mutex::new(HashMap::new())),
            wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
            recovery_skipped_jobs: 0,
        };
        this.recovery_skipped_jobs = this.resume_pending_settlement_jobs().await?;
        Ok(this)
    }

    /// Number of settlement jobs the startup recovery scan skipped because
    /// they could not be loaded. The node records this as a metric; the
    /// service crate stays free of the telemetry dependency.
    pub fn recovery_skipped_jobs(&self) -> u64 {
        self.recovery_skipped_jobs
    }

    /// Returns the number of jobs skipped because they could not be loaded.
    #[tracing::instrument(skip_all)]
    async fn resume_pending_settlement_jobs(&self) -> eyre::Result<u64> {
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
        let mut skipped_jobs = 0u64;
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
            {
                Ok(StoredSettlementJob::Completed(_, _)) => {
                    completed_jobs += 1;
                }
                Ok(StoredSettlementJob::Pending(task)) => {
                    self.spawn_settlement_task(job_id, task, task_control_handle)
                        .await;
                    resumed_jobs += 1;
                }
                // An unloadable job must not prevent node boot: skip it and
                // report loudly so it can be inspected and repaired.
                Err(error) => {
                    error!(
                        ?error,
                        %job_id,
                        "Failed to load settlement job during startup recovery; skipping"
                    );
                    skipped_jobs += 1;
                }
            }
        }

        info!(
            completed_jobs,
            resumed_jobs, skipped_jobs, "Settlement service startup recovery scan completed"
        );
        Ok(skipped_jobs)
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
        let asserter = Asserter::new();
        asserter.push_success(&U64::from(gas_estimate));
        ProviderBuilder::new()
            .wallet(EthereumWallet::from(
                PrivateKeySigner::from_slice(&[0x11; 32]).expect("valid test signing key"),
            ))
            .connect_mocked_client(asserter)
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
        assert!(service.result_watchers.lock().await.is_empty());
    }

    #[tokio::test]
    async fn start_skips_unloadable_jobs_and_keeps_scanning() {
        let mut store = MockStateStore::new();
        let unloadable_job_id = mk_job_id(8);
        let completed_job_id = mk_job_id(9);
        let completed_job = mk_job(9);
        let completed_result = mk_result(9, ContractCallOutcome::Success);

        // The unloadable job comes first: startup must skip it and still
        // process the following one.
        store
            .expect_list_settlement_job_ids()
            .once()
            .return_once(move || Ok(vec![unloadable_job_id, completed_job_id]));
        store
            .expect_get_settlement_job()
            .once()
            .withf(move |requested_job_id| requested_job_id == &unloadable_job_id)
            .return_once(|_| {
                Err(agglayer_storage::error::Error::UnprocessedAction(
                    "corrupt settlement job row".into(),
                ))
            });
        store
            .expect_get_settlement_job()
            .once()
            .withf(move |requested_job_id| requested_job_id == &completed_job_id)
            .return_once(move |_| Ok(Some(completed_job)));
        store
            .expect_get_settlement_job_result()
            .once()
            .withf(move |requested_job_id| requested_job_id == &completed_job_id)
            .return_once(move |_| Ok(Some(completed_result)));

        let service = mk_service(Arc::new(store)).await;

        assert_eq!(service.recovery_skipped_jobs(), 1);
        assert!(service.task_controls.lock().await.is_empty());
        assert!(service.result_watchers.lock().await.is_empty());
    }

    #[tokio::test]
    async fn retrieve_uses_in_memory_watcher_before_storage() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let service = mk_service(Arc::new(store)).await;
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
        assert!(service.result_watchers.lock().await.contains_key(&job_id));
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

    mod same_wallet_nonce_race;
}
