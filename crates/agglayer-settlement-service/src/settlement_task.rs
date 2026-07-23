use std::{
    collections::{BTreeMap, BTreeSet},
    future::IntoFuture as _,
    sync::{Arc, OnceLock},
    time::{Duration, SystemTime},
};

use agglayer_config::{
    settlement_service::{SettlementPolicy, SettlementTransactionConfig},
    Multiplier,
};
use agglayer_storage::stores::{SettlementReader, SettlementWriter, StateReader, StateWriter};
use agglayer_types::{
    CertificateId, ClientError, ClientErrorType, ContractCallOutcome, ContractCallResult, Digest,
    Nonce, SettlementAttempt, SettlementAttemptNumber, SettlementAttemptResult, SettlementJob,
    SettlementJobId, SettlementJobResult, SettlementTxHash,
};
use alloy::{
    consensus::{BlockHeader as _, EthereumTxEnvelope, Transaction as _, TxEip4844Variant},
    eips::{eip1559::Eip1559Estimation, eip2718::Encodable2718 as _, BlockNumberOrTag},
    network::{
        BlockResponse as _, Ethereum, ReceiptResponse as _, TransactionBuilder as _,
        TransactionBuilderError,
    },
    primitives::{Address, TxHash},
    providers::{Provider, WalletProvider},
    rpc::types::TransactionRequest,
    transports::{TransportError, TransportErrorKind},
};
use eyre::Context as _;
use tokio::sync::{mpsc, OwnedMutexGuard};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, warn};

use crate::{utils::RetryCallbackError, wallet_nonce_locks::WalletNonceLocks};

type TxEnvelope = EthereumTxEnvelope<TxEip4844Variant>;

/// Fully-resolved gas parameters for a single settlement attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GasParams {
    gas_limit: u64,
    max_fee_per_gas: u128,
    max_priority_fee_per_gas: u128,
}

/// Clamps `value` into `[floor, ceiling]`. When `floor > ceiling` the ceiling
/// wins (unlike [`Ord::clamp`], this never panics on an inverted range).
fn clamp_u128(value: u128, floor: u128, ceiling: u128) -> u128 {
    value.max(floor).min(ceiling)
}

/// Minimum replacement bump accepted by standard execution-layer clients
/// (geth's default `txpool.pricebump` of 10%), expressed as a multiplier.
const MIN_REPLACEMENT_BUMP: Multiplier = Multiplier::from_u64_per_1000(1100);

/// Increases one EIP-1559 fee field relative to `previous`, while tracking a
/// rising network `fresh_adjusted` estimate and clamping to `[floor, ceiling]`.
///
/// The effective multiplier is at least [`MIN_REPLACEMENT_BUMP`], so a returned
/// value is always `>= previous * 1.10` — enough for a standard node to accept
/// the replacement. Returns `None` when the ceiling caps the result below that
/// minimum, i.e. the fee can no longer be strictly bumped.
fn bump_fee(
    previous: u128,
    fresh_adjusted: u128,
    configured_multiplier: Multiplier,
    floor: u128,
    ceiling: u128,
) -> Option<u128> {
    let effective_multiplier = configured_multiplier.max(MIN_REPLACEMENT_BUMP);
    let required_min = MIN_REPLACEMENT_BUMP.saturating_mul_u128(previous);
    let target = effective_multiplier
        .saturating_mul_u128(previous)
        .max(fresh_adjusted);
    let bumped = clamp_u128(target, floor, ceiling);
    (bumped >= required_min).then_some(bumped)
}

/// Error surfaced while building a settlement attempt.
#[derive(Debug, thiserror::Error)]
enum BuildAttemptError {
    #[error("L1 RPC error while building settlement attempt: {0}")]
    Transport(#[from] TransportError),
    #[error("failed to assign settlement nonce: {0}")]
    NonceAssignment(eyre::Error),
    #[error("failed to build or sign settlement transaction: {0}")]
    Build(#[from] TransactionBuilderError<Ethereum>),
}

impl BuildAttemptError {
    /// True for an opaque signer-backend failure (e.g. a remote KMS error
    /// wrapped in [`alloy::signers::Error::Other`]), which may be a transient
    /// blip or a permanent misconfiguration that cannot be told apart here.
    fn is_signer_backend(&self) -> bool {
        matches!(
            self,
            Self::Build(TransactionBuilderError::Signer(
                alloy::signers::Error::Other(_)
            ))
        )
    }
}

/// Retry policy for building a settlement attempt.
///
/// Transport (L1 RPC) failures are retried for as long as they look transient.
/// Opaque signer-backend failures are retried a bounded number of times — long
/// enough to ride out a transient remote-signer (e.g. GCP KMS) blip, but not
/// forever, so a permanent signer misconfiguration eventually surfaces through
/// the non-recoverable path instead of looping until cancellation. Structural
/// signer errors and other build failures are non-recoverable immediately.
struct BuildRetryPolicy {
    signer_failures: u32,
}

impl BuildRetryPolicy {
    const MAX_SIGNER_BUILD_RETRIES: u32 = 3;

    fn new() -> Self {
        Self { signer_failures: 0 }
    }

    fn should_retry(&mut self, error: &BuildAttemptError) -> bool {
        match error {
            BuildAttemptError::Transport(error) => crate::utils::is_transient_alloy_error(error),
            error if error.is_signer_backend() => {
                self.signer_failures += 1;
                self.signer_failures <= Self::MAX_SIGNER_BUILD_RETRIES
            }
            BuildAttemptError::NonceAssignment(_) => false,
            BuildAttemptError::Build(_) => false,
        }
    }
}

/// Returns the minimum selected settlement-head block number required for a
/// transaction receipt to be considered settled.
///
/// `receipt_block_number` is the block that included the transaction.
/// `confirmations` is inclusive of that block, so 0 or 1 confirmations settle
/// at `receipt_block_number`, while 2 confirmations require the selected head
/// (`latest`, `safe`, or `finalized`) to be at least one block later.
fn required_settlement_head_number(receipt_block_number: u64, confirmations: usize) -> u64 {
    let confirmation_offset = confirmations.saturating_sub(1);
    let confirmation_offset = confirmation_offset.try_into().unwrap_or(u64::MAX);

    receipt_block_number.saturating_add(confirmation_offset)
}

/// Why submitting a settlement attempt to L1 did not complete normally.
#[derive(Debug)]
enum SubmitAttemptError {
    /// The task was cancelled mid-submission. The already-saved attempt is left
    /// pending so it resumes on reload, and the runner is told to stop.
    Cancelled,
    /// Submission failed for a non-transient reason and should be recorded.
    Failed(eyre::Error),
}

/// Maps the result of broadcasting a settlement attempt to a submit outcome.
///
/// Cancellation becomes [`SubmitAttemptError::Cancelled`] so the caller can
/// leave the already-saved attempt pending and propagate a stop signal to the
/// runner, instead of recording a spurious `ClientError` or silently continuing
/// into the post-submit wait after shutdown.
///
/// A non-transient error becomes [`SubmitAttemptError::Failed`]. Note that a
/// re-broadcast whose first response was lost can return an "already
/// known"/nonce-used error reported here as failure even though the transaction
/// was accepted; recognizing those responses as success depends on the RPC
/// error classification deferred to `SettlementServiceConfig`.
/// XREF: https://github.com/agglayer/agglayer/issues/1321
fn submission_outcome(
    result: Result<(), RetryCallbackError<TransportError>>,
) -> Result<(), SubmitAttemptError> {
    match result {
        Ok(()) => Ok(()),
        Err(RetryCallbackError::Cancelled) => Err(SubmitAttemptError::Cancelled),
        Err(RetryCallbackError::Error(error)) => {
            Err(SubmitAttemptError::Failed(eyre::Error::new(error)))
        }
    }
}

/// Error returned by the L1 polling callbacks; the "not yet" variants are
/// transient and tell the retry loop to keep polling.
#[derive(Debug)]
enum WaitForSettlementError {
    NotSettledYet,
    NotIncludedYet,
    Transport(TransportError),
}

impl WaitForSettlementError {
    fn is_transient(&self) -> bool {
        match self {
            Self::NotSettledYet | Self::NotIncludedYet => true,
            Self::Transport(error) => crate::utils::is_transient_alloy_error(error),
        }
    }

    /// Whether the retry loop should log this transient error at warning
    /// level. The "not yet" variants are the expected steady state while
    /// polling for inclusion or settlement of a submitted transaction, so they
    /// are only worth debug logs; transport errors are anomalies worth
    /// surfacing.
    fn needs_warning_log(&self) -> bool {
        match self {
            Self::NotSettledYet | Self::NotIncludedYet => false,
            Self::Transport(_) => true,
        }
    }
}

impl From<TransportError> for WaitForSettlementError {
    fn from(error: TransportError) -> Self {
        Self::Transport(error)
    }
}

#[derive(Debug, thiserror::Error)]
#[error(
    "assumed non-recoverable in settlement task {settlement_task_id} at {file}:{line}: \
     {error_message}"
)]
struct NonRecoverableError {
    settlement_task_id: SettlementJobId,
    file: &'static str,
    line: u32,
    error_message: String,
}
pub enum StoredSettlementJob<L1Provider, SettlementStore> {
    Pending(SettlementTask<L1Provider, SettlementStore>),
    Completed(SettlementJob, SettlementJobResult),
}

