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
    result_watchers:
        Arc<Mutex<HashMap<SettlementJobId, watch::Receiver<Option<SettlementJobResult>>>>>,
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
            result_watchers: Arc::new(Mutex::new(HashMap::new())),
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
    /// Returns before the task observes the cancellation; watch the actual
    /// teardown through [`Self::has_live_task`]. `Ok(())` can race with task
    /// completion, where the cancel is a no-op on a lingering handle. An
    /// abort chained with a reload-and-restart can drop the queued reload if
    /// the task exits on the cancellation first; retry once
    /// [`Self::has_live_task`] is `false`. The storage classification behind
    /// the returned errors is a best-effort snapshot.
    #[tracing::instrument(skip(self))]
    pub async fn admin_abort_task(
        &self,
        job_id: SettlementJobId,
    ) -> Result<(), SettlementAdminError> {
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
    /// concurrently cancelled can exit without draining it, and its
    /// channel can close between the liveness check and the send,
    /// failing the reload with `TaskNotResponding`. Retry once
    /// [`Self::has_live_task`] reports `false`; the respawn path then
    /// picks the job up from storage.
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
            Ok(StoredSettlementJob::Completed(_, _)) => {
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

    #[tokio::test]
    async fn admin_abort_task_unknown_job_returns_job_not_found() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(20);
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

        let error = service
            .admin_abort_task(job_id)
            .await
            .expect_err("abort of an unknown job must fail");
        assert!(matches!(
            error,
            crate::SettlementAdminError::JobNotFound(id) if id == job_id
        ));
    }

    #[tokio::test]
    async fn admin_abort_task_completed_job_returns_job_completed() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(21);
        let result = mk_result(21, ContractCallOutcome::Success);
        store
            .expect_get_settlement_job_result()
            .once()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .return_once(move |_| Ok(Some(result)));

        let service = mk_service(Arc::new(store)).await;

        let error = service
            .admin_abort_task(job_id)
            .await
            .expect_err("abort of a completed job must fail");
        assert!(matches!(
            error,
            crate::SettlementAdminError::JobCompleted(id) if id == job_id
        ));
    }

    #[tokio::test]
    async fn admin_abort_task_pending_job_without_task_returns_no_live_task() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(22);
        let job = mk_job(22);
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

        let error = service
            .admin_abort_task(job_id)
            .await
            .expect_err("abort without a live task must fail");
        assert!(matches!(
            error,
            crate::SettlementAdminError::NoLiveTask(id) if id == job_id
        ));
    }

    #[tokio::test]
    async fn admin_abort_task_storage_error_on_result_read_returns_storage_error() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(24);
        store
            .expect_get_settlement_job_result()
            .once()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .return_once(|_| {
                Err(agglayer_storage::error::Error::Unexpected(
                    "boom".to_string(),
                ))
            });

        let service = mk_service(Arc::new(store)).await;

        let error = service
            .admin_abort_task(job_id)
            .await
            .expect_err("abort must surface the storage error");
        assert!(matches!(error, crate::SettlementAdminError::Storage { .. }));
    }

    #[tokio::test]
    async fn admin_abort_task_storage_error_on_job_read_returns_storage_error() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(25);
        store
            .expect_get_settlement_job_result()
            .once()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .return_once(|_| Ok(None));
        store
            .expect_get_settlement_job()
            .once()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .return_once(|_| {
                Err(agglayer_storage::error::Error::Unexpected(
                    "boom".to_string(),
                ))
            });

        let service = mk_service(Arc::new(store)).await;

        let error = service
            .admin_abort_task(job_id)
            .await
            .expect_err("abort must surface the storage error");
        assert!(matches!(error, crate::SettlementAdminError::Storage { .. }));
    }

    #[tokio::test]
    async fn admin_reload_and_restart_task_unknown_job_returns_job_not_found() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(26);
        // Tolerant mocks: how often the reload path consults storage before
        // classifying the miss is an implementation detail.
        store
            .expect_get_settlement_job_result()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .returning(|_| Ok(None));
        store
            .expect_get_settlement_job()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .returning(|_| Ok(None));

        let service = mk_service(Arc::new(store)).await;

        let error = service
            .admin_reload_and_restart_task(job_id)
            .await
            .expect_err("reload of an unknown job must fail");
        assert!(matches!(
            error,
            crate::SettlementAdminError::JobNotFound(id) if id == job_id
        ));
    }

    /// Loads a pending job through `SettlementTask::load` and spawns its
    /// task, mirroring the reload test above. The caller provides the
    /// storage expectations for the load (job, no result, no attempts).
    async fn load_and_spawn_pending_task(
        service: &SettlementService<impl Provider + WalletProvider + 'static, MockStateStore>,
        job_id: SettlementJobId,
    ) {
        let (task_control_handle, task_control) =
            TaskControlHandle::new(&service.cancellation_token);
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
            StoredSettlementJob::Completed(_, _) => {
                panic!("load should find a pending job")
            }
        };
        service
            .spawn_settlement_task(job_id, task, task_control_handle)
            .await;
    }

    /// Storage expectations for loading one pending job, tolerant of the
    /// extra reads the spawned task performs before it gets cancelled.
    fn expect_pending_job_reads(store: &mut MockStateStore, seed: u8) {
        let job = mk_job(seed);
        store
            .expect_get_settlement_job()
            .returning(move |_| Ok(Some(job.clone())));
        store
            .expect_get_settlement_job_result()
            .returning(|_| Ok(None));
        store
            .expect_list_settlement_attempts()
            .returning(|_| Ok(Vec::new()));
        store
            .expect_list_settlement_attempt_results()
            .returning(|_| Ok(Vec::new()));
        store
            .expect_max_settlement_nonce_for_wallet()
            .returning(|_| Ok(None));
    }

    #[tokio::test]
    async fn admin_abort_task_cancels_live_task() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(23);
        expect_pending_job_reads(&mut store, 23);

        let service = mk_service(Arc::new(store)).await;
        // Determinism assumption: on the current_thread test runtime the
        // spawned task is first polled after `cancel()` below, so it exits at
        // the loop-top control check without touching the dead provider
        // endpoint.
        load_and_spawn_pending_task(&service, job_id).await;
        assert!(service.has_live_task(job_id).await);

        service
            .admin_abort_task(job_id)
            .await
            .expect("abort of a live task must succeed");

        // The task observes the cancellation asynchronously.
        tokio::time::timeout(std::time::Duration::from_secs(10), async {
            while service.has_live_task(job_id).await {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("aborted task should deregister its control handle");
    }

    #[tokio::test]
    async fn admin_reload_and_restart_respawns_dead_task() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(24);
        expect_pending_job_reads(&mut store, 24);

        let cancellation_token = CancellationToken::new();
        let service = mk_service_with_token(Arc::new(store), cancellation_token.clone()).await;
        assert!(!service.has_live_task(job_id).await);

        // Determinism assumption: on the current_thread test runtime the
        // spawned task is first polled after `cancel()` below, so it exits at
        // the loop-top control check without touching the dead provider
        // endpoint.
        service
            .admin_reload_and_restart_task(job_id)
            .await
            .expect("reload of a pending job without a task must respawn it");

        assert!(service.has_live_task(job_id).await);

        // A retrieval after the respawn gets a functioning watcher.
        let retrieved = service
            .retrieve_settlement_result(job_id)
            .await
            .expect("retrieval after respawn should succeed");
        assert!(matches!(retrieved, RetrievedSettlementResult::Pending(_)));

        cancellation_token.cancel();
    }

    #[tokio::test]
    async fn admin_reload_and_restart_completed_job_returns_job_completed() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(25);
        let job = mk_job(25);
        let result = mk_result(25, ContractCallOutcome::Success);
        store
            .expect_get_settlement_job()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .returning(move |_| Ok(Some(job.clone())));
        store
            .expect_get_settlement_job_result()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .returning(move |_| Ok(Some(result.clone())));

        let service = mk_service(Arc::new(store)).await;

        let error = service
            .admin_reload_and_restart_task(job_id)
            .await
            .expect_err("reload of a completed job must fail");
        assert!(matches!(
            error,
            crate::SettlementAdminError::JobCompleted(id) if id == job_id
        ));
    }

    #[tokio::test]
    async fn admin_reload_and_restart_live_task_sends_command() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(27);
        // Strict counts: the initial explicit load is the only permitted
        // read, so a regression that respawned instead of sending the
        // command would trip these expectations with a second load.
        let job = mk_job(27);
        store
            .expect_get_settlement_job()
            .times(1)
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .returning(move |_| Ok(Some(job.clone())));
        store
            .expect_get_settlement_job_result()
            .times(1)
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .returning(|_| Ok(None));
        store
            .expect_list_settlement_attempts()
            .times(1)
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .returning(|_| Ok(Vec::new()));
        store
            .expect_list_settlement_attempt_results()
            .times(1)
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .returning(|_| Ok(Vec::new()));
        store
            .expect_max_settlement_nonce_for_wallet()
            .returning(|_| Ok(None));

        let cancellation_token = CancellationToken::new();
        let service = mk_service_with_token(Arc::new(store), cancellation_token.clone()).await;
        // Determinism assumption: on the current_thread test runtime the
        // spawned task is first polled after `cancel()` below, so it exits at
        // the loop-top control check without touching the dead provider
        // endpoint.
        load_and_spawn_pending_task(&service, job_id).await;

        service
            .admin_reload_and_restart_task(job_id)
            .await
            .expect("reload of a live task must be accepted");
        // No second load occurred: the command was queued to the live
        // task, not respawned. The task stays registered.
        assert!(service.has_live_task(job_id).await);

        cancellation_token.cancel();
    }

    #[tokio::test]
    async fn admin_reload_and_restart_load_failure_returns_reload_failed() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(28);
        let job = mk_job(28);
        // The job row exists (both for the load and for the failure
        // classification re-read), but hydrating the attempts fails, so
        // the load fails on a job that is still present.
        store
            .expect_get_settlement_job()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .returning(move |_| Ok(Some(job.clone())));
        store
            .expect_get_settlement_job_result()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .returning(|_| Ok(None));
        store
            .expect_list_settlement_attempt_results()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .returning(|_| Ok(Vec::new()));
        store
            .expect_list_settlement_attempts()
            .withf(move |requested_job_id| requested_job_id == &job_id)
            .returning(|_| {
                Err(agglayer_storage::error::Error::Unexpected(
                    "boom".to_string(),
                ))
            });

        let service = mk_service(Arc::new(store)).await;

        let error = service
            .admin_reload_and_restart_task(job_id)
            .await
            .expect_err("reload must surface the load failure");
        assert!(matches!(
            error,
            crate::SettlementAdminError::ReloadFailed { .. }
        ));
    }

    #[tokio::test]
    async fn admin_reload_and_restart_respawns_after_task_panic() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(29);
        expect_pending_job_reads(&mut store, 29);

        let cancellation_token = CancellationToken::new();
        let service = mk_service_with_token(Arc::new(store), cancellation_token.clone()).await;

        // Simulate a panicked task: its control handle stays registered
        // while the receiver side is gone.
        let (task_control_handle, task_control) =
            TaskControlHandle::new(&service.cancellation_token);
        drop(task_control);
        service
            .task_controls
            .lock()
            .await
            .insert(job_id, task_control_handle);
        assert!(service.has_live_task(job_id).await);

        service
            .admin_reload_and_restart_task(job_id)
            .await
            .expect("reload must respawn over the stale entry of a panicked task");

        // The respawned task registered a fresh, working handle.
        assert!(service.has_live_task(job_id).await);
        let control = service
            .task_controls
            .lock()
            .await
            .get(&job_id)
            .cloned()
            .expect("control handle must be registered");
        assert!(!control.is_closed());

        cancellation_token.cancel();
    }

    mod same_wallet_nonce_race;
}
