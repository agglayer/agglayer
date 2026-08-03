use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc, time::SystemTime};

use agglayer_config::settlement_service::{SettlementServiceConfig, SettlementTransactionConfig};
use agglayer_storage::stores::{
    EditEvenIfCompleted, SettlementReader, SettlementWriter, StateReader, StateWriter,
};
use agglayer_types::{
    Address, CertificateId, ClientError, Nonce, RpcErrorCode, SettlementAttempt,
    SettlementAttemptResult, SettlementJob, SettlementJobId, SettlementJobResult, SettlementTxHash,
};
use alloy::{
    consensus::Transaction as _,
    network::TransactionResponse as _,
    providers::{Provider, WalletProvider},
};
use educe::Educe;
use eyre::Context as _;
use tokio::sync::{mpsc, watch, Mutex};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::{
    settlement_task::{
        SettlementTask, SettlementTaskRunResult, StoredSettlementJob, TaskAdminCommand,
        TaskControl, TaskControlHandle,
    },
    wallet_nonce_locks::WalletNonceLocks,
};

/// How the live task for a job (if any) was told about an admin mutation.
///
/// Admin mutations are declarative edits of stored state; a running task only
/// picks them up by reloading from storage. Anything but [`Queued`] means
/// the operator should check the job before relying on the edit being live.
///
/// Serializes as `queued` / `absent` / `notify-failed` in admin RPC
/// responses. Keeping serialization on this service-level response is a
/// deliberate pragmatic choice; it is not an `agglayer-types` domain type.
///
/// [`Queued`]: LiveTaskNotification::Queued
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LiveTaskNotification {
    /// A reload command was queued for the running task. Not a wake-up: the
    /// task drains its command queue only at run-loop control checks, and
    /// its waits are interrupted by cancellation, L1 progress, or attempt
    /// deadlines — never by this queue. A task parked in an L1 wait keeps
    /// acting on stale in-memory state until that wait returns. The retry
    /// policy caps individual backoff sleeps, not total wait duration;
    /// settlement polling can continue until the configured settlement
    /// policy is satisfied. Abort the task before editing when prompt
    /// observation matters.
    Queued,
    /// No live task exists for this job. The edit persists and is picked up
    /// whenever a task is started for the job (e.g. on startup recovery).
    Absent,
    /// A live task exists but could not be notified, so it keeps acting on
    /// stale in-memory state until it reloads.
    ///
    /// This covers both a full command queue (the task is alive but
    /// wedged/slow; commands drain only at control checks) and a closed
    /// channel (the task just died or completed). The warning log records
    /// which case occurred. Use `admin_reloadSettlementTask` as the escape
    /// hatch, or abort the task and restart the node.
    NotifyFailed,
}

/// A settlement attempt to register through the admin surface.
///
/// Only the transaction hash is mandatory. The transaction is fetched from L1
/// by hash when available: explicit sender and nonce values must match it, and
/// missing values are resolved from it. An unknown transaction is accepted
/// only when both identity fields are explicit, with a warning. Missing fees
/// fall back to the fetched transaction's fees, or 0 when it is unknown. A
/// missing submission time defaults to now.
#[derive(Clone, Debug)]
pub struct NewSettlementAttempt {
    pub tx_hash: SettlementTxHash,
    pub sender_wallet: Option<Address>,
    pub nonce: Option<Nonce>,
    pub submission_time: Option<SystemTime>,
    pub max_fee_per_gas: Option<u128>,
    pub max_priority_fee_per_gas: Option<u128>,
}

fn tag_admin_storage_error(error: agglayer_storage::error::Error) -> eyre::Report {
    use agglayer_storage::error::Error as E;

    let code = match &error {
        E::SettlementJobNotFound(_)
        | E::SettlementAttemptNotFound { .. }
        | E::SettlementAttemptResultNotRecorded { .. } => RpcErrorCode::NotFound,
        E::SettlementJobAlreadyCompleted(_) => RpcErrorCode::AlreadyCompleted,
        E::SettlementJobNotCompleted(_) => RpcErrorCode::NotCompleted,
        _ => return error.into(),
    };
    eyre::Report::new(error).wrap_err(code)
}

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
    task_controls: Arc<std::sync::Mutex<HashMap<SettlementJobId, TaskControlHandle>>>,
    result_watchers:
        Arc<Mutex<HashMap<SettlementJobId, watch::Receiver<Option<SettlementJobResult>>>>>,
    /// Serializes spawn-capable admin operations so two concurrent calls
    /// cannot both observe "no live task + result present" and spawn two
    /// tasks for one job. Today only force-remove takes this lock. A global
    /// lock is deliberate at this stack's scope: admin operations are rare
    /// and operator-driven, and no create/spawn/run-loop/retrieve hot path
    /// takes it. Revisit per-job keying, like `wallet_nonce_locks` and
    /// `settlement_write_locks`, before adding more operations such as a
    /// future respawn-capable reload.
    admin_operation_lock: Arc<Mutex<()>>,
    /// Per-wallet locks serializing the nonce read-to-save window across
    /// concurrent settlement tasks.
    /// XREF: https://github.com/agglayer/agglayer/issues/1597
    wallet_nonce_locks: Arc<WalletNonceLocks>,
}