pub enum TaskAdminCommand {
    ReloadAndRestart,
}

const ADMIN_CHANNEL_BUFFER_SIZE: usize = 10;

pub struct TaskControl {
    cancellation_token: CancellationToken,
    admin_commands: mpsc::Receiver<TaskAdminCommand>,
}

#[derive(Clone)]
pub struct TaskControlHandle {
    cancellation_token: CancellationToken,
    admin_commands: mpsc::Sender<TaskAdminCommand>,
}

impl TaskControlHandle {
    pub fn new(parent_cancellation_token: &CancellationToken) -> (Self, TaskControl) {
        let (admin_commands, admin_command_receiver) = mpsc::channel(ADMIN_CHANNEL_BUFFER_SIZE);
        let cancellation_token = parent_cancellation_token.child_token();
        (
            Self {
                cancellation_token: cancellation_token.clone(),
                admin_commands,
            },
            TaskControl {
                cancellation_token,
                admin_commands: admin_command_receiver,
            },
        )
    }

    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    pub fn try_send(
        &self,
        command: TaskAdminCommand,
    ) -> Result<(), mpsc::error::TrySendError<TaskAdminCommand>> {
        self.admin_commands.try_send(command)
    }
}

pub enum SettlementTaskRunResult {
    Completed(SettlementJobResult),
    Cancelled,
    ReloadAndRestart,
}

enum TaskControlAction {
    Cancelled,
    ReloadAndRestart,
}

struct ActiveSettlementAttempt {
    attempt: SettlementAttempt,
    result: Option<SettlementAttemptResult>,
}

type ActiveSettlementAttempts =
    BTreeMap<(Address, Nonce), BTreeMap<SettlementAttemptNumber, ActiveSettlementAttempt>>;

fn hydrate_settlement_attempts(
    attempts: impl IntoIterator<Item = (impl Into<SettlementAttemptNumber>, SettlementAttempt)>,
    results: impl IntoIterator<Item = (impl Into<SettlementAttemptNumber>, SettlementAttemptResult)>,
    job_id: SettlementJobId,
) -> eyre::Result<ActiveSettlementAttempts> {
    let mut results_by_attempt_number = BTreeMap::new();
    for (attempt_number, result) in results {
        let attempt_number = attempt_number.into();
        debug!(%job_id, %attempt_number, ?result, "Loaded settlement attempt result from storage");
        if results_by_attempt_number
            .insert(attempt_number, result)
            .is_some()
        {
            eyre::bail!("Duplicate settlement attempt result {attempt_number} for job {job_id}",);
        }
    }

    let mut loaded_attempt_numbers = BTreeSet::new();
    let mut loaded_attempts = ActiveSettlementAttempts::new();
    for (attempt_number, attempt) in attempts {
        let attempt_number = attempt_number.into();
        debug!(%job_id, %attempt_number, ?attempt, "Loaded settlement attempt from storage");
        if !loaded_attempt_numbers.insert(attempt_number) {
            eyre::bail!("Duplicate settlement attempt {attempt_number} for job {job_id}",);
        }

        let result = results_by_attempt_number.remove(&attempt_number);
        loaded_attempts
            .entry((attempt.sender_wallet.into_alloy(), attempt.nonce))
            .or_default()
            .insert(attempt_number, ActiveSettlementAttempt { attempt, result });
    }

    if let Some((attempt_number, _)) = results_by_attempt_number.first_key_value() {
        eyre::bail!(
            "Settlement attempt result {attempt_number} exists for job {job_id} without a \
             recorded settlement attempt",
        );
    }

    Ok(loaded_attempts)
}

pub struct SettlementTask<L1Provider, SettlementStore> {
    id: SettlementJobId,
    job: SettlementJob,
    tx_config: Arc<SettlementTransactionConfig>,
    provider: Arc<L1Provider>,
    store: Arc<SettlementStore>,
    /// Shared per-wallet locks from
    /// [`SettlementService`](crate::SettlementService); held across the
    /// nonce read-to-save window in [`Self::run`].
    /// XREF: https://github.com/agglayer/agglayer/issues/1597
    wallet_nonce_locks: Arc<WalletNonceLocks>,
    control: TaskControl,
    attempts: ActiveSettlementAttempts,
}

static ID_GENERATOR: OnceLock<std::sync::Mutex<ulid::Generator>> = OnceLock::new();

/// The settlement call without nonce, gas, or fees — shared by gas estimation
/// and the final attempt build.
fn settlement_call_request(job: &SettlementJob, wallet: Address) -> TransactionRequest {
    TransactionRequest::default()
        .from(wallet)
        .to(job.contract_address.into_alloy())
        .value(job.eth_value)
        .input(job.calldata.clone().into())
}

/// Resolve the gas limit (`min(multiplier × estimateGas, ceiling)`) before the
/// job and cert->job-id link are persisted, so a deterministic `estimateGas`
/// failure fails here rather than on every restart of a persisted job.
/// Transient RPC failures retry; deterministic ones propagate.
async fn resolve_settlement_gas_limit<P: Provider + WalletProvider>(
    provider: &P,
    tx_config: &SettlementTransactionConfig,
    mut job: SettlementJob,
    cancellation_token: &CancellationToken,
) -> eyre::Result<SettlementJob> {
    let wallet = provider.default_signer_address();
    let request = settlement_call_request(&job, wallet);

    let gas_estimate = crate::utils::retry_alloy_callback_until_success(
        &tx_config.retry_on_transient_failure,
        cancellation_token,
        || provider.estimate_gas(request.clone()).into_future(),
    )
    .await
    .map_err(|error| match error {
        RetryCallbackError::Cancelled => {
            eyre::eyre!("settlement gas estimation cancelled before completion")
        }
        RetryCallbackError::Error(error) => {
            eyre::eyre!("failed to estimate settlement gas limit: {error}")
        }
    })?;

    job.gas_limit = tx_config
        .gas_limit_multiplier_factor
        .saturating_mul_u128(gas_estimate as u128)
        .min(tx_config.gas_limit_ceiling.saturating_to::<u128>());
    Ok(job)
}

impl<
        L1Provider: Provider + WalletProvider + 'static,
        SettlementStore: SettlementReader + SettlementWriter + StateReader + StateWriter,
    > SettlementTask<L1Provider, SettlementStore>
{
    pub async fn create(
        certificate_id: Option<CertificateId>,
        job: SettlementJob,
        tx_config: Arc<SettlementTransactionConfig>,
        provider: Arc<L1Provider>,
        store: Arc<SettlementStore>,
        wallet_nonce_locks: Arc<WalletNonceLocks>,
        control: TaskControl,
    ) -> eyre::Result<(SettlementJobId, Self)> {
        let job = resolve_settlement_gas_limit(
            provider.as_ref(),
            tx_config.as_ref(),
            job,
            &control.cancellation_token,
        )
        .await?;
        let id = Self::reserve_settlement_job_id(store.as_ref(), certificate_id).await?;
        let this = Self {
            id,
            job,
            tx_config,
            provider,
            store,
            wallet_nonce_locks,
            control,
            attempts: BTreeMap::new(),
        };
        this.save_settlement_job_to_db().await?;
        Ok((id, this))
    }

    async fn reserve_settlement_job_id(
        store: &SettlementStore,
        certificate_id: Option<CertificateId>,
    ) -> eyre::Result<SettlementJobId> {
        let Some(certificate_id) = certificate_id else {
            return Ok(Self::generate_settlement_job_id().await);
        };

        let settlement_job_id = Self::generate_settlement_job_id().await;
        store
            .insert_certificate_settlement_job_id(&certificate_id, &settlement_job_id)
            .wrap_err_with(|| {
                format!(
                    "Failed to write settlement job id {settlement_job_id} for certificate \
                     {certificate_id}"
                )
            })?;

        Ok(settlement_job_id)
    }

    async fn generate_settlement_job_id() -> SettlementJobId {
        loop {
            if let Ok(id) = ID_GENERATOR
                .get_or_init(|| std::sync::Mutex::new(ulid::Generator::new()))
                .lock()
                .unwrap()
                .generate()
            {
                return SettlementJobId::from(id);
            }
            tokio::time::sleep(std::time::Duration::from_micros(100)).await;
        }
    }

    pub async fn load(
        id: SettlementJobId,
        tx_config: Arc<SettlementTransactionConfig>,
        provider: Arc<L1Provider>,
        store: Arc<SettlementStore>,
        wallet_nonce_locks: Arc<WalletNonceLocks>,
        control: TaskControl,
    ) -> eyre::Result<StoredSettlementJob<L1Provider, SettlementStore>> {
        let (job, result) = Self::load_settlement_job_from_db(store.as_ref(), id).await?;
        if let Some(result) = result {
            Ok(StoredSettlementJob::Completed(job, result))
        } else {
            let mut this = SettlementTask {
                id,
                job,
                tx_config,
                provider,
                store,
                wallet_nonce_locks,
                control,
                attempts: BTreeMap::new(),
            };
            this.load_settlement_attempts_from_db()?;
            Ok(StoredSettlementJob::Pending(this))
        }
    }

    pub async fn run(&mut self) -> SettlementTaskRunResult {
        let settlement_task_id = self.id;

        macro_rules! retry {
            ($result:expr, $($format_args:tt)*) => {
                match $result {
                    Ok(value) => value,
                    Err(RetryCallbackError::Cancelled) => {
                        return SettlementTaskRunResult::Cancelled;
                    }
                    Err(RetryCallbackError::Error(error)) => {
                        panic!(
                            "{:#?}",
                            eyre::Error::from(error).wrap_err(NonRecoverableError {
                                settlement_task_id,
                                file: file!(),
                                line: line!(),
                                error_message: format!($($format_args)*),
                            })
                        )
                    }
                }
            };
        }

        'start: loop {
            if let Some(run_result) = self.try_handle_control_action() {
                return run_result;
            }

            // Process in a big loop. We'll come back here whenever a reorg is detected, and
            // after waiting when we're done with one cycle.

            // First, for each nonce we know of, identify whether it is done or whether we
            // need to submit more txes for it. For this, we'll keep a list of
            // nonces used externally and reverts (that are not finalized yet), as well as
            // helper markers.
            let mut nonces_used_externally = BTreeMap::new();
            let mut reverts = BTreeMap::new();
            let mut not_included_on_l1 = BTreeSet::new();
            let mut all_nonces_seen_on_l1 = true;
            let mut need_to_submit_attempt_with_new_nonce = true;
            'nonces: for (wallet, nonce) in self.nonces_in_processing_order() {
                if let Some(run_result) = self.try_handle_control_action() {
                    return run_result;
                }

                let tx_hash_on_l1 = retry!(
                    self.tx_hash_on_l1_for_nonce(wallet, nonce).await,
                    "querying nonce inclusion on L1 for wallet {wallet} / nonce {nonce}",
                );
                if let Some(tx_hash) = tx_hash_on_l1 {
                    // If the nonce is used on L1, we won't need to submit any new tx related to it.
                    let Some(attempt_number) =
                        self.settlement_attempt_number_for(wallet, nonce, tx_hash)
                    else {
                        nonces_used_externally.insert((wallet, nonce), tx_hash);
                        continue 'nonces;
                    };
                    let tx_result = retry!(
                        self.current_result_on_l1_for(wallet, nonce, tx_hash).await,
                        "querying current result on L1 for tx {tx_hash}",
                    );
                    let Some(tx_result) = tx_result else {
                        continue 'start; // reorg
                    };
                    if tx_result.outcome != ContractCallOutcome::Success {
                        reverts.insert((wallet, nonce), (attempt_number, tx_hash, tx_result));
                        continue 'nonces;
                    }
                    let settlement_result = retry!(
                        self.wait_for_settlement_of(tx_hash).await,
                        "waiting for settlement of tx {tx_hash}",
                    );
                    let Some(settled_result) = settlement_result else {
                        continue 'start; // reorg
                    };
                    if settled_result != tx_result {
                        continue 'start; // reorg
                    }
                    let job_result = self
                        .write_job_result_to_db(wallet, nonce, attempt_number, tx_result.clone())
                        .await;
                    return SettlementTaskRunResult::Completed(job_result);
                } else {
                    // If the nonce is not used on L1, we'll need to either wait more or submit a
                    // new attempt with the same nonce.
                    all_nonces_seen_on_l1 = false;
                    not_included_on_l1.insert((wallet, nonce));
                    if !self.is_wallet_privkey_known(wallet) {
                        continue 'nonces; // we don't have access to the wallet
                                          // any longer, so it makes no sense to
                                          // check if we need to resubmit.
                    }
                    // This nonce is not included yet and we still know the privkey, so we won't
                    // need to submit an attempt with a new nonce, regardless of whether we
                    // resubmit.
                    need_to_submit_attempt_with_new_nonce = false;
                    if self.is_any_attempt_pending_for_nonce(wallet, nonce) {
                        // At least one attempt is not in-error yet, so we'll need to wait for the
                        // previous nonce to be included before processing it further.
                        if let Some(previous_nonce) = nonce.previous() {
                            let previous_nonce_on_l1 = retry!(
                                self.tx_hash_on_l1_for_nonce(wallet, previous_nonce).await,
                                "querying previous nonce inclusion on L1 for wallet {wallet} / \
                                 nonce {previous_nonce}",
                            );
                            if previous_nonce_on_l1.is_none() {
                                continue 'nonces; // wait for previous nonce to
                                                  // be included
                            }
                        }
                    }
                    let deadline = self.next_attempt_deadline_for_nonce(wallet, nonce);
                    if deadline > SystemTime::now() {
                        continue 'nonces; // wait for deadline to be reached
                    }
                    let Some((attempt_number, tx)) = retry!(
                        self.build_next_attempt_with_nonce(wallet, nonce).await,
                        "building next settlement attempt for wallet {wallet} / nonce {nonce}",
                    ) else {
                        continue 'nonces; // ceiling reached; keep waiting on
                                          // the existing attempt
                    };
                    if let Some(run_result) = self
                        .save_attempt_to_db_and_submit_to_l1(
                            None,
                            wallet,
                            nonce,
                            attempt_number,
                            tx,
                        )
                        .await
                    {
                        return run_result;
                    }
                }
            }
            if all_nonces_seen_on_l1 && !reverts.is_empty() {
                // All nonces were seen on L1, but we didn't get a successful settlement result
                // for any of them. Also, there was at least one revert.
                // We can wait for finalization without submitting a new attempt.
                let (
                    earliest_revert_wallet,
                    earliest_revert_nonce,
                    earliest_revert_attempt_number,
                    earliest_revert_result,
                ) = reverts
                    .iter()
                    .map(|(&(wallet, nonce), (attempt_number, _, result))| {
                        (wallet, nonce, *attempt_number, result.clone())
                    })
                    .min_by_key(|(_, _, _, result)| result.block_number)
                    .unwrap(); // No panic: we checked `!reverts.is_empty()` just before.
                for (wallet, nonce) in self.all_used_nonces() {
                    if let Some(tx_hash) = nonces_used_externally.remove(&(wallet, nonce)) {
                        let settlement_result = retry!(
                            self.wait_for_settlement_of(tx_hash).await,
                            "waiting for settlement of externally-used tx {tx_hash}",
                        );
                        if settlement_result.is_none() {
                            continue 'start; // reorg
                        }
                        self.write_nonce_used_externally_to_db(wallet, nonce, tx_hash)
                            .await;
                    } else if let Some((attempt_number, tx_hash, result)) =
                        reverts.remove(&(wallet, nonce))
                    {
                        let settlement_result = retry!(
                            self.wait_for_settlement_of(tx_hash).await,
                            "waiting for settlement of reverting tx {tx_hash}",
                        );
                        let Some(settled_result) = settlement_result else {
                            continue 'start; // reorg
                        };
                        if settled_result != result {
                            continue 'start; // reorg
                        }
                        self.write_nonce_revert_to_db(wallet, nonce, attempt_number, result)
                            .await;
                    } else {
                        // Invariant: If we finish the `'nonces` loop with `all_nonces_seen_on_l1`,
                        // all nonces must be one of success, revert or external use.
                        // Any success would have led to either an early return, or a loop back to
                        // `'start` if it did not settle properly.
                        // As such, we must have entered at least one of the two branches above for
                        // each nonce.
                        panic!(
                            "Settlement logic invariant broken: nonces seen on L1 must be either \
                             success, revert or external use"
                        );
                    }
                }
                let job_result = self
                    .write_job_result_to_db(
                        earliest_revert_wallet,
                        earliest_revert_nonce,
                        earliest_revert_attempt_number,
                        earliest_revert_result,
                    )
                    .await;
                return SettlementTaskRunResult::Completed(job_result);
            }
            // There was no successful attempt, and either at least one nonce was not yet
            // seen on L1 or there is no reverting attempt. So we need to wait
            // for more nonces to be seen on L1.
            if need_to_submit_attempt_with_new_nonce {
                // There was no attempt that was pending or that received a retry in the
                // `'nonces` loop above. This means that either all nonces were
                // used externally, or that we no longer have the required wallets to bump
                // pending nonces. So we need to submit a new attempt with a new
                // nonce.
                //
                // Hold the wallet's nonce lock from before the nonce is read
                // until the attempt is saved, so no other same-wallet task
                // can pick the same nonce in that window; XREF:
                // https://github.com/agglayer/agglayer/issues/1597.
                let locked_wallet = self.provider.default_signer_address();
                // Race the lock wait against cancellation: the holder may be
                // stuck in transient L1 retries, and an aborted task must not
                // stay parked in the lock queue until the holder releases.
                let nonce_guard = tokio::select! {
                    biased;
                    _ = self.control.cancellation_token.cancelled() => {
                        return SettlementTaskRunResult::Cancelled;
                    }
                    guard = self.wallet_nonce_locks.lock(locked_wallet) => guard,
                };
                let (wallet, nonce, attempt_number, tx) = retry!(
                    self.build_next_attempt_with_new_nonce().await,
                    "building next settlement attempt with a new nonce",
                );
                // The build derives its wallet the same way; if wallet
                // selection ever becomes dynamic, the lock key must follow.
                debug_assert_eq!(wallet, locked_wallet);
                not_included_on_l1.insert((wallet, nonce));
                if let Some(run_result) = self
                    .save_attempt_to_db_and_submit_to_l1(
                        Some(nonce_guard),
                        wallet,
                        nonce,
                        attempt_number,
                        tx,
                    )
                    .await
                {
                    return run_result;
                }
            }
            // We now are sure we did at least one step to make things move forward. Wait
            // for the next external event or for the next deadline.
            let timeout = self
                .next_overall_deadline()
                .expect("There is at least one attempt but no deadline")
                .duration_since(SystemTime::now())
                .unwrap_or_else(|_| Duration::from_secs(0));
            let _ =
                tokio::time::timeout(timeout, self.wait_for_any_nonce_on_l1(&not_included_on_l1))
                    .await;
        }
    }

    fn try_handle_control_action(&mut self) -> Option<SettlementTaskRunResult> {
        match self.poll_control_action() {
            Some(TaskControlAction::Cancelled) => Some(SettlementTaskRunResult::Cancelled),
            Some(TaskControlAction::ReloadAndRestart) => {
                Some(SettlementTaskRunResult::ReloadAndRestart)
            }
            None => None,
        }
    }

    fn poll_control_action(&mut self) -> Option<TaskControlAction> {
        if self.control.cancellation_token.is_cancelled() {
            return Some(TaskControlAction::Cancelled);
        }

        match self.control.admin_commands.try_recv() {
            Ok(TaskAdminCommand::ReloadAndRestart) => Some(TaskControlAction::ReloadAndRestart),
            Err(mpsc::error::TryRecvError::Empty) => None,
            Err(mpsc::error::TryRecvError::Disconnected) => {
                error!(
                    task_id = ?self.id,
                    cancelled = self.control.cancellation_token.is_cancelled(),
                    "Settlement task lost its admin command channel while still running; \
                     stopping task"
                );
                Some(TaskControlAction::Cancelled)
            }
        }
    }

    /// Saves the attempt, then submits it to L1.
    ///
    /// `nonce_guard` is the per-wallet nonce lock held since before the
    /// nonce was assigned; XREF:
    /// https://github.com/agglayer/agglayer/issues/1597. It is dropped as
    /// soon as the attempt is saved: from that point the nonce is visible to
    /// other tasks through the store, and the L1 submission does not need to
    /// block them. The retry path passes `None` because it reuses a nonce
    /// this job already owns.
    ///
    /// Returns `Some(SettlementTaskRunResult::Cancelled)` when submission was
    /// interrupted by a shutdown, so the runner stops promptly while leaving
    /// the already-saved attempt pending; returns `None` when the runner
    /// should keep going (whether submission succeeded or failed with a
    /// recorded client error).
    async fn save_attempt_to_db_and_submit_to_l1(
        &mut self,
        nonce_guard: Option<OwnedMutexGuard<()>>,
        wallet: Address,
        nonce: Nonce,
        attempt_number: SettlementAttemptNumber,
        tx: TxEnvelope,
    ) -> Option<SettlementTaskRunResult> {
        self.save_attempt_to_db(wallet, nonce, attempt_number, &tx);
        // The nonce is recorded; other same-wallet tasks may now read it.
        drop(nonce_guard);
        match self.submit_attempt_to_l1(tx).await {
            Ok(()) => None,
            Err(SubmitAttemptError::Cancelled) => Some(SettlementTaskRunResult::Cancelled),
            Err(SubmitAttemptError::Failed(error)) => {
                warn!(?error, "Failed to submit settlement attempt to L1");
                self.write_client_error_to_db(
                    attempt_number,
                    ClientError {
                        kind: ClientErrorType::Unknown,
                        message: format!("Failed to submit settlement attempt to L1: {error:?}"),
                    },
                )
                .await;
                None
            }
        }
    }

    fn all_used_nonces(&self) -> BTreeSet<(Address, Nonce)> {
        self.attempts.keys().cloned().collect()
    }

    fn all_attempt_keys(&self) -> Vec<(Address, Nonce, SettlementAttemptNumber)> {
        self.attempts
            .iter()
            .flat_map(|(&(wallet, nonce), attempts_for_nonce)| {
                attempts_for_nonce
                    .keys()
                    .copied()
                    .map(move |attempt_number| (wallet, nonce, attempt_number))
            })
            .collect()
    }

    fn next_attempt_number(&self) -> SettlementAttemptNumber {
        let next = self
            .attempts
            .values()
            .flat_map(|attempts_for_nonce| attempts_for_nonce.keys())
            .map(|attempt_number| attempt_number.0)
            .max()
            .map_or(0, |max| max.saturating_add(1));
        SettlementAttemptNumber(next)
    }

    fn is_any_attempt_pending_for_nonce(&self, wallet: Address, nonce: Nonce) -> bool {
        self.attempts
            .get(&(wallet, nonce))
            .map(|attempts_for_nonce| {
                attempts_for_nonce
                    .values()
                    .any(|attempt| attempt.result.is_none())
            })
            .unwrap_or(false)
    }

    fn settlement_attempt_number_for(
        &self,
        wallet: Address,
        nonce: Nonce,
        tx_hash: SettlementTxHash,
    ) -> Option<SettlementAttemptNumber> {
        self.attempts
            .get(&(wallet, nonce))
            .and_then(|attempts_for_nonce| {
                attempts_for_nonce
                    .iter()
                    .find(|(_, attempt)| attempt.attempt.hash == tx_hash)
                    .map(|(attempt_number, _)| *attempt_number)
            })
    }

    fn attempt_numbers_for_nonce(
        &self,
        wallet: Address,
        nonce: Nonce,
    ) -> Vec<SettlementAttemptNumber> {
        self.attempts
            .get(&(wallet, nonce))
            .map(|attempts_for_nonce| attempts_for_nonce.keys().copied().collect())
            .unwrap_or_default()
    }

    fn attempt_key_for_attempt_number(
        &self,
        attempt_number: SettlementAttemptNumber,
    ) -> Option<(Address, Nonce)> {
        self.attempts
            .iter()
            .find(|(_, attempts_for_nonce)| attempts_for_nonce.contains_key(&attempt_number))
            .map(|(key, _)| *key)
    }

    fn is_wallet_privkey_known(&self, wallet: Address) -> bool {
        self.provider.has_signer_for(&wallet)
    }

    /// Returns when the next attempt for `(wallet, nonce)` is due: the most
    /// recent attempt's submission time plus exponential backoff (the fast
    /// transient policy after an RPC `ClientError`, else the slower
    /// non-inclusion policy). Returns `now` when no attempt is tracked.
    fn next_attempt_deadline_for_nonce(&self, wallet: Address, nonce: Nonce) -> SystemTime {
        let Some(attempts_for_nonce) = self.attempts.get(&(wallet, nonce)) else {
            return SystemTime::now();
        };
        let Some((_, last_attempt)) = attempts_for_nonce.last_key_value() else {
            return SystemTime::now();
        };

        // RPC-level failures retry on the fast transient policy; an attempt still
        // pending inclusion on L1 retries on the slow non-inclusion policy.
        let policy = match last_attempt.result {
            Some(SettlementAttemptResult::ClientError(_)) => {
                &self.tx_config.retry_on_transient_failure
            }
            _ => &self.tx_config.retry_on_not_included_on_l1,
        };

        // Exponential backoff matching `retry_callback_until_success`: the n-th
        // attempt waits `initial * multiplier^(n-1)`, capped per step at max_interval.
        let retries = attempts_for_nonce.len().saturating_sub(1) as u64;
        let interval = (0..retries).fold(policy.initial_interval, |interval, _| {
            policy
                .interval_multiplier_factor
                .saturating_mul_duration(interval)
                .min(policy.max_interval)
        });

        last_attempt
            .attempt
            .submission_time
            .checked_add(interval)
            .unwrap_or(last_attempt.attempt.submission_time)
    }

    fn next_overall_deadline(&self) -> Option<SystemTime> {
        self.attempts
            .keys()
            .map(|(wallet, nonce)| self.next_attempt_deadline_for_nonce(*wallet, *nonce))
            .min()
    }

    /// Polls L1 until one of the `pending` (not-yet-included) nonces is mined.
    /// Watching only the pending set lets the run loop wake on a *new*
    /// inclusion rather than returning instantly on an already-included
    /// nonce.
    async fn wait_for_any_nonce_on_l1(&self, pending: &BTreeSet<(Address, Nonce)>) {
        // Result discarded: the caller wraps this in a timeout and re-queries under
        // `retry!` in `'start`, which escalates non-recoverable errors there.
        let _ = crate::utils::retry_callback_until_success(
            &self.tx_config.retry_on_not_included_on_l1,
            &self.control.cancellation_token,
            || async move {
                for &(wallet, nonce) in pending {
                    if crate::utils::tx_hash_on_l1_for_nonce(self.provider.as_ref(), wallet, nonce)
                        .await?
                        .is_some()
                    {
                        return Ok(());
                    }
                }
                Err(WaitForSettlementError::NotIncludedYet)
            },
            WaitForSettlementError::is_transient,
            WaitForSettlementError::needs_warning_log,
        )
        .await;
    }

    async fn tx_hash_on_l1_for_nonce(
        &self,
        wallet: Address,
        nonce: Nonce,
    ) -> Result<Option<SettlementTxHash>, RetryCallbackError<TransportError>> {
        // Test-only failpoint: pretend the nonce is not yet included on L1 so the
        // run loop keeps waiting and resubmits. Compiled out of production builds.
        #[cfg(feature = "testutils")]
        if fail::eval("settlement::tx_not_included", |_| true).unwrap_or(false) {
            return Ok(None);
        }
        crate::utils::retry_alloy_callback_until_success(
            &self.tx_config.retry_on_transient_failure,
            &self.control.cancellation_token,
            || crate::utils::tx_hash_on_l1_for_nonce(self.provider.as_ref(), wallet, nonce),
        )
        .await
    }

    async fn assign_next_nonce_for_wallet(
        &self,
        wallet: Address,
    ) -> Result<Nonce, BuildAttemptError> {
        let l1_nonce = Nonce(
            self.provider
                .get_transaction_count(wallet)
                .pending()
                .await?,
        );

        let Some(max_local_nonce) = self
            .store
            .max_settlement_nonce_for_wallet(wallet.into())
            .wrap_err_with(|| {
                format!("Failed to inspect recorded settlement attempts for wallet {wallet}")
            })
            .map_err(BuildAttemptError::NonceAssignment)?
        else {
            return Ok(l1_nonce);
        };

        let next_local_nonce = Nonce(max_local_nonce.0.checked_add(1).ok_or_else(|| {
            BuildAttemptError::NonceAssignment(eyre::eyre!(
                "Unable to assign settlement nonce for wallet {wallet}: nonce overflow"
            ))
        })?);

        Ok(l1_nonce.max(next_local_nonce))
    }

    /// Polls an L1 settlement check until it yields a terminal answer, retrying
    /// transient "not ready yet" signals with the not-included backoff. The
    /// inner check returns `Ok(None)` to report a reorg, which the caller turns
    /// into a fresh cycle.
    async fn retry_l1_check<Check, Fut>(
        &self,
        check: Check,
    ) -> Result<Option<ContractCallResult>, RetryCallbackError<TransportError>>
    where
        Check: FnMut() -> Fut,
        Fut: std::future::Future<
            Output = Result<Option<ContractCallResult>, WaitForSettlementError>,
        >,
    {
        crate::utils::retry_callback_until_success(
            &self.tx_config.retry_on_not_included_on_l1,
            &self.control.cancellation_token,
            check,
            WaitForSettlementError::is_transient,
            WaitForSettlementError::needs_warning_log,
        )
        .await
        .map_err(|error| match error {
            RetryCallbackError::Cancelled => RetryCallbackError::Cancelled,
            RetryCallbackError::Error(WaitForSettlementError::Transport(error)) => {
                RetryCallbackError::Error(error)
            }
            // Defensive only: `is_transient` marks these always-transient, so the
            // retry helper never returns them terminally. Listing them explicitly
            // keeps this match exhaustive, so adding a new variant is a compile
            // error rather than a silent fallthrough. Degrade to a transport error
            // instead of `unreachable!()` so a future drift cannot panic the
            // settlement loop.
            RetryCallbackError::Error(
                unexpected @ (WaitForSettlementError::NotSettledYet
                | WaitForSettlementError::NotIncludedYet),
            ) => {
                error!(
                    ?unexpected,
                    "transient signal surfaced as terminal settlement error"
                );
                RetryCallbackError::Error(TransportErrorKind::custom_str(
                    "settlement retry returned a transient signal as terminal",
                ))
            }
        })
    }

    /// Reads the current result of `tx_hash` on L1 without waiting for
    /// finality.
    ///
    /// A missing receipt for a nonce that still maps to `tx_hash` is indexing
    /// lag (`NotSettledYet`, retried with backoff). If the nonce no longer maps
    /// to `tx_hash`, the transaction was reorged out and we report `Ok(None)`
    /// so the caller restarts its cycle.
    async fn current_result_once(
        &self,
        wallet: Address,
        nonce: Nonce,
        tx_hash: SettlementTxHash,
    ) -> Result<Option<ContractCallResult>, WaitForSettlementError> {
        let receipt = self
            .provider
            .get_transaction_receipt(tx_hash.into())
            .await?;

        // Test-only failpoint: hide an otherwise-available receipt so the run loop
        // treats it as indexing lag and keeps polling. Compiled out of production.
        #[cfg(feature = "testutils")]
        let receipt = receipt.filter(|_| {
            !fail::eval("settlement::receipt_transiently_unavailable", |_| true).unwrap_or(false)
        });

        if let Some(receipt) = receipt {
            return match crate::utils::contract_call_result_from_receipt(&receipt) {
                Some(result) => Ok(Some(result)),
                None => Err(WaitForSettlementError::NotSettledYet),
            };
        }
        let still_mined =
            crate::utils::tx_hash_on_l1_for_nonce(self.provider.as_ref(), wallet, nonce).await?;
        if still_mined == Some(tx_hash) {
            Err(WaitForSettlementError::NotSettledYet)
        } else {
            Ok(None)
        }
    }

    async fn current_result_on_l1_for(
        &self,
        wallet: Address,
        nonce: Nonce,
        tx_hash: SettlementTxHash,
    ) -> Result<Option<ContractCallResult>, RetryCallbackError<TransportError>> {
        self.retry_l1_check(|| self.current_result_once(wallet, nonce, tx_hash))
            .await
    }

    async fn wait_for_settlement_of(
        &self,
        tx_hash: SettlementTxHash,
    ) -> Result<Option<ContractCallResult>, RetryCallbackError<TransportError>> {
        self.retry_l1_check(|| self.check_settlement_once(tx_hash))
            .await
    }

    #[tracing::instrument(
        level = "debug",
        skip_all,
        fields(
            task_id = ?self.id,
            ?tx_hash,
            settlement_policy = ?self.tx_config.settlement_policy,
        )
    )]
    async fn check_settlement_once(
        &self,
        tx_hash: SettlementTxHash,
    ) -> Result<Option<ContractCallResult>, WaitForSettlementError> {
        // Read the settlement head first so any later receipt lookup is checked
        // against a head that was already acceptable for the configured policy.
        let settlement_head_number = self.settlement_head_number().await?;
        let Some(settlement_head_number) = settlement_head_number else {
            debug!("Waiting for selected settlement head before checking settlement transaction");
            return Err(WaitForSettlementError::NotSettledYet);
        };

        let provider_tx_hash: TxHash = tx_hash.into();
        let receipt = self
            .provider
            .get_transaction_receipt(provider_tx_hash)
            .await?;
        let Some(receipt) = receipt else {
            // The caller only waits after observing this transaction on L1, so
            // a missing receipt is a reorg/drop signal.
            debug!(
                settlement_head_number,
                "Settlement transaction receipt missing after inclusion; treating as reorg or drop"
            );
            return Ok(None);
        };

        let Some(block_hash) = receipt.block_hash() else {
            debug!(
                settlement_head_number,
                "Waiting for settlement transaction receipt block hash"
            );
            return Err(WaitForSettlementError::NotSettledYet);
        };
        let Some(block_number) = receipt.block_number() else {
            debug!(
                ?block_hash,
                settlement_head_number, "Waiting for settlement transaction receipt block number"
            );
            return Err(WaitForSettlementError::NotSettledYet);
        };

        let required_head_number =
            required_settlement_head_number(block_number, self.tx_config.confirmations);
        if settlement_head_number < required_head_number {
            debug!(
                block_number,
                settlement_head_number,
                required_head_number,
                "Waiting for settlement transaction finality"
            );
            return Err(WaitForSettlementError::NotSettledYet);
        }

        let canonical_block = self
            .provider
            .get_block_by_number(BlockNumberOrTag::Number(block_number))
            .await?;
        let Some(canonical_block) = canonical_block else {
            debug!(
                block_number,
                ?block_hash,
                settlement_head_number,
                "Waiting for settlement transaction block to be available"
            );
            return Err(WaitForSettlementError::NotSettledYet);
        };

        // A receipt whose block number no longer resolves to the same canonical
        // block hash is a reorg signal, not a transient "wait longer" condition.
        let canonical_block_hash = canonical_block.header().hash;
        if canonical_block_hash != block_hash {
            debug!(
                block_number,
                ?block_hash,
                ?canonical_block_hash,
                settlement_head_number,
                "Settlement transaction receipt block hash differs from canonical block; treating \
                 as reorg"
            );
            return Ok(None);
        }

        Ok(crate::utils::contract_call_result_from_receipt(&receipt))
    }

    async fn settlement_head_number(&self) -> Result<Option<u64>, WaitForSettlementError> {
        match self.tx_config.settlement_policy {
            SettlementPolicy::LatestBlock => self
                .provider
                .get_block_number()
                .await
                .map(Some)
                .map_err(WaitForSettlementError::Transport),
            SettlementPolicy::SafeBlock => self
                .provider
                .get_block_by_number(BlockNumberOrTag::Safe)
                .await
                .map(|block| block.map(|block| block.header().number()))
                .map_err(WaitForSettlementError::Transport),
            SettlementPolicy::FinalizedBlock => self
                .provider
                .get_block_by_number(BlockNumberOrTag::Finalized)
                .await
                .map(|block| block.map(|block| block.header().number()))
                .map_err(WaitForSettlementError::Transport),
        }
    }

    fn resolve_base_gas_params(&self, estimate: &Eip1559Estimation) -> GasParams {
        let config = self.tx_config.as_ref();

        let max_fee_per_gas = clamp_u128(
            config
                .max_fee_per_gas_multiplier_factor
                .saturating_mul_u128(estimate.max_fee_per_gas),
            config.max_fee_per_gas_floor,
            config.max_fee_per_gas_ceiling,
        );
        // The priority fee must never exceed the max fee, otherwise the
        // transaction is invalid. Independent per-field floors/ceilings could
        // otherwise produce `priority > max_fee` under misconfiguration, so we
        // cap it here to keep the EIP-1559 invariant at the point of resolution.
        let max_priority_fee_per_gas = clamp_u128(
            config
                .max_priority_fee_per_gas_multiplier_factor
                .saturating_mul_u128(estimate.max_priority_fee_per_gas),
            config.max_priority_fee_per_gas_floor,
            config.max_priority_fee_per_gas_ceiling,
        )
        .min(max_fee_per_gas);

        // The job's gas limit is already resolved (at job creation).
        let gas_limit = u64::try_from(self.job.gas_limit).unwrap_or(u64::MAX);

        GasParams {
            gas_limit,
            max_fee_per_gas,
            max_priority_fee_per_gas,
        }
    }

    /// Fees of the most recent *pending* attempt for `(wallet, nonce)` — the
    /// live tx a replacement must out-bid — as `(max_fee_per_gas,
    /// max_priority_fee_per_gas)`. Attempts that errored on broadcast never
    /// reached the mempool, so they are ignored. Returns `None` when no attempt
    /// for the nonce is still pending.
    fn latest_pending_attempt_fees_for_nonce(
        &self,
        wallet: Address,
        nonce: Nonce,
    ) -> Option<(u128, u128)> {
        self.attempts
            .get(&(wallet, nonce))?
            .values()
            .rev()
            .find(|active| active.result.is_none())
            .map(|active| {
                (
                    active.attempt.max_fee_per_gas,
                    active.attempt.max_priority_fee_per_gas,
                )
            })
    }

    /// Resolves gas parameters for a retry on an existing nonce, bumping both
    /// fee fields over the previous attempt while tracking a fresh estimate.
    ///
    /// Returns `None` when the configured ceiling prevents a strict,
    /// node-acceptable bump on either field — the caller then skips submitting
    /// a duplicate/underpriced replacement.
    fn bump_gas_params(
        &self,
        previous_max_fee_per_gas: u128,
        previous_max_priority_fee_per_gas: u128,
        estimate: &Eip1559Estimation,
    ) -> Option<GasParams> {
        let config = self.tx_config.as_ref();
        let base = self.resolve_base_gas_params(estimate);

        let max_fee_per_gas = bump_fee(
            previous_max_fee_per_gas,
            base.max_fee_per_gas,
            config.max_fee_per_gas_multiplier_factor,
            config.max_fee_per_gas_floor,
            config.max_fee_per_gas_ceiling,
        )?;

        let max_priority_fee_per_gas = bump_fee(
            previous_max_priority_fee_per_gas,
            base.max_priority_fee_per_gas,
            config.max_priority_fee_per_gas_multiplier_factor,
            config.max_priority_fee_per_gas_floor,
            config.max_priority_fee_per_gas_ceiling,
        )?;

        // Preserve the EIP-1559 invariant `priority <= max_fee`. If capping
        // priority to max_fee drops it below its minimum replacement bump, no
        // valid replacement exists, so signal ceiling-reached.
        let max_priority_fee_per_gas = max_priority_fee_per_gas.min(max_fee_per_gas);
        let required_priority_min =
            MIN_REPLACEMENT_BUMP.saturating_mul_u128(previous_max_priority_fee_per_gas);
        if max_priority_fee_per_gas < required_priority_min {
            return None;
        }

        Some(GasParams {
            gas_limit: base.gas_limit,
            max_fee_per_gas,
            max_priority_fee_per_gas,
        })
    }

    /// Builds and signs an EIP-1559 settlement transaction for [`Self::job`].
    ///
    /// The gas limit is `gas.gas_limit`, resolved at job creation, so this path
    /// makes no `eth_estimateGas` call. `wallet` needs a registered signer.
    async fn build_attempt(
        &self,
        wallet: Address,
        nonce: Nonce,
        chain_id: u64,
        gas: GasParams,
    ) -> Result<TxEnvelope, TransactionBuilderError<Ethereum>> {
        let request = settlement_call_request(&self.job, wallet)
            .nonce(nonce.0)
            .gas_limit(gas.gas_limit)
            .max_fee_per_gas(gas.max_fee_per_gas)
            .max_priority_fee_per_gas(gas.max_priority_fee_per_gas)
            .with_chain_id(chain_id);

        request.build(self.provider.wallet()).await
    }

    /// Builds the next settlement attempt reusing an existing `nonce`.
    ///
    /// When a prior attempt for the nonce is still pending (a live tx in the
    /// mempool), this bumps gas over that nonce's most recent *pending* attempt
    /// (errored attempts never reached the mempool, so they are ignored) and
    /// returns `Ok(None)` if the configured ceiling prevents a strict,
    /// node-acceptable bump — the caller then keeps waiting on the
    /// already-submitted ceiling-priced attempt rather than broadcasting an
    /// underpriced replacement.
    ///
    /// When no attempt is pending (every prior attempt errored on broadcast, so
    /// there is nothing in the mempool to replace), it re-broadcasts on the
    /// same nonce at freshly-resolved base fees and always returns
    /// `Ok(Some)`. This keeps a nonce making progress even when a failed
    /// broadcast left its most recent attempt recorded at the ceiling.
    async fn build_next_attempt_with_nonce(
        &self,
        wallet: Address,
        nonce: Nonce,
    ) -> Result<Option<(SettlementAttemptNumber, TxEnvelope)>, RetryCallbackError<BuildAttemptError>>
    {
        let attempt_number = self.next_attempt_number();
        // The fees of the live tx a replacement must out-bid. Attempts that
        // errored on broadcast never reached the mempool, so they are ignored;
        // `None` means there is no live tx to replace, so we re-broadcast at
        // freshly-resolved fees rather than bumping over a prior attempt (which
        // could otherwise stall forever once those fees reach the ceiling).
        let live_tx_fees = self.latest_pending_attempt_fees_for_nonce(wallet, nonce);
        let mut retry_policy = BuildRetryPolicy::new();

        crate::utils::retry_callback_until_success(
            &self.tx_config.retry_on_transient_failure,
            &self.control.cancellation_token,
            || async {
                // The nonce is fixed; only chain id and the fee estimate are
                // read, concurrently, to keep each (retried) build to one
                // round-trip.
                let (chain_id, estimate) = tokio::try_join!(
                    self.provider.get_chain_id().into_future(),
                    self.provider.estimate_eip1559_fees().into_future(),
                )?;
                let gas = match live_tx_fees {
                    Some((previous_max_fee_per_gas, previous_max_priority_fee_per_gas)) => {
                        // Out-bid the live tx; impossible once it sits at the ceiling.
                        let Some(gas) = self.bump_gas_params(
                            previous_max_fee_per_gas,
                            previous_max_priority_fee_per_gas,
                            &estimate,
                        ) else {
                            return Ok(None);
                        };
                        gas
                    }
                    None => {
                        // Nothing live to replace: re-broadcast at base fees.
                        self.resolve_base_gas_params(&estimate)
                    }
                };
                let tx = self.build_attempt(wallet, nonce, chain_id, gas).await?;
                Ok(Some((attempt_number, tx)))
            },
            |error| retry_policy.should_retry(error),
            |_| true,
        )
        .await
    }

    /// Selects a wallet and a fresh nonce, resolves base gas parameters from
    /// the latest L1 fee estimate, and builds a signed settlement attempt.
    ///
    /// Transient L1 RPC failures are retried in place using the configured
    /// transient-failure policy; a build/sign failure is non-recoverable.
    async fn build_next_attempt_with_new_nonce(
        &self,
    ) -> Result<
        (Address, Nonce, SettlementAttemptNumber, TxEnvelope),
        RetryCallbackError<BuildAttemptError>,
    > {
        let wallet = self.provider.default_signer_address();
        let attempt_number = self.next_attempt_number();
        let mut retry_policy = BuildRetryPolicy::new();

        crate::utils::retry_callback_until_success(
            &self.tx_config.retry_on_transient_failure,
            &self.control.cancellation_token,
            || async {
                // These fetches are independent, so run them concurrently to
                // keep each (retried) build to one round-trip.
                let (nonce, chain_id, estimate) = tokio::try_join!(
                    self.assign_next_nonce_for_wallet(wallet),
                    async {
                        self.provider
                            .get_chain_id()
                            .await
                            .map_err(BuildAttemptError::from)
                    },
                    async {
                        self.provider
                            .estimate_eip1559_fees()
                            .await
                            .map_err(BuildAttemptError::from)
                    },
                )?;
                let gas = self.resolve_base_gas_params(&estimate);
                let tx = self.build_attempt(wallet, nonce, chain_id, gas).await?;
                Ok((wallet, nonce, attempt_number, tx))
            },
            |error| retry_policy.should_retry(error),
            |_| true,
        )
        .await
    }

    async fn submit_attempt_to_l1(&self, tx: TxEnvelope) -> Result<(), SubmitAttemptError> {
        // Encode the signed transaction once, consuming the envelope. This is the
        // only thing `send_tx_envelope` does with it before calling
        // `eth_sendRawTransaction`, so each retry can re-broadcast the same bytes
        // via `send_raw_transaction` without cloning or re-encoding the envelope.
        // Re-broadcasting is idempotent: the bytes carry the same sender, nonce, and
        // signature, hence the same hash, so retrying cannot submit a second
        // on-chain transaction.
        let encoded_tx = tx.encoded_2718();

        // Retry only transient network failures, mirroring `tx_hash_on_l1_for_nonce`.
        // The returned pending-transaction handle is dropped on purpose: this helper
        // must never wait for inclusion or settlement.
        let submission = crate::utils::retry_alloy_callback_until_success(
            &self.tx_config.retry_on_transient_failure,
            &self.control.cancellation_token,
            || self.provider.send_raw_transaction(&encoded_tx),
        )
        .await
        .map(drop);

        submission_outcome(submission)
    }

    async fn save_settlement_job_to_db(&self) -> eyre::Result<()> {
        self.store
            .insert_settlement_job(&self.id, &self.job)
            .wrap_err_with(|| format!("Failed to write settlement job {}", self.id))?;

        Ok(())
    }

    async fn load_settlement_job_from_db(
        store: &SettlementStore,
        id: SettlementJobId,
    ) -> eyre::Result<(SettlementJob, Option<SettlementJobResult>)> {
        let Some(job) = store
            .get_settlement_job(&id)
            .wrap_err_with(|| format!("Failed to read settlement job {id}"))?
        else {
            eyre::bail!("No settlement job found for id {id}");
        };

        let result = store.get_settlement_job_result(&id).wrap_err_with(|| {
            format!("Failed to read settlement job terminal result for id {id}")
        })?;

        Ok((job, result))
    }

    fn load_settlement_attempts_from_db(&mut self) -> eyre::Result<()> {
        let results = self.store.list_settlement_attempt_results(&self.id)?;
        let attempts = self.store.list_settlement_attempts(&self.id)?;

        self.attempts = hydrate_settlement_attempts(attempts, results, self.id)?;
        Ok(())
    }

    fn save_attempt_to_db(
        &mut self,
        wallet: Address,
        nonce: Nonce,
        attempt_number: SettlementAttemptNumber,
        tx: &TxEnvelope,
    ) {
        if let Some((existing_wallet, existing_nonce)) =
            self.attempt_key_for_attempt_number(attempt_number)
        {
            panic!(
                "Settlement attempt already tracked in memory for job {} attempt {} at \
                 {existing_wallet}/{existing_nonce}",
                self.id, attempt_number
            );
        }

        let settlement_attempt = SettlementAttempt {
            sender_wallet: wallet.into(),
            nonce,
            hash: SettlementTxHash::from(Digest::from(*tx.tx_hash())),
            submission_time: SystemTime::now(),
            max_fee_per_gas: tx.max_fee_per_gas(),
            max_priority_fee_per_gas: tx.max_priority_fee_per_gas().unwrap_or(0),
        };

        self.store
            .insert_settlement_attempt(&self.id, attempt_number.0, &settlement_attempt)
            .unwrap_or_else(|error| {
                panic!(
                    "Failed to write settlement attempt for job {} attempt {}: {error:?}",
                    self.id, attempt_number
                )
            });

        let previous_attempt = self.attempts.entry((wallet, nonce)).or_default().insert(
            attempt_number,
            ActiveSettlementAttempt {
                attempt: settlement_attempt,
                result: None,
            },
        );
        if previous_attempt.is_some() {
            panic!(
                "Settlement attempt was unexpectedly already tracked after DB write for job {} \
                 attempt {}",
                self.id, attempt_number
            );
        }
    }

    async fn write_client_error_to_db(
        &mut self,
        attempt_number: SettlementAttemptNumber,
        result: ClientError,
    ) {
        self.record_attempt_result_to_db(
            attempt_number,
            SettlementAttemptResult::ClientError(result),
        );
    }

    async fn write_nonce_revert_to_db(
        &mut self,
        wallet: Address,
        nonce: Nonce,
        attempt_number: SettlementAttemptNumber,
        result: ContractCallResult,
    ) {
        if result.outcome != ContractCallOutcome::Revert {
            panic!(
                "Settlement task {} tried to record a nonce revert for non-revert attempt {}",
                self.id, attempt_number
            );
        }

        let included_tx_hash = result.tx_hash;
        self.record_attempt_result_to_db(
            attempt_number,
            SettlementAttemptResult::ContractCall(result),
        );

        self.record_nonce_already_used_attempts_to_db(
            wallet,
            nonce,
            included_tx_hash,
            Some(attempt_number),
        );
    }

    async fn write_nonce_used_externally_to_db(
        &mut self,
        wallet: Address,
        nonce: Nonce,
        tx_hash: SettlementTxHash,
    ) {
        if self.record_nonce_already_used_attempts_to_db(wallet, nonce, tx_hash, None) == 0 {
            panic!(
                "Settlement task {} tried to record external nonce use for unknown nonce \
                 {wallet}/{nonce}",
                self.id
            );
        }
    }

    fn record_nonce_already_used_attempts_to_db(
        &mut self,
        wallet: Address,
        nonce: Nonce,
        tx_hash: SettlementTxHash,
        excluded_attempt_number: Option<SettlementAttemptNumber>,
    ) -> usize {
        let mut recorded_attempt_count = 0;

        for attempt_number in self.attempt_numbers_for_nonce(wallet, nonce) {
            if Some(attempt_number) == excluded_attempt_number {
                continue;
            }

            self.record_attempt_result_to_db(
                attempt_number,
                SettlementAttemptResult::ClientError(ClientError::nonce_already_used(
                    wallet.into(),
                    nonce,
                    tx_hash,
                )),
            );
            recorded_attempt_count += 1;
        }

        recorded_attempt_count
    }

    /// Nonces in run-loop processing order: nonces carrying a recorded
    /// successful attempt result come first.
    ///
    /// Such a result is only written after settlement-policy confirmation,
    /// so at most the terminal job-result write is missing (a crash between
    /// the completion writes). Handling that nonce first lets the loop finish
    /// the interrupted completion through its normal L1 checks before any
    /// other nonce can submit a new transaction; re-recorded completion
    /// writes are idempotent no-ops. Interrupted revert completions need no
    /// ordering: they re-derive from L1 wherever the loop starts.
    fn nonces_in_processing_order(&self) -> Vec<(Address, Nonce)> {
        let has_recorded_success = |key: &(Address, Nonce)| {
            self.attempts[key].values().any(|attempt| {
                matches!(
                    attempt.result.as_ref(),
                    Some(SettlementAttemptResult::ContractCall(result))
                        if result.outcome == ContractCallOutcome::Success
                )
            })
        };
        let (successes, others): (Vec<_>, Vec<_>) = self
            .all_used_nonces()
            .into_iter()
            .partition(has_recorded_success);
        [successes, others].concat()
    }

    async fn write_job_result_to_db(
        &mut self,
        wallet: Address,
        nonce: Nonce,
        attempt_number: SettlementAttemptNumber,
        tx_result: ContractCallResult,
    ) -> SettlementJobResult {
        self.record_attempt_result_to_db(
            attempt_number,
            SettlementAttemptResult::ContractCall(tx_result.clone()),
        );

        if tx_result.outcome == ContractCallOutcome::Success {
            self.record_nonce_already_used_attempts_to_db(
                wallet,
                nonce,
                tx_result.tx_hash,
                Some(attempt_number),
            );

            for (attempt_wallet, attempt_nonce, other_attempt_number) in self.all_attempt_keys() {
                if attempt_wallet == wallet && attempt_nonce == nonce {
                    continue;
                }

                self.record_attempt_result_to_db(
                    other_attempt_number,
                    SettlementAttemptResult::ClientError(
                        ClientError::settlement_succeeded_elsewhere(tx_result.tx_hash),
                    ),
                );
            }
        }

        let job_result = SettlementJobResult {
            wallet: wallet.into(),
            nonce,
            attempt_number,
            contract_call_result: tx_result,
        };

        self.store
            .insert_settlement_job_result(&self.id, &job_result)
            .unwrap_or_else(|error| {
                panic!(
                    "Failed to write settlement job result for job {}: {error:?}",
                    self.id
                )
            });

        job_result
    }

    fn record_attempt_result_to_db(
        &mut self,
        attempt_number: SettlementAttemptNumber,
        result: SettlementAttemptResult,
    ) {
        let Some((wallet, nonce)) = self.attempt_key_for_attempt_number(attempt_number) else {
            panic!(
                "Settlement task {} tried to record a result for unknown attempt {}",
                self.id, attempt_number
            );
        };

        let active_attempt = self
            .attempts
            .get_mut(&(wallet, nonce))
            .and_then(|attempts_for_nonce| attempts_for_nonce.get_mut(&attempt_number))
            .expect("attempt existence was checked before storage write");

        if let Some(current_result) = active_attempt.result.as_ref() {
            if current_result == &result {
                return;
            }

            if !current_result.can_be_replaced_by(&result) {
                // A "settled elsewhere" note can reach an attempt that already has a real
                // result. Keep the real result. Panicking here would repeat on every restart
                // and wedge the job forever.
                if result.is_resolved_elsewhere() {
                    warn!(
                        "Settlement task {} kept result {current_result:?} for attempt \
                         {attempt_number}, dropped conflicting write {result:?}",
                        self.id
                    );
                    return;
                }

                panic!(
                    "Settlement task {} tried to replace conflicting result for attempt \
                     {attempt_number}: {current_result:?} -> {result:?}",
                    self.id
                );
            }
        }

        self.store
            .record_settlement_attempt_result(&self.id, attempt_number.0, &result)
            .unwrap_or_else(|error| {
                panic!(
                    "Failed to write settlement attempt result for job {} attempt {}: {error:?}",
                    self.id, attempt_number
                )
            });

        active_attempt.result = Some(result);
    }
}

#[cfg(test)]
mod tests;