struct TaskControlRegistrationGuard {
    job_id: SettlementJobId,
    task_controls: Arc<std::sync::Mutex<HashMap<SettlementJobId, TaskControlHandle>>>,
}

impl Drop for TaskControlRegistrationGuard {
    fn drop(&mut self) {
        self.task_controls
            .lock()
            .expect("settlement task_controls lock poisoned")
            .remove(&self.job_id);
    }
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
    ) -> eyre::Result<(Self, u64)> {
        let this = Self {
            tx_config,
            provider,
            store,
            cancellation_token,
            task_controls: Arc::new(std::sync::Mutex::new(HashMap::new())),
            result_watchers: Arc::new(Mutex::new(HashMap::new())),
            admin_operation_lock: Arc::new(Mutex::new(())),
            wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
        };
        let recovery_skipped_jobs = this.resume_pending_settlement_jobs().await?;
        Ok((this, recovery_skipped_jobs))
    }

    async fn load_stored_job(
        &self,
        job_id: SettlementJobId,
        task_control: TaskControl,
    ) -> eyre::Result<StoredSettlementJob<L1Provider, SettlementStore>> {
        SettlementTask::load(
            job_id,
            self.tx_config.clone(),
            self.provider.clone(),
            self.store.clone(),
            self.wallet_nonce_locks.clone(),
            task_control,
        )
        .await
    }

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
            match self.load_stored_job(job_id, task_control).await {
                Ok(StoredSettlementJob::Completed(_)) => {
                    completed_jobs += 1;
                }
                Ok(StoredSettlementJob::Pending(task)) => {
                    self.spawn_settlement_task(job_id, task, task_control_handle)
                        .await;
                    resumed_jobs += 1;
                }
                // Load fails only when this job's stored rows cannot be read
                // back (corrupt or undecodable data); never expected in
                // normal operation. Such a job must not prevent node boot:
                // skip it and report loudly so it can be inspected and
                // repaired.
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
        // Register the watcher first so a concurrent retrieval observes a
        // harmless pending watcher, never a task without a watcher.
        self.result_watchers
            .lock()
            .await
            .insert(job_id, result_receiver.clone());
        self.task_controls
            .lock()
            .expect("settlement task_controls lock poisoned")
            .insert(job_id, task_control_handle);
        let task_controls = self.task_controls.clone();
        let result_watchers = self.result_watchers.clone();
        let tx_config = self.tx_config.clone();
        let provider = self.provider.clone();
        let store = self.store.clone();
        let wallet_nonce_locks = self.wallet_nonce_locks.clone();
        let cancellation_token = self.cancellation_token.clone();
        tokio::task::spawn(async move {
            let _task_control_registration = TaskControlRegistrationGuard {
                job_id,
                task_controls: task_controls.clone(),
            };
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
                        break;
                    }
                    SettlementTaskRunResult::Cancelled => {
                        info!(?job_id, "Settlement task cancelled");
                        result_watchers.lock().await.remove(&job_id);
                        break;
                    }
                    SettlementTaskRunResult::ReloadAndRestart => {
                        info!(?job_id, "Reloading and restarting settlement task");
                        let (task_control_handle, task_control) =
                            TaskControlHandle::new(&cancellation_token);
                        task_controls
                            .lock()
                            .expect("settlement task_controls lock poisoned")
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
                            Ok(StoredSettlementJob::Completed(result)) => {
                                if let Err(error) = result_sender.send(Some(result)) {
                                    error!(
                                        ?error,
                                        ?job_id,
                                        "Failed to send settlement job result to watchers"
                                    );
                                }
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
        let task_control = self
            .task_controls
            .lock()
            .expect("settlement task_controls lock poisoned")
            .get(&job_id)
            .cloned();
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

    /// Queues a command telling the live task for `job_id`, if any, to drop
    /// its in-memory state and reload from storage, so it observes an admin
    /// edit.
    ///
    /// Best-effort: the edit is already persisted when this runs, and a task
    /// that cannot be notified will still observe it on its next reload.
    /// Queueing does not interrupt a wait in progress; the task acts on the
    /// command at its next control check (see
    /// [`LiveTaskNotification::Queued`]).
    async fn notify_live_task_of_admin_edit(
        &self,
        job_id: SettlementJobId,
    ) -> LiveTaskNotification {
        let task_controls = self
            .task_controls
            .lock()
            .expect("settlement task_controls lock poisoned");
        let Some(task_control) = task_controls.get(&job_id) else {
            return LiveTaskNotification::Absent;
        };

        match task_control.try_send(TaskAdminCommand::ReloadAndRestart) {
            Ok(()) => LiveTaskNotification::Queued,
            Err(error) => {
                warn!(
                    ?job_id,
                    ?error,
                    "Failed to notify live settlement task of an admin edit; the task acts on \
                     stale in-memory state until it reloads"
                );
                LiveTaskNotification::NotifyFailed
            }
        }
    }

    /// Resolves an admin-provided attempt into a full [`SettlementAttempt`].
    ///
    /// The transaction is always queried on L1. When found, its sender and
    /// nonce are authoritative: explicit values must match, and missing values
    /// are filled from it. When it is unknown, both values must be explicit and
    /// are trusted with a warning. Fees use explicit values, then fetched
    /// values, then `0`; zero makes a fee-bumping retry start over from freshly
    /// estimated fees. Submission time uses the explicit value or the current
    /// time, seeding the task's retry backoff for this attempt.
    async fn resolve_new_settlement_attempt(
        &self,
        attempt: NewSettlementAttempt,
    ) -> eyre::Result<SettlementAttempt> {
        let tx_hash = attempt.tx_hash;
        let fetched_tx = self
            .provider
            .get_transaction_by_hash(tx_hash.into())
            .await
            .wrap_err(RpcErrorCode::Unavailable)
            .wrap_err_with(|| {
                format!("Failed to fetch settlement transaction {tx_hash} from L1")
            })?;

        let (sender_wallet, nonce) = match fetched_tx.as_ref() {
            Some(transaction) => {
                let l1_sender_wallet = transaction.from().into();
                let l1_nonce = Nonce(transaction.nonce());

                if let Some(provided_sender_wallet) = attempt.sender_wallet {
                    if provided_sender_wallet != l1_sender_wallet {
                        return Err(eyre::eyre!(
                            "Explicit sender wallet {provided_sender_wallet} does not match L1 \
                             sender wallet {l1_sender_wallet} for settlement transaction {tx_hash}"
                        )
                        .wrap_err(RpcErrorCode::InvalidParams));
                    }
                }
                if let Some(provided_nonce) = attempt.nonce {
                    if provided_nonce != l1_nonce {
                        return Err(eyre::eyre!(
                            "Explicit nonce {provided_nonce} does not match L1 nonce {l1_nonce} \
                             for settlement transaction {tx_hash}"
                        )
                        .wrap_err(RpcErrorCode::InvalidParams));
                    }
                }

                (l1_sender_wallet, l1_nonce)
            }
            None => match (attempt.sender_wallet, attempt.nonce) {
                (Some(sender_wallet), Some(nonce)) => {
                    warn!(
                        %tx_hash,
                        %sender_wallet,
                        %nonce,
                        "Settlement transaction is not known to the L1 RPC; trusting explicitly \
                         provided sender wallet and nonce"
                    );
                    (sender_wallet, nonce)
                }
                _ => {
                    return Err(eyre::eyre!(
                        "Settlement transaction {tx_hash} is not known to the L1 RPC; provide \
                         sender_wallet and nonce explicitly"
                    )
                    .wrap_err(RpcErrorCode::NotFound));
                }
            },
        };

        Ok(SettlementAttempt {
            sender_wallet,
            nonce,
            hash: tx_hash,
            submission_time: attempt.submission_time.unwrap_or_else(SystemTime::now),
            max_fee_per_gas: attempt
                .max_fee_per_gas
                // Fully qualified: the RPC transaction type also offers
                // `TransactionResponse::max_fee_per_gas`.
                .or_else(|| {
                    fetched_tx
                        .as_ref()
                        .map(alloy::consensus::Transaction::max_fee_per_gas)
                })
                .unwrap_or(0),
            max_priority_fee_per_gas: attempt
                .max_priority_fee_per_gas
                .or_else(|| {
                    fetched_tx
                        .as_ref()
                        .and_then(|tx| tx.max_priority_fee_per_gas())
                })
                .unwrap_or(0),
        })
    }

    /// Appends a new settlement attempt to `job_id` and returns its assigned
    /// attempt number.
    ///
    /// This always adds one new attempt under the next unused number and never
    /// overwrites an existing one, so it is safe for porting an externally
    /// submitted settlement transaction into the job.
    #[tracing::instrument(skip(self))]
    pub async fn admin_insert_settlement_attempt(
        &self,
        job_id: SettlementJobId,
        attempt: NewSettlementAttempt,
        edit_even_if_completed: EditEvenIfCompleted,
    ) -> eyre::Result<(u64, LiveTaskNotification)> {
        let attempt = self.resolve_new_settlement_attempt(attempt).await?;
        let attempt_number = self
            .store
            .admin_insert_settlement_attempt(&job_id, &attempt, edit_even_if_completed)
            .map_err(tag_admin_storage_error)
            .wrap_err_with(|| format!("Failed to insert settlement attempt for job {job_id}"))?;
        let live_task = self.notify_live_task_of_admin_edit(job_id).await;
        Ok((attempt_number, live_task))
    }

    /// Records that an administrator asserts the attempt will never land on
    /// L1, overwriting any previously recorded result for it.
    ///
    /// Terminal for the attempt, never for the job: the reloaded task no
    /// longer waits on this attempt and drives the settlement elsewhere.
    #[tracing::instrument(skip(self))]
    pub async fn admin_mark_attempt_definitely_failed(
        &self,
        job_id: SettlementJobId,
        attempt_number: u64,
        reason: &str,
        edit_even_if_completed: EditEvenIfCompleted,
    ) -> eyre::Result<LiveTaskNotification> {
        let result = SettlementAttemptResult::ClientError(ClientError::abandoned_by_admin(reason));
        self.store
            .admin_override_settlement_attempt_result(
                &job_id,
                attempt_number,
                &result,
                edit_even_if_completed,
            )
            .map_err(tag_admin_storage_error)
            .wrap_err_with(|| {
                format!(
                    "Failed to mark settlement attempt {attempt_number} of job {job_id} as \
                     definitely failed"
                )
            })?;
        Ok(self.notify_live_task_of_admin_edit(job_id).await)
    }

    /// Removes the recorded result of an attempt, handing the attempt back to
    /// the settlement task as pending.
    #[tracing::instrument(skip(self))]
    pub async fn admin_remove_attempt_result(
        &self,
        job_id: SettlementJobId,
        attempt_number: u64,
        edit_even_if_completed: EditEvenIfCompleted,
    ) -> eyre::Result<LiveTaskNotification> {
        self.store
            .admin_remove_settlement_attempt_result(&job_id, attempt_number, edit_even_if_completed)
            .map_err(tag_admin_storage_error)
            .wrap_err_with(|| {
                format!(
                    "Failed to remove result of settlement attempt {attempt_number} of job \
                     {job_id}"
                )
            })?;
        Ok(self.notify_live_task_of_admin_edit(job_id).await)
    }

    /// Removes the terminal result of a completed job and spawns a fresh task
    /// for it, so the job is re-driven from its stored state.
    ///
    /// Force operation: if the removed result was real, only the settlement
    /// contract's replay protection stands between the re-driven job and a
    /// double settlement.
    #[tracing::instrument(skip(self))]
    pub async fn admin_force_remove_settlement_job_result(
        &self,
        job_id: SettlementJobId,
    ) -> eyre::Result<()> {
        let _admin_op = self.admin_operation_lock.lock().await;

        // A completed job has no live task. Refusing while one is still
        // registered (e.g. mid-completion, or the job is simply not done)
        // keeps the old task and the fresh one below from racing.
        if self
            .task_controls
            .lock()
            .expect("settlement task_controls lock poisoned")
            .contains_key(&job_id)
        {
            return Err(eyre::eyre!(
                "a settlement task is still live for job {job_id}; its terminal result can only \
                 be removed once it has completed"
            )
            .wrap_err(RpcErrorCode::TaskStillLive));
        }

        self.store
            .admin_force_remove_settlement_job_result(&job_id)
            .map_err(tag_admin_storage_error)
            .wrap_err_with(|| {
                format!("Failed to remove terminal result of settlement job {job_id}")
            })?;

        // Drop the in-memory watcher that still broadcasts the removed
        // result, then bring the job back to life.
        self.result_watchers.lock().await.remove(&job_id);

        let (task_control_handle, task_control) = TaskControlHandle::new(&self.cancellation_token);
        // Nothing stale is registered at this point: the watcher was removed,
        // and the new control handle is not inserted until spawning succeeds.
        // A load failure therefore leaves the job explicitly aborted: pending
        // in storage, with no task, and recoverable by the next node restart.
        let stored_job = self
            .load_stored_job(job_id, task_control)
            .await
            .wrap_err(RpcErrorCode::Unavailable)
            .wrap_err_with(|| {
                format!(
                    "Removed the terminal result of settlement job {job_id} but failed to reload \
                     the job; it will be re-driven after the next node restart"
                )
            })?;

        match stored_job {
            StoredSettlementJob::Pending(task) => {
                // Keep the admin lock until both registrations are complete,
                // so a competing force-remove observes the live task.
                self.spawn_settlement_task(job_id, task, task_control_handle)
                    .await;
                Ok(())
            }
            StoredSettlementJob::Completed(_) => Err(eyre::eyre!(
                "Settlement job {job_id} still has a terminal result right after its removal; was \
                 one re-recorded concurrently?"
            )),
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
