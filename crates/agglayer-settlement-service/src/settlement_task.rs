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
mod tests {
    use std::{
        collections::BTreeMap,
        panic::{catch_unwind, AssertUnwindSafe},
        sync::{Arc, Mutex},
    };

    use agglayer_config::Multiplier;
    use agglayer_storage::{error::Error, tests::mocks::MockStateStore};
    use agglayer_types::{
        CertificateId, ClientError, ClientErrorType, ContractCallOutcome, Digest,
        SettlementAttemptResult, B256, U256,
    };
    use alloy::{
        consensus::{Signed, TxEip1559},
        network::EthereumWallet,
        node_bindings::Anvil,
        primitives::{Signature, TxKind, U64},
        providers::{mock::Asserter, ProviderBuilder},
        rpc::types::TransactionRequest,
        signers::local::PrivateKeySigner,
        transports::TransportErrorKind,
    };
    use rstest::rstest;

    use super::*;
    use crate::utils::build_provider;

    fn test_signer() -> PrivateKeySigner {
        PrivateKeySigner::from_slice(&[0x11; 32]).expect("valid test signing key")
    }

    fn mk_provider() -> impl Provider + WalletProvider + 'static {
        ProviderBuilder::new()
            .wallet(EthereumWallet::from(test_signer()))
            .connect_http(
                "http://127.0.0.1:0"
                    .parse()
                    .expect("test provider URL should parse"),
            )
    }

    fn mk_job_id(seed: u128) -> SettlementJobId {
        SettlementJobId::from(ulid::Ulid::from(seed))
    }

    fn mk_control() -> TaskControl {
        let (_handle, control) = TaskControlHandle::new(&CancellationToken::new());
        control
    }

    fn mk_job() -> SettlementJob {
        SettlementJob {
            contract_address: agglayer_types::Address::from([1; 20]),
            calldata: vec![2, 3].into(),
            eth_value: U256::from(0),
            gas_limit: 100_000,
        }
    }

    fn mk_tx_hash(seed: u8) -> SettlementTxHash {
        SettlementTxHash::new(Digest::from([seed; 32]))
    }

    fn mk_tx(hash_seed: u8) -> TxEnvelope {
        TxEnvelope::Eip1559(Signed::new_unchecked(
            TxEip1559 {
                chain_id: 1,
                nonce: 2,
                gas_limit: 100_000,
                max_fee_per_gas: 100,
                max_priority_fee_per_gas: 10,
                to: TxKind::Call(Address::from([6; 20])),
                value: U256::from(7_u64),
                input: vec![8].into(),
                access_list: Default::default(),
            },
            Signature::test_signature(),
            B256::from([hash_seed; 32]),
        ))
    }

    fn mk_contract_call_result(seed: u8, outcome: ContractCallOutcome) -> ContractCallResult {
        ContractCallResult {
            outcome,
            metadata: vec![seed, seed.wrapping_add(1)].into(),
            block_hash: B256::from([seed.wrapping_add(2); 32]),
            block_number: seed as u64,
            tx_hash: mk_tx_hash(seed.wrapping_add(3)),
        }
    }

    fn mk_job_result(seed: u8, outcome: ContractCallOutcome) -> SettlementJobResult {
        SettlementJobResult {
            wallet: Address::from([seed; 20]).into(),
            nonce: Nonce(seed as u64),
            attempt_number: SettlementAttemptNumber(seed as u64),
            contract_call_result: mk_contract_call_result(seed, outcome),
        }
    }

    fn mk_active_attempt(
        wallet: Address,
        nonce: Nonce,
        hash: SettlementTxHash,
        result: Option<SettlementAttemptResult>,
    ) -> ActiveSettlementAttempt {
        ActiveSettlementAttempt {
            attempt: SettlementAttempt {
                sender_wallet: wallet.into(),
                nonce,
                hash,
                submission_time: SystemTime::UNIX_EPOCH,
                max_fee_per_gas: 0,
                max_priority_fee_per_gas: 0,
            },
            result,
        }
    }

    fn mk_stored_attempt(seed: u8, sender_wallet: Address, nonce: Nonce) -> SettlementAttempt {
        SettlementAttempt {
            sender_wallet: sender_wallet.into(),
            nonce,
            hash: mk_tx_hash(seed),
            submission_time: SystemTime::UNIX_EPOCH + Duration::from_secs(seed.into()),
            max_fee_per_gas: 0,
            max_priority_fee_per_gas: 0,
        }
    }

    fn mk_client_error(seed: u8) -> SettlementAttemptResult {
        SettlementAttemptResult::ClientError(ClientError {
            kind: ClientErrorType::Unknown,
            message: format!("client error {seed}"),
        })
    }

    fn mk_task(
        store: Arc<MockStateStore>,
        attempts: ActiveSettlementAttempts,
    ) -> SettlementTask<impl Provider + WalletProvider + 'static, MockStateStore> {
        mk_task_with_id(SettlementJobId::from(1u128), store, attempts)
    }

    fn mk_task_with_id(
        job_id: SettlementJobId,
        store: Arc<MockStateStore>,
        attempts: ActiveSettlementAttempts,
    ) -> SettlementTask<impl Provider + WalletProvider + 'static, MockStateStore> {
        mk_task_with_id_and_provider(job_id, mk_provider(), store, attempts)
    }

    fn mk_task_with_provider<L1Provider: Provider + WalletProvider + 'static>(
        provider: L1Provider,
        store: Arc<MockStateStore>,
        attempts: ActiveSettlementAttempts,
    ) -> SettlementTask<L1Provider, MockStateStore> {
        mk_task_with_id_and_provider(SettlementJobId::from(1u128), provider, store, attempts)
    }

    fn mk_task_with_id_and_provider<L1Provider: Provider + WalletProvider + 'static>(
        job_id: SettlementJobId,
        provider: L1Provider,
        store: Arc<MockStateStore>,
        attempts: ActiveSettlementAttempts,
    ) -> SettlementTask<L1Provider, MockStateStore> {
        SettlementTask {
            id: job_id,
            job: mk_job(),
            tx_config: Arc::new(SettlementTransactionConfig::default()),
            provider: Arc::new(provider),
            store,
            wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
            control: mk_control(),
            attempts,
        }
    }

    fn mk_task_with_tx_config<P: Provider + WalletProvider + 'static>(
        provider: P,
        tx_config: SettlementTransactionConfig,
    ) -> SettlementTask<P, MockStateStore> {
        SettlementTask {
            id: mk_job_id(1),
            job: mk_job(),
            tx_config: Arc::new(tx_config),
            provider: Arc::new(provider),
            store: Arc::new(MockStateStore::new()),
            wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
            control: mk_control(),
            attempts: BTreeMap::new(),
        }
    }

    mod nonce_lock;

    #[test]
    fn next_attempt_number_starts_at_zero_and_increments_past_max() {
        let store = Arc::new(MockStateStore::new());

        let empty = mk_task(store.clone(), BTreeMap::new());
        assert_eq!(empty.next_attempt_number(), SettlementAttemptNumber(0));

        let wallet = Address::from([1; 20]);
        let nonce = Nonce(7);
        let attempts = BTreeMap::from([(
            (wallet, nonce),
            BTreeMap::from([
                (
                    SettlementAttemptNumber(2),
                    mk_active_attempt(wallet, nonce, mk_tx_hash(1), None),
                ),
                (
                    SettlementAttemptNumber(5),
                    mk_active_attempt(wallet, nonce, mk_tx_hash(2), None),
                ),
            ]),
        )]);
        let task = mk_task(store, attempts);
        assert_eq!(task.next_attempt_number(), SettlementAttemptNumber(6));
    }

    fn mk_mock_provider_with_pending_nonce(nonce: u64) -> impl Provider + WalletProvider + 'static {
        let asserter = Asserter::new();
        asserter.push_success(&U64::from(nonce));
        ProviderBuilder::new()
            .wallet(EthereumWallet::from(test_signer()))
            .connect_mocked_client(asserter)
    }

    // `create` now resolves the gas limit via `eth_estimateGas`; this answers that
    // one call so the durable-write path under test runs.
    fn mk_mock_provider_with_gas_estimate(gas: u64) -> impl Provider + WalletProvider + 'static {
        let asserter = Asserter::new();
        asserter.push_success(&U64::from(gas));
        ProviderBuilder::new()
            .wallet(EthereumWallet::from(test_signer()))
            .connect_mocked_client(asserter)
    }

    async fn load_job_from_store<L1Provider: Provider + WalletProvider + 'static>(
        _provider: L1Provider,
        store: &MockStateStore,
        job_id: SettlementJobId,
    ) -> eyre::Result<(SettlementJob, Option<SettlementJobResult>)> {
        SettlementTask::<L1Provider, MockStateStore>::load_settlement_job_from_db(store, job_id)
            .await
    }

    #[tokio::test]
    async fn save_settlement_job_to_db_inserts_job() {
        let mut store = MockStateStore::new();
        let job_id = mk_job_id(1);
        let job = mk_job();
        let expected_job = job.clone();

        store
            .expect_insert_settlement_job()
            .once()
            .withf(move |recorded_job_id, recorded_job| {
                recorded_job_id == &job_id && recorded_job == &expected_job
            })
            .return_once(|_, _| Ok(()));

        let task = mk_task(Arc::new(store), BTreeMap::new());

        task.save_settlement_job_to_db()
            .await
            .expect("settlement job should be saved");
    }

    #[tokio::test]
    async fn create_generates_settlement_job_id() {
        let mut store = MockStateStore::new();
        let job = mk_job();
        // `create` resolves the gas limit via estimateGas (mock returns 200_000).
        let mut expected_job = job.clone();
        expected_job.gas_limit = 200_000;
        let recorded_job_id = Arc::new(Mutex::new(None));
        let recorded_job_id_for_store = recorded_job_id.clone();

        store
            .expect_insert_settlement_job()
            .once()
            .withf(move |_, recorded_job| recorded_job == &expected_job)
            .return_once(move |recorded_job_id, _| {
                *recorded_job_id_for_store.lock().unwrap() = Some(*recorded_job_id);
                Ok(())
            });

        let (job_id, task) = SettlementTask::create(
            None,
            job,
            Arc::new(SettlementTransactionConfig::default()),
            Arc::new(mk_mock_provider_with_gas_estimate(200_000)),
            Arc::new(store),
            Arc::new(WalletNonceLocks::default()),
            mk_control(),
        )
        .await
        .expect("settlement task should be created");

        assert_eq!(task.id, job_id);
        assert_eq!(*recorded_job_id.lock().unwrap(), Some(job_id));
    }

    #[tokio::test]
    async fn create_records_certificate_link_before_settlement_job() {
        let mut store = MockStateStore::new();
        let certificate_id = CertificateId::new(Digest::from([7; 32]));
        let job = mk_job();
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

        let (job_id, task) = SettlementTask::create(
            Some(certificate_id),
            job,
            Arc::new(SettlementTransactionConfig::default()),
            Arc::new(mk_mock_provider_with_gas_estimate(200_000)),
            Arc::new(store),
            Arc::new(WalletNonceLocks::default()),
            mk_control(),
        )
        .await
        .expect("settlement task should be created");

        assert_eq!(task.id, job_id);
        assert_eq!(*recorded_job_id.lock().unwrap(), Some(job_id));
        assert_eq!(
            ordering.lock().unwrap().as_slice(),
            ["write_link", "write_job"]
        );
    }

    #[tokio::test]
    async fn create_fails_when_certificate_link_already_exists() {
        let mut store = MockStateStore::new();
        let certificate_id = CertificateId::new(Digest::from([8; 32]));
        let job = mk_job();

        store
            .expect_insert_certificate_settlement_job_id()
            .once()
            .withf(move |recorded_certificate_id, _| recorded_certificate_id == &certificate_id)
            .return_once(|_, _| {
                Err(Error::Unexpected(
                    "Certificate already has a settlement job id".to_string(),
                ))
            });

        store.expect_insert_settlement_job().never();

        let result = SettlementTask::create(
            Some(certificate_id),
            job,
            Arc::new(SettlementTransactionConfig::default()),
            Arc::new(mk_mock_provider_with_gas_estimate(200_000)),
            Arc::new(store),
            Arc::new(WalletNonceLocks::default()),
            mk_control(),
        )
        .await;

        let error = result
            .err()
            .expect("duplicate certificate link should fail");
        assert!(
            error
                .to_string()
                .contains("Failed to write settlement job id"),
            "{error:?}"
        );
    }

    #[tokio::test]
    async fn save_settlement_job_to_db_reports_storage_error() {
        let mut store = MockStateStore::new();
        store
            .expect_insert_settlement_job()
            .once()
            .return_once(|_, _| Err(Error::Unexpected("injected storage failure".to_string())));

        let task = mk_task(Arc::new(store), BTreeMap::new());

        let error = task
            .save_settlement_job_to_db()
            .await
            .expect_err("storage errors should be surfaced");

        assert!(error.to_string().contains("Failed to write settlement job"));
    }

    #[tokio::test]
    async fn load_settlement_job_from_db_returns_pending_job() {
        let mut store = MockStateStore::new();
        let job_id = mk_job_id(2);
        let job = mk_job();
        let expected_job = job.clone();

        store
            .expect_get_settlement_job()
            .once()
            .withf(move |recorded_job_id| recorded_job_id == &job_id)
            .return_once(move |_| Ok(Some(job)));
        store
            .expect_get_settlement_job_result()
            .once()
            .withf(move |recorded_job_id| recorded_job_id == &job_id)
            .return_once(|_| Ok(None));

        let (loaded_job, loaded_result) = load_job_from_store(mk_provider(), &store, job_id)
            .await
            .expect("settlement job should load");

        assert_eq!(loaded_job, expected_job);
        assert!(loaded_result.is_none());
    }

    #[tokio::test]
    async fn load_settlement_job_from_db_returns_completed_job() {
        let mut store = MockStateStore::new();
        let job_id = mk_job_id(3);
        let job = mk_job();
        let job_result = mk_job_result(4, ContractCallOutcome::Success);
        let expected_job = job.clone();
        let expected_job_result = job_result.clone();

        store
            .expect_get_settlement_job()
            .once()
            .return_once(move |_| Ok(Some(job)));
        store
            .expect_get_settlement_job_result()
            .once()
            .withf(move |recorded_job_id| recorded_job_id == &job_id)
            .return_once(move |_| Ok(Some(job_result)));

        let (loaded_job, loaded_result) = load_job_from_store(mk_provider(), &store, job_id)
            .await
            .expect("settlement job should load");

        assert_eq!(loaded_job, expected_job);
        assert_eq!(loaded_result, Some(expected_job_result));
    }

    #[tokio::test]
    async fn load_settlement_job_from_db_reports_missing_job() {
        let mut store = MockStateStore::new();
        let job_id = mk_job_id(4);

        store
            .expect_get_settlement_job()
            .once()
            .withf(move |recorded_job_id| recorded_job_id == &job_id)
            .return_once(|_| Ok(None));
        store.expect_get_settlement_job_result().never();

        let error = load_job_from_store(mk_provider(), &store, job_id)
            .await
            .expect_err("missing settlement job should be reported");

        assert!(error.to_string().contains("No settlement job found for id"));
    }

    #[tokio::test]
    async fn load_returns_completed_settlement_job() {
        let mut store = MockStateStore::new();
        let job_id = mk_job_id(5);
        let job = mk_job();
        let job_result = mk_job_result(6, ContractCallOutcome::Success);
        let expected_job = job.clone();
        let expected_job_result = job_result.clone();

        store
            .expect_get_settlement_job()
            .once()
            .withf(move |recorded_job_id| recorded_job_id == &job_id)
            .return_once(move |_| Ok(Some(job)));
        store
            .expect_get_settlement_job_result()
            .once()
            .withf(move |recorded_job_id| recorded_job_id == &job_id)
            .return_once(move |_| Ok(Some(job_result)));
        store.expect_list_settlement_attempts().never();
        store.expect_list_settlement_attempt_results().never();

        let loaded = SettlementTask::load(
            job_id,
            Arc::new(SettlementTransactionConfig::default()),
            Arc::new(mk_provider()),
            Arc::new(store),
            Arc::new(WalletNonceLocks::default()),
            mk_control(),
        )
        .await
        .expect("completed settlement job should load");

        match loaded {
            StoredSettlementJob::Completed(loaded_job, loaded_result) => {
                assert_eq!(loaded_job, expected_job);
                assert_eq!(loaded_result, expected_job_result);
            }
            StoredSettlementJob::Pending(_) => {
                panic!("completed settlement job should not reload as pending")
            }
        }
    }

    #[tokio::test]
    async fn assign_next_nonce_for_wallet_uses_l1_pending_nonce_when_queue_is_empty() {
        let wallet = Address::from([9; 20]);
        let expected_wallet: agglayer_types::Address = wallet.into();
        let mut store = MockStateStore::new();
        store
            .expect_max_settlement_nonce_for_wallet()
            .once()
            .withf(move |recorded_wallet| *recorded_wallet == expected_wallet)
            .return_once(|_| Ok(None));

        let task = mk_task_with_provider(
            mk_mock_provider_with_pending_nonce(5),
            Arc::new(store),
            BTreeMap::new(),
        );

        let nonce = task
            .assign_next_nonce_for_wallet(wallet)
            .await
            .expect("nonce assignment should succeed");

        assert_eq!(nonce, Nonce(5));
    }

    #[tokio::test]
    async fn assign_next_nonce_for_wallet_uses_next_local_nonce_when_queue_is_ahead_of_l1() {
        let wallet = Address::from([10; 20]);
        let expected_wallet: agglayer_types::Address = wallet.into();
        let mut store = MockStateStore::new();
        store
            .expect_max_settlement_nonce_for_wallet()
            .once()
            .withf(move |recorded_wallet| *recorded_wallet == expected_wallet)
            .return_once(|_| Ok(Some(Nonce(6))));

        let task = mk_task_with_provider(
            mk_mock_provider_with_pending_nonce(5),
            Arc::new(store),
            BTreeMap::new(),
        );

        let nonce = task
            .assign_next_nonce_for_wallet(wallet)
            .await
            .expect("nonce assignment should succeed");

        assert_eq!(nonce, Nonce(7));
    }

    #[tokio::test]
    async fn assign_next_nonce_for_wallet_uses_l1_pending_nonce_when_l1_is_ahead_of_queue() {
        let wallet = Address::from([11; 20]);
        let expected_wallet: agglayer_types::Address = wallet.into();
        let mut store = MockStateStore::new();
        store
            .expect_max_settlement_nonce_for_wallet()
            .once()
            .withf(move |recorded_wallet| *recorded_wallet == expected_wallet)
            .return_once(|_| Ok(Some(Nonce(6))));

        let task = mk_task_with_provider(
            mk_mock_provider_with_pending_nonce(9),
            Arc::new(store),
            BTreeMap::new(),
        );

        let nonce = task
            .assign_next_nonce_for_wallet(wallet)
            .await
            .expect("nonce assignment should succeed");

        assert_eq!(nonce, Nonce(9));
    }

    #[tokio::test]
    async fn save_attempt_to_db_records_attempt_in_storage_and_memory() {
        let mut store = MockStateStore::new();
        let job_id = mk_job_id(1);
        let wallet = Address::from([2; 20]);
        let expected_wallet: agglayer_types::Address = wallet.into();
        let nonce = Nonce(7);
        let attempt_number = SettlementAttemptNumber(3);
        let tx = mk_tx(4);
        let tx_hash = SettlementTxHash::from(Digest::from(*tx.tx_hash()));
        let earliest_submission_time = SystemTime::now();

        store
            .expect_insert_settlement_attempt()
            .once()
            .withf(
                move |recorded_job_id, recorded_attempt_number, recorded_attempt| {
                    recorded_job_id == &job_id
                        && *recorded_attempt_number == attempt_number.0
                        && recorded_attempt.sender_wallet == expected_wallet
                        && recorded_attempt.nonce == nonce
                        && recorded_attempt.hash == tx_hash
                        && recorded_attempt.submission_time >= earliest_submission_time
                        && recorded_attempt.max_fee_per_gas == 100
                        && recorded_attempt.max_priority_fee_per_gas == 10
                },
            )
            .return_once(|_, _, _| Ok(()));

        let mut task = mk_task(Arc::new(store), BTreeMap::new());

        task.save_attempt_to_db(wallet, nonce, attempt_number, &tx);

        let active_attempt = task
            .attempts
            .get(&(wallet, nonce))
            .and_then(|attempts_for_nonce| attempts_for_nonce.get(&attempt_number))
            .expect("attempt should be tracked in memory");

        assert_eq!(active_attempt.attempt.sender_wallet, wallet.into());
        assert_eq!(active_attempt.attempt.nonce, nonce);
        assert_eq!(active_attempt.attempt.hash, tx_hash);
        assert!(active_attempt.result.is_none());
        assert_eq!(active_attempt.attempt.max_fee_per_gas, 100);
        assert_eq!(active_attempt.attempt.max_priority_fee_per_gas, 10);
    }

    #[tokio::test]
    async fn save_attempt_to_db_does_not_track_attempt_when_storage_write_fails() {
        let mut store = MockStateStore::new();
        store
            .expect_insert_settlement_attempt()
            .once()
            .return_once(|_, _, _| Err(Error::Unexpected("injected storage failure".to_string())));

        let wallet = Address::from([2; 20]);
        let nonce = Nonce(7);
        let tx = mk_tx(4);
        let mut task = mk_task(Arc::new(store), BTreeMap::new());

        let result = catch_unwind(AssertUnwindSafe(|| {
            task.save_attempt_to_db(wallet, nonce, SettlementAttemptNumber(3), &tx);
        }));

        assert!(result.is_err());
        assert!(task.attempts.is_empty());
    }

    #[tokio::test]
    async fn save_attempt_to_db_rejects_attempt_number_already_tracked_for_other_nonce() {
        let store = MockStateStore::new();
        let existing_wallet = Address::from([1; 20]);
        let existing_nonce = Nonce(7);
        let new_wallet = Address::from([2; 20]);
        let new_nonce = Nonce(8);
        let attempt_number = SettlementAttemptNumber(3);
        let existing_hash = mk_tx_hash(9);

        let attempts = BTreeMap::from([(
            (existing_wallet, existing_nonce),
            BTreeMap::from([(
                attempt_number,
                mk_active_attempt(existing_wallet, existing_nonce, existing_hash, None),
            )]),
        )]);
        let mut task = mk_task(Arc::new(store), attempts);
        let tx = mk_tx(4);

        let result = catch_unwind(AssertUnwindSafe(|| {
            task.save_attempt_to_db(new_wallet, new_nonce, attempt_number, &tx);
        }));

        assert!(result.is_err());
        assert_eq!(task.attempts.len(), 1);
        assert_eq!(
            task.attempts[&(existing_wallet, existing_nonce)][&attempt_number]
                .attempt
                .hash,
            existing_hash
        );
        assert!(!task.attempts.contains_key(&(new_wallet, new_nonce)));
    }

    #[tokio::test]
    async fn record_attempt_result_keeps_revert_over_conflicting_write() {
        // Regression (#1607): a stored revert must survive a later "settled elsewhere"
        // write instead of panicking (which used to repeat on every restart).
        let wallet = Address::from([2; 20]);
        let nonce = Nonce(7);
        let attempt_number = SettlementAttemptNumber(3);
        let revert = SettlementAttemptResult::ContractCall(mk_contract_call_result(
            1,
            ContractCallOutcome::Revert,
        ));

        let attempts = BTreeMap::from([(
            (wallet, nonce),
            BTreeMap::from([(
                attempt_number,
                mk_active_attempt(wallet, nonce, mk_tx_hash(1), Some(revert.clone())),
            )]),
        )]);

        // The conflicting write is dropped, so the store is never called.
        let mut task = mk_task(Arc::new(MockStateStore::new()), attempts);

        task.record_attempt_result_to_db(
            attempt_number,
            SettlementAttemptResult::ClientError(ClientError::settlement_succeeded_elsewhere(
                mk_tx_hash(9),
            )),
        );

        assert_eq!(
            task.attempts[&(wallet, nonce)][&attempt_number].result,
            Some(revert)
        );
    }

    #[tokio::test]
    async fn is_wallet_privkey_known_true_for_configured_wallet() {
        let task = mk_task(Arc::new(MockStateStore::new()), BTreeMap::new());
        assert!(task.is_wallet_privkey_known(test_signer().address()));
    }

    #[tokio::test]
    async fn is_wallet_privkey_known_false_for_unknown_wallet() {
        let task = mk_task(Arc::new(MockStateStore::new()), BTreeMap::new());
        assert!(!task.is_wallet_privkey_known(Address::repeat_byte(0xAB)));
    }

    #[tokio::test]
    async fn write_job_result_records_success_and_marks_other_attempts() {
        let wallet = Address::from([1; 20]);
        let other_wallet = Address::from([2; 20]);
        let nonce = Nonce(7);
        let other_nonce = Nonce(8);
        let attempt_number = SettlementAttemptNumber(1);
        let sibling_attempt_number = SettlementAttemptNumber(2);
        let other_attempt_number = SettlementAttemptNumber(3);
        let tx_result = mk_contract_call_result(10, ContractCallOutcome::Success);
        let expected_wallet: agglayer_types::Address = wallet.into();
        let expected_tx_result = tx_result.clone();

        let mut attempts = BTreeMap::new();
        attempts.insert(
            (wallet, nonce),
            BTreeMap::from([
                (
                    attempt_number,
                    mk_active_attempt(wallet, nonce, tx_result.tx_hash, None),
                ),
                (
                    sibling_attempt_number,
                    mk_active_attempt(wallet, nonce, mk_tx_hash(20), None),
                ),
            ]),
        );
        attempts.insert(
            (other_wallet, other_nonce),
            BTreeMap::from([(
                other_attempt_number,
                mk_active_attempt(other_wallet, other_nonce, mk_tx_hash(30), None),
            )]),
        );

        let mut store = MockStateStore::new();
        store
            .expect_record_settlement_attempt_result()
            .times(3)
            .returning(|_, _, _| Ok(()));
        store
            .expect_insert_settlement_job_result()
            .once()
            .withf(move |_, result| {
                result.wallet == expected_wallet
                    && result.nonce == nonce
                    && result.attempt_number == attempt_number
                    && result.contract_call_result == expected_tx_result
            })
            .returning(|_, _| Ok(()));

        let mut task = mk_task(Arc::new(store), attempts);

        let job_result = task
            .write_job_result_to_db(wallet, nonce, attempt_number, tx_result.clone())
            .await;

        assert_eq!(job_result.contract_call_result, tx_result);
        assert!(matches!(
            task.attempts[&(wallet, nonce)][&attempt_number]
                .result
                .as_ref(),
            Some(SettlementAttemptResult::ContractCall(_))
        ));
        assert!(matches!(
            task.attempts[&(wallet, nonce)][&sibling_attempt_number]
                .result
                .as_ref(),
            Some(SettlementAttemptResult::ClientError(ClientError {
                kind: ClientErrorType::NonceAlreadyUsed,
                ..
            }))
        ));
        assert!(matches!(
            task.attempts[&(other_wallet, other_nonce)][&other_attempt_number]
                .result
                .as_ref(),
            Some(SettlementAttemptResult::ClientError(ClientError {
                kind: ClientErrorType::SettlementSucceededElsewhere,
                ..
            }))
        ));
    }

    #[tokio::test]
    async fn write_nonce_revert_replaces_previous_client_error_for_finalized_attempt() {
        let wallet = Address::from([3; 20]);
        let nonce = Nonce(9);
        let attempt_number = SettlementAttemptNumber(1);
        let sibling_attempt_number = SettlementAttemptNumber(2);
        let tx_result = mk_contract_call_result(40, ContractCallOutcome::Revert);

        let attempts = BTreeMap::from([(
            (wallet, nonce),
            BTreeMap::from([
                (
                    attempt_number,
                    mk_active_attempt(
                        wallet,
                        nonce,
                        tx_result.tx_hash,
                        Some(SettlementAttemptResult::ClientError(ClientError {
                            kind: ClientErrorType::Unknown,
                            message: "submission failed".to_string(),
                        })),
                    ),
                ),
                (
                    sibling_attempt_number,
                    mk_active_attempt(wallet, nonce, mk_tx_hash(50), None),
                ),
            ]),
        )]);

        let mut store = MockStateStore::new();
        store
            .expect_record_settlement_attempt_result()
            .times(2)
            .returning(|_, _, _| Ok(()));

        let mut task = mk_task(Arc::new(store), attempts);

        task.write_nonce_revert_to_db(wallet, nonce, attempt_number, tx_result.clone())
            .await;

        assert_eq!(
            task.attempts[&(wallet, nonce)][&attempt_number]
                .result
                .as_ref(),
            Some(&SettlementAttemptResult::ContractCall(tx_result))
        );
        assert!(matches!(
            task.attempts[&(wallet, nonce)][&sibling_attempt_number]
                .result
                .as_ref(),
            Some(SettlementAttemptResult::ClientError(ClientError {
                kind: ClientErrorType::NonceAlreadyUsed,
                ..
            }))
        ));
    }

    fn mk_rpc_block(number: u64, hash: B256) -> alloy::rpc::types::Block {
        let mut block: alloy::rpc::types::Block = Default::default();
        block.header.hash = hash;
        block.header.inner.number = number;
        block
    }

    fn mk_rpc_receipt(
        tx_hash: SettlementTxHash,
        block_hash: B256,
        block_number: u64,
    ) -> alloy::rpc::types::TransactionReceipt {
        alloy::rpc::types::TransactionReceipt {
            inner: alloy::consensus::ReceiptEnvelope::Eip1559(alloy::consensus::ReceiptWithBloom {
                receipt: alloy::consensus::Receipt {
                    status: true.into(),
                    cumulative_gas_used: 0,
                    logs: vec![],
                },
                logs_bloom: Default::default(),
            }),
            transaction_hash: tx_hash.into(),
            transaction_index: Some(0),
            block_hash: Some(block_hash),
            block_number: Some(block_number),
            gas_used: 0,
            effective_gas_price: 0,
            blob_gas_used: None,
            blob_gas_price: None,
            from: Address::from([9; 20]),
            to: None,
            contract_address: None,
        }
    }

    /// Attempts of a job whose completion was interrupted right after the
    /// winning attempt's result write: the winner carries `stored_result`,
    /// the same-nonce sibling and the other-wallet attempt are unresolved.
    fn mk_interrupted_completion_attempts(
        wallet: Address,
        nonce: Nonce,
        other_wallet: Address,
        other_nonce: Nonce,
        stored_result: &ContractCallResult,
    ) -> ActiveSettlementAttempts {
        let mut attempts = BTreeMap::new();
        attempts.insert(
            (wallet, nonce),
            BTreeMap::from([
                (
                    SettlementAttemptNumber(1),
                    mk_active_attempt(
                        wallet,
                        nonce,
                        stored_result.tx_hash,
                        Some(SettlementAttemptResult::ContractCall(stored_result.clone())),
                    ),
                ),
                (
                    SettlementAttemptNumber(2),
                    mk_active_attempt(wallet, nonce, mk_tx_hash(70), None),
                ),
            ]),
        );
        attempts.insert(
            (other_wallet, other_nonce),
            BTreeMap::from([(
                SettlementAttemptNumber(3),
                mk_active_attempt(other_wallet, other_nonce, mk_tx_hash(80), None),
            )]),
        );
        attempts
    }

    fn mk_rpc_transaction(
        tx: TxEnvelope,
        from: Address,
        block_number: u64,
    ) -> alloy::rpc::types::Transaction {
        alloy::rpc::types::Transaction {
            inner: alloy::consensus::transaction::Recovered::new_unchecked(tx, from),
            block_hash: Some(B256::from([2; 32])),
            block_number: Some(block_number),
            transaction_index: Some(0),
            effective_gas_price: Some(0),
        }
    }

    #[tokio::test]
    async fn run_finishes_interrupted_completion_before_other_nonces() {
        // The other wallet sorts before the winner: without the processing
        // order it would be handled first and consume the mocked responses.
        let wallet = Address::from([4; 20]);
        let other_wallet = Address::from([3; 20]);
        let nonce = Nonce(11);
        let other_nonce = Nonce(12);
        let attempt_number = SettlementAttemptNumber(1);
        let block_hash = B256::from([7; 32]);
        let block_number = 10;
        let stored_result = ContractCallResult {
            outcome: ContractCallOutcome::Success,
            metadata: Default::default(),
            block_hash,
            block_number,
            tx_hash: mk_tx_hash(60),
        };
        let expected_wallet: agglayer_types::Address = wallet.into();
        let expected_tx_result = stored_result.clone();

        let attempts = mk_interrupted_completion_attempts(
            wallet,
            nonce,
            other_wallet,
            other_nonce,
            &stored_result,
        );

        // The loop replays its normal success checks on the winning nonce:
        // the mined transaction for the nonce, its receipt, then the
        // settlement check (safe head, receipt again, canonical block).
        let asserter = Asserter::new();
        asserter.push_success(&mk_rpc_transaction(mk_tx(60), wallet, block_number));
        asserter.push_success(&mk_rpc_receipt(
            stored_result.tx_hash,
            block_hash,
            block_number,
        ));
        asserter.push_success(&mk_rpc_block(1_000, B256::from([1; 32])));
        asserter.push_success(&mk_rpc_receipt(
            stored_result.tx_hash,
            block_hash,
            block_number,
        ));
        asserter.push_success(&mk_rpc_block(block_number, block_hash));
        let provider = ProviderBuilder::new()
            .wallet(EthereumWallet::from(test_signer()))
            .connect_mocked_client(asserter);

        let mut store = MockStateStore::new();
        // Only the two unresolved attempts get a store write; the winner's
        // identical re-record no-ops in memory.
        store
            .expect_record_settlement_attempt_result()
            .times(2)
            .returning(|_, _, _| Ok(()));
        store
            .expect_insert_settlement_job_result()
            .once()
            .withf(move |_, result| {
                result.wallet == expected_wallet
                    && result.nonce == nonce
                    && result.attempt_number == attempt_number
                    && result.contract_call_result == expected_tx_result
            })
            .returning(|_, _| Ok(()));

        let cancellation_token = CancellationToken::new();
        let (_control_handle, control) = TaskControlHandle::new(&cancellation_token);
        let mut task = SettlementTask {
            id: mk_job_id(1),
            job: mk_job(),
            tx_config: Arc::new(SettlementTransactionConfig::default()),
            provider: Arc::new(provider),
            store: Arc::new(store),
            wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
            control,
            attempts,
        };

        let run_result = tokio::time::timeout(Duration::from_secs(30), task.run())
            .await
            .expect("the interrupted completion must finish without further L1 events");

        let SettlementTaskRunResult::Completed(job_result) = run_result else {
            panic!("expected the run to complete the job");
        };
        assert_eq!(job_result.contract_call_result, stored_result);
        assert!(matches!(
            task.attempts[&(wallet, nonce)][&SettlementAttemptNumber(2)]
                .result
                .as_ref(),
            Some(SettlementAttemptResult::ClientError(ClientError {
                kind: ClientErrorType::NonceAlreadyUsed,
                ..
            }))
        ));
        assert!(matches!(
            task.attempts[&(other_wallet, other_nonce)][&SettlementAttemptNumber(3)]
                .result
                .as_ref(),
            Some(SettlementAttemptResult::ClientError(ClientError {
                kind: ClientErrorType::SettlementSucceededElsewhere,
                ..
            }))
        ));
    }

    #[test]
    fn nonces_in_processing_order_puts_recorded_success_first() {
        let first_wallet = Address::from([3; 20]);
        let winner_wallet = Address::from([4; 20]);
        let success = mk_contract_call_result(60, ContractCallOutcome::Success);
        let revert = mk_contract_call_result(90, ContractCallOutcome::Revert);

        let attempts = BTreeMap::from([
            (
                (first_wallet, Nonce(12)),
                BTreeMap::from([(
                    SettlementAttemptNumber(3),
                    mk_active_attempt(first_wallet, Nonce(12), mk_tx_hash(80), None),
                )]),
            ),
            (
                (winner_wallet, Nonce(11)),
                BTreeMap::from([
                    (
                        SettlementAttemptNumber(1),
                        mk_active_attempt(
                            winner_wallet,
                            Nonce(11),
                            success.tx_hash,
                            Some(SettlementAttemptResult::ContractCall(success)),
                        ),
                    ),
                    (
                        SettlementAttemptNumber(2),
                        mk_active_attempt(winner_wallet, Nonce(11), mk_tx_hash(70), None),
                    ),
                ]),
            ),
        ]);
        let task = mk_task(Arc::new(MockStateStore::new()), attempts);
        assert_eq!(
            task.nonces_in_processing_order(),
            vec![(winner_wallet, Nonce(11)), (first_wallet, Nonce(12))]
        );

        // Without a recorded success (a revert or client error is not one),
        // the natural nonce order is kept.
        let attempts = BTreeMap::from([
            (
                (first_wallet, Nonce(12)),
                BTreeMap::from([(
                    SettlementAttemptNumber(3),
                    mk_active_attempt(first_wallet, Nonce(12), mk_tx_hash(80), None),
                )]),
            ),
            (
                (winner_wallet, Nonce(11)),
                BTreeMap::from([(
                    SettlementAttemptNumber(1),
                    mk_active_attempt(
                        winner_wallet,
                        Nonce(11),
                        revert.tx_hash,
                        Some(SettlementAttemptResult::ContractCall(revert)),
                    ),
                )]),
            ),
        ]);
        let task = mk_task(Arc::new(MockStateStore::new()), attempts);
        assert_eq!(
            task.nonces_in_processing_order(),
            vec![(first_wallet, Nonce(12)), (winner_wallet, Nonce(11))]
        );
    }

    #[test]
    fn required_settlement_head_number_is_inclusive_of_receipt_block() {
        // Confirmations count the receipt block itself, and saturate rather than
        // overflow.
        for (receipt_block, confirmations, required_head) in [
            (10, 0, 10),
            (10, 1, 10),
            (10, 12, 21),
            (10, usize::MAX, u64::MAX),
        ] {
            assert_eq!(
                required_settlement_head_number(receipt_block, confirmations),
                required_head
            );
        }
    }

    #[test]
    fn hydrate_settlement_attempts_groups_attempts_and_results() {
        let job_id = SettlementJobId::from(1u128);
        let wallet = Address::repeat_byte(2);
        let other_wallet = Address::repeat_byte(3);
        let nonce = Nonce(7);
        let other_nonce = Nonce(8);
        let pending_attempt = mk_stored_attempt(1, wallet, nonce);
        let completed_attempt = mk_stored_attempt(2, wallet, nonce);
        let other_attempt = mk_stored_attempt(3, other_wallet, other_nonce);
        let completed_result = mk_client_error(4);

        let hydrated_attempts = hydrate_settlement_attempts(
            vec![
                (1, pending_attempt.clone()),
                (2, completed_attempt.clone()),
                (3, other_attempt.clone()),
            ],
            vec![(2, completed_result.clone())],
            job_id,
        )
        .expect("stored attempts should hydrate");

        let attempts_for_nonce = hydrated_attempts
            .get(&(wallet, nonce))
            .expect("wallet nonce should be loaded");
        assert_eq!(attempts_for_nonce.len(), 2);
        let loaded_pending = attempts_for_nonce
            .get(&SettlementAttemptNumber(1))
            .expect("pending attempt should be loaded");
        assert_eq!(loaded_pending.attempt, pending_attempt);
        assert_eq!(loaded_pending.result, None);
        let loaded_completed = attempts_for_nonce
            .get(&SettlementAttemptNumber(2))
            .expect("completed attempt should be loaded");
        assert_eq!(loaded_completed.attempt, completed_attempt);
        assert_eq!(loaded_completed.result.as_ref(), Some(&completed_result));

        let attempts_for_other_nonce = hydrated_attempts
            .get(&(other_wallet, other_nonce))
            .expect("other wallet nonce should be loaded");
        let loaded_other = attempts_for_other_nonce
            .get(&SettlementAttemptNumber(3))
            .expect("other attempt should be loaded");
        assert_eq!(loaded_other.attempt, other_attempt);
        assert_eq!(loaded_other.result, None);
    }

    #[test]
    fn load_settlement_attempts_from_db_hydrates_attempts_and_results() {
        let job_id = SettlementJobId::from(1u128);
        let wallet = Address::repeat_byte(2);
        let nonce = Nonce(7);
        let pending_attempt = mk_stored_attempt(1, wallet, nonce);
        let completed_attempt = mk_stored_attempt(2, wallet, nonce);
        let completed_result = mk_client_error(4);

        let attempts_for_store = vec![(1, pending_attempt.clone()), (2, completed_attempt.clone())];
        let completed_result_for_store = completed_result.clone();
        let mut store = MockStateStore::new();
        let expected_job_id = job_id;
        store
            .expect_list_settlement_attempt_results()
            .once()
            .withf(move |requested_job_id| requested_job_id == &expected_job_id)
            .return_once(move |_| Ok(vec![(2, completed_result_for_store)]));
        let expected_job_id = job_id;
        store
            .expect_list_settlement_attempts()
            .once()
            .withf(move |requested_job_id| requested_job_id == &expected_job_id)
            .return_once(move |_| Ok(attempts_for_store));

        let mut task = mk_task_with_id(job_id, Arc::new(store), BTreeMap::new());

        task.load_settlement_attempts_from_db()
            .expect("stored attempts should hydrate");

        let attempts_for_nonce = task
            .attempts
            .get(&(wallet, nonce))
            .expect("wallet nonce should be loaded");
        assert_eq!(attempts_for_nonce.len(), 2);
        let loaded_pending = attempts_for_nonce
            .get(&SettlementAttemptNumber(1))
            .expect("pending attempt should be loaded");
        assert_eq!(loaded_pending.attempt, pending_attempt);
        assert_eq!(loaded_pending.result, None);
        let loaded_completed = attempts_for_nonce
            .get(&SettlementAttemptNumber(2))
            .expect("completed attempt should be loaded");
        assert_eq!(loaded_completed.attempt, completed_attempt);
        assert_eq!(loaded_completed.result.as_ref(), Some(&completed_result));
    }

    #[test]
    fn hydrate_settlement_attempts_rejects_result_without_attempt() {
        let error = hydrate_settlement_attempts(
            std::iter::empty::<(u64, SettlementAttempt)>(),
            vec![(7, mk_client_error(5))],
            SettlementJobId::from(2u128),
        )
        .err()
        .expect("orphaned attempt result should fail hydration");

        assert!(error
            .to_string()
            .contains("without a recorded settlement attempt"));
    }

    #[tokio::test]
    async fn wait_for_any_nonce_on_l1_returns_when_a_pending_nonce_is_included() {
        let anvil = Anvil::new().spawn();
        let sender = anvil.addresses()[0];
        let provider = build_provider(&anvil);
        provider
            .send_transaction(TransactionRequest::default().to(anvil.addresses()[1]))
            .await
            .expect("send transaction")
            .get_receipt()
            .await
            .expect("get receipt");

        // The zero address sorts first and is never mined, so the wait must skip it
        // and still resolve on the mined (sender, Nonce(0)).
        let pending = BTreeSet::from([(Address::ZERO, Nonce(0)), (sender, Nonce(0))]);
        let task = mk_task_with_tx_config(provider, SettlementTransactionConfig::default());

        tokio::time::timeout(
            Duration::from_secs(5),
            task.wait_for_any_nonce_on_l1(&pending),
        )
        .await
        .expect("wait should return once a pending nonce is included");
    }

    #[tokio::test]
    async fn wait_for_any_nonce_on_l1_keeps_waiting_when_no_pending_nonce_is_included() {
        let anvil = Anvil::new().spawn();
        let sender = anvil.addresses()[0];
        let provider = build_provider(&anvil);

        // No matching tx is mined. A large non-inclusion interval keeps the first
        // poll's backoff longer than the timeout. `start_paused` is unusable here:
        // Anvil uses real I/O and the paused clock would auto-advance the sleep.
        let mut tx_config = SettlementTransactionConfig::default();
        tx_config.retry_on_not_included_on_l1.initial_interval = Duration::from_secs(3600);

        let pending = BTreeSet::from([(sender, Nonce(5))]);
        let task = mk_task_with_tx_config(provider, tx_config);

        assert!(tokio::time::timeout(
            Duration::from_millis(300),
            task.wait_for_any_nonce_on_l1(&pending)
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn current_result_once_returns_result_for_mined_tx() {
        let anvil = Anvil::new().spawn();
        let sender = anvil.addresses()[0];
        let provider = build_provider(&anvil);
        let receipt = provider
            .send_transaction(TransactionRequest::default().to(anvil.addresses()[1]))
            .await
            .expect("send transaction")
            .get_receipt()
            .await
            .expect("get receipt");
        let tx_hash = SettlementTxHash::from(receipt.transaction_hash);

        let task = mk_task_with_tx_config(provider, SettlementTransactionConfig::default());
        let result = task
            .current_result_once(sender, Nonce(0), tx_hash)
            .await
            .expect("query should succeed")
            .expect("mined tx should have a result");

        assert_eq!(result.outcome, ContractCallOutcome::Success);
        assert_eq!(result.tx_hash, tx_hash);
        assert_eq!(Some(result.block_number), receipt.block_number);
        assert!(result.metadata.is_empty());
    }

    #[tokio::test]
    async fn current_result_once_reports_none_when_nonce_no_longer_maps() {
        // A receipt is missing and the nonce no longer resolves to this tx, so it
        // was reorged out (here: never mined). The lag branch -- receipt missing
        // but the nonce still maps -- needs a node that is inconsistent between
        // `eth_getTransactionReceipt` and sender+nonce lookup, which anvil is not.
        let anvil = Anvil::new().arg("--no-mining").spawn();
        let sender = anvil.addresses()[0];
        let provider = build_provider(&anvil);
        let pending = provider
            .send_transaction(TransactionRequest::default().to(anvil.addresses()[1]))
            .await
            .expect("send transaction");
        let tx_hash = SettlementTxHash::from(*pending.tx_hash());

        let task = mk_task_with_tx_config(provider, SettlementTransactionConfig::default());
        let result = task
            .current_result_once(sender, Nonce(0), tx_hash)
            .await
            .expect("query should succeed");

        assert_eq!(result, None);
    }

    #[test]
    fn submission_outcome_reports_success() {
        assert!(submission_outcome(Ok(())).is_ok());
    }

    #[test]
    fn submission_outcome_treats_cancellation_as_cancelled() {
        // A shutdown mid-retry must surface as cancellation so the caller leaves
        // the already-saved attempt pending and stops the runner, rather than
        // recording a client error or silently continuing as success.
        assert!(matches!(
            submission_outcome(Err(RetryCallbackError::Cancelled)),
            Err(SubmitAttemptError::Cancelled)
        ));
    }

    #[test]
    fn submission_outcome_reports_transport_error_as_failed() {
        let error = RetryCallbackError::Error(TransportErrorKind::custom_str("boom"));
        assert!(matches!(
            submission_outcome(Err(error)),
            Err(SubmitAttemptError::Failed(_))
        ));
    }

    #[tokio::test]
    async fn submit_attempt_to_l1_broadcasts_signed_envelope() {
        let anvil = Anvil::new().spawn();
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
        let provider = ProviderBuilder::new()
            .wallet(EthereumWallet::from(signer))
            .connect_http(anvil.endpoint_url());

        // Build and sign a transaction envelope through the provider's fillers so
        // it carries a valid nonce, gas, and chain id, then hand it off as the
        // settlement attempt to submit.
        let tx_request = TransactionRequest::default()
            .to(anvil.addresses()[1])
            .value(alloy::primitives::U256::from(1));
        let envelope = provider
            .fill(tx_request)
            .await
            .expect("filling the settlement transaction should succeed")
            .try_into_envelope()
            .expect("a wallet-filled transaction should be a signed envelope");
        let expected_tx_hash: TxHash = *envelope.tx_hash();

        let task = SettlementTask {
            id: mk_job_id(1),
            job: mk_job(),
            tx_config: Arc::new(SettlementTransactionConfig::default()),
            provider: Arc::new(provider),
            store: Arc::new(MockStateStore::new()),
            wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
            control: mk_control(),
            attempts: BTreeMap::new(),
        };

        task.submit_attempt_to_l1(envelope)
            .await
            .expect("submitting the settlement attempt should succeed");

        // The helper does not wait for inclusion, but the node must have accepted
        // the broadcast, so it should know the transaction by hash.
        let broadcast_tx = task
            .provider
            .get_transaction_by_hash(expected_tx_hash)
            .await
            .expect("querying the broadcast transaction should succeed");
        assert!(
            broadcast_tx.is_some(),
            "the node should know the broadcast settlement transaction"
        );
    }

    #[tokio::test]
    async fn submit_attempt_to_l1_skips_broadcast_when_already_cancelled() {
        let anvil = Anvil::new().spawn();
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
        let provider = ProviderBuilder::new()
            .wallet(EthereumWallet::from(signer))
            .connect_http(anvil.endpoint_url());

        let tx_request = TransactionRequest::default()
            .to(anvil.addresses()[1])
            .value(alloy::primitives::U256::from(1));
        let envelope = provider
            .fill(tx_request)
            .await
            .expect("filling the settlement transaction should succeed")
            .try_into_envelope()
            .expect("a wallet-filled transaction should be a signed envelope");
        let expected_tx_hash: TxHash = *envelope.tx_hash();

        let task = SettlementTask {
            id: mk_job_id(1),
            job: mk_job(),
            tx_config: Arc::new(SettlementTransactionConfig::default()),
            provider: Arc::new(provider),
            store: Arc::new(MockStateStore::new()),
            wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
            control: mk_control(),
            attempts: BTreeMap::new(),
        };

        // Request shutdown before submitting: the retry helper only observes the
        // token while backing off, so without an up-front guard the first
        // broadcast would still go out.
        task.control.cancellation_token.cancel();

        let result = task.submit_attempt_to_l1(envelope).await;
        assert!(matches!(result, Err(SubmitAttemptError::Cancelled)));

        // The transaction must never have been broadcast.
        let broadcast_tx = task
            .provider
            .get_transaction_by_hash(expected_tx_hash)
            .await
            .expect("querying the transaction should succeed");
        assert!(
            broadcast_tx.is_none(),
            "a cancelled submission must not broadcast the transaction"
        );
    }

    /// Tracks the given per-attempt results under one `(wallet, nonce)`, every
    /// attempt stamped at `UNIX_EPOCH` so deadlines read as seconds past epoch.
    fn attempts_with_results(
        wallet: Address,
        nonce: Nonce,
        results: impl IntoIterator<Item = Option<SettlementAttemptResult>>,
    ) -> ActiveSettlementAttempts {
        let for_nonce: BTreeMap<_, _> = results
            .into_iter()
            .enumerate()
            .map(|(i, result)| {
                let attempt = mk_active_attempt(wallet, nonce, mk_tx_hash(i as u8), result);
                (SettlementAttemptNumber(i as u64), attempt)
            })
            .collect();
        let mut attempts = ActiveSettlementAttempts::new();
        attempts.insert((wallet, nonce), for_nonce);
        attempts
    }

    // Per-attempt results → expected seconds after the last submission. The policy
    // comes from the *last* attempt's result, then backs off
    // `initial * multiplier^(attempts - 1)`, capped at max_interval.
    // Defaults: transient 10s/x1.5/120s, non-inclusion 60s/x2/600s.
    #[rstest]
    #[case::pending(vec![None], 60)]
    #[case::rpc_error(vec![Some(mk_client_error(1))], 10)]
    #[case::backoff(vec![None, None, None], 240)]
    #[case::capped(vec![None; 20], 600)]
    #[case::last_attempt_wins(vec![Some(mk_client_error(1)), None], 120)]
    fn deadline_is_last_submission_plus_policy_backoff(
        #[case] results: Vec<Option<SettlementAttemptResult>>,
        #[case] expected_secs: u64,
    ) {
        let wallet = Address::from([9; 20]);
        let nonce = Nonce(0);
        let task = mk_task(
            Arc::new(MockStateStore::new()),
            attempts_with_results(wallet, nonce, results),
        );
        assert_eq!(
            task.next_attempt_deadline_for_nonce(wallet, nonce),
            SystemTime::UNIX_EPOCH + Duration::from_secs(expected_secs),
        );
    }

    #[test]
    fn deadline_without_attempts_is_due_now() {
        let task = mk_task(
            Arc::new(MockStateStore::new()),
            ActiveSettlementAttempts::new(),
        );

        // Nothing tracked for the nonce → due immediately, never in the future.
        let deadline = task.next_attempt_deadline_for_nonce(Address::from([9; 20]), Nonce(0));
        assert!(deadline <= SystemTime::now());
    }

    #[test]
    fn resolve_base_gas_params_applies_multiplier_floor_and_ceiling() {
        let config = SettlementTransactionConfig {
            max_fee_per_gas_multiplier_factor: Multiplier::from_u64_per_1000(1000),
            max_fee_per_gas_floor: 1_000_000_000,    // 1 gwei
            max_fee_per_gas_ceiling: 50_000_000_000, // 50 gwei
            max_priority_fee_per_gas_multiplier_factor: Multiplier::from_u64_per_1000(1000),
            max_priority_fee_per_gas_floor: 2_000_000_000, // 2 gwei
            max_priority_fee_per_gas_ceiling: 50_000_000_000, // 50 gwei
            ..Default::default()
        };

        let mut task = mk_task(Arc::new(MockStateStore::new()), BTreeMap::new());
        task.tx_config = Arc::new(config);
        task.job = SettlementJob {
            gas_limit: 100_000,
            ..mk_job()
        };

        // Estimate above the fee ceiling and below the priority floor.
        let estimate = Eip1559Estimation {
            max_fee_per_gas: 80_000_000_000,       // 80 gwei -> clamps to 50 gwei
            max_priority_fee_per_gas: 100_000_000, // 0.1 gwei -> raised to 2 gwei floor
        };

        let gas = task.resolve_base_gas_params(&estimate);

        // gas_limit passes through the job's gas_limit unchanged.
        assert_eq!(gas.gas_limit, 100_000);
        assert_eq!(gas.max_fee_per_gas, 50_000_000_000);
        assert_eq!(gas.max_priority_fee_per_gas, 2_000_000_000);
    }

    #[test]
    fn resolve_base_gas_params_scales_fees_by_multiplier() {
        // Multipliers scale an estimate that lands strictly inside [floor, ceiling],
        // so this exercises the multiply path (not just clamping).
        let config = SettlementTransactionConfig {
            max_fee_per_gas_multiplier_factor: Multiplier::from_u64_per_1000(1500), // 1.5x
            max_fee_per_gas_floor: 1_000_000_000,                                   // 1 gwei
            max_fee_per_gas_ceiling: 50_000_000_000,                                // 50 gwei
            max_priority_fee_per_gas_multiplier_factor: Multiplier::from_u64_per_1000(1500), // 1.5x
            max_priority_fee_per_gas_floor: 0,
            max_priority_fee_per_gas_ceiling: 50_000_000_000, // 50 gwei
            ..Default::default()
        };

        let mut task = mk_task(Arc::new(MockStateStore::new()), BTreeMap::new());
        task.tx_config = Arc::new(config);

        let estimate = Eip1559Estimation {
            max_fee_per_gas: 10_000_000_000,         // 10 gwei * 1.5 -> 15 gwei
            max_priority_fee_per_gas: 4_000_000_000, // 4 gwei * 1.5 -> 6 gwei
        };

        let gas = task.resolve_base_gas_params(&estimate);

        assert_eq!(gas.max_fee_per_gas, 15_000_000_000);
        assert_eq!(gas.max_priority_fee_per_gas, 6_000_000_000);
    }

    #[test]
    fn resolve_base_gas_params_caps_priority_fee_at_max_fee() {
        // A priority floor above the max-fee ceiling would otherwise produce an
        // invalid `priority > max_fee`; the resolver must cap priority at max_fee.
        let config = SettlementTransactionConfig {
            max_fee_per_gas_multiplier_factor: Multiplier::from_u64_per_1000(1000),
            max_fee_per_gas_floor: 0,
            max_fee_per_gas_ceiling: 50_000_000_000, // 50 gwei
            max_priority_fee_per_gas_multiplier_factor: Multiplier::from_u64_per_1000(1000),
            max_priority_fee_per_gas_floor: 60_000_000_000, // 60 gwei (above max-fee ceiling)
            max_priority_fee_per_gas_ceiling: 100_000_000_000, // 100 gwei
            ..Default::default()
        };

        let mut task = mk_task(Arc::new(MockStateStore::new()), BTreeMap::new());
        task.tx_config = Arc::new(config);

        let estimate = Eip1559Estimation {
            max_fee_per_gas: 70_000_000_000, // -> clamps to 50 gwei ceiling
            max_priority_fee_per_gas: 1_000_000_000, // -> raised to 60 gwei floor, then capped
        };

        let gas = task.resolve_base_gas_params(&estimate);

        assert_eq!(gas.max_fee_per_gas, 50_000_000_000);
        assert_eq!(gas.max_priority_fee_per_gas, gas.max_fee_per_gas);
    }

    fn bump_config() -> SettlementTransactionConfig {
        // Wide ceilings so the bump path (not clamping) is exercised.
        SettlementTransactionConfig {
            max_fee_per_gas_multiplier_factor: Multiplier::ONE,
            max_fee_per_gas_floor: 0,
            max_fee_per_gas_ceiling: 1_000_000_000_000,
            max_priority_fee_per_gas_multiplier_factor: Multiplier::ONE,
            max_priority_fee_per_gas_floor: 0,
            max_priority_fee_per_gas_ceiling: 1_000_000_000_000,
            ..Default::default()
        }
    }

    #[test]
    fn bump_gas_params_increases_both_fields_at_least_ten_percent() {
        let mut task = mk_task(Arc::new(MockStateStore::new()), BTreeMap::new());
        task.tx_config = Arc::new(bump_config());

        // Fresh estimate below the previous attempt: the prev * 1.10 path wins.
        let estimate = Eip1559Estimation {
            max_fee_per_gas: 1_000_000_000,
            max_priority_fee_per_gas: 100_000_000,
        };
        let bumped = task
            .bump_gas_params(30_000_000_000, 1_000_000_000, &estimate)
            .expect("bump should succeed below ceiling");

        assert!(bumped.max_fee_per_gas >= 33_000_000_000); // 30 gwei * 1.10
        assert!(bumped.max_priority_fee_per_gas >= 1_100_000_000); // 1 gwei * 1.10
        assert!(bumped.max_priority_fee_per_gas <= bumped.max_fee_per_gas);
        // gas_limit comes from the base resolution (job gas_limit, default 1.0x).
        assert_eq!(bumped.gas_limit, 100_000);
    }

    #[test]
    fn bump_gas_params_returns_none_when_max_fee_ceiling_reached() {
        let config = SettlementTransactionConfig {
            max_fee_per_gas_ceiling: 30_000_000_000, // == previous max fee
            max_priority_fee_per_gas_ceiling: 1_000_000_000_000,
            ..bump_config()
        };
        let mut task = mk_task(Arc::new(MockStateStore::new()), BTreeMap::new());
        task.tx_config = Arc::new(config);

        let estimate = Eip1559Estimation {
            max_fee_per_gas: 1_000_000_000,
            max_priority_fee_per_gas: 100_000_000,
        };
        // Previous max fee already sits at the ceiling, so no strict bump exists.
        assert_eq!(
            task.bump_gas_params(30_000_000_000, 1_000_000_000, &estimate),
            None
        );
    }

    #[test]
    fn bump_gas_params_falls_back_to_base_resolution_when_no_previous_fees() {
        // Zero previous fees model the defensive fallback used when no prior
        // attempt is recorded; the bump then degrades to the base resolution of
        // the fresh estimate (required_min = 0, so the fresh value wins).
        let mut task = mk_task(Arc::new(MockStateStore::new()), BTreeMap::new());
        task.tx_config = Arc::new(bump_config());

        let estimate = Eip1559Estimation {
            max_fee_per_gas: 5_000_000_000,
            max_priority_fee_per_gas: 2_000_000_000,
        };
        let bumped = task
            .bump_gas_params(0, 0, &estimate)
            .expect("zero previous degrades to base resolution");

        // bump_config: multiplier 1.0, floor 0, wide ceiling -> base == estimate.
        assert_eq!(bumped.max_fee_per_gas, 5_000_000_000);
        assert_eq!(bumped.max_priority_fee_per_gas, 2_000_000_000);
        assert_eq!(bumped.gas_limit, 100_000);
    }

    #[test]
    fn build_retry_policy_bounds_signer_failures_and_rejects_build_bugs() {
        // `Error::message` produces an opaque `Error::Other`, mirroring how a
        // remote signer backend (e.g. GCP KMS) surfaces a signing failure.
        let signer_failure = || {
            BuildAttemptError::from(TransactionBuilderError::Signer(
                alloy::signers::Error::message("remote signer failure"),
            ))
        };

        // A signer-backend failure is retried up to the bound (to ride out a
        // transient blip), then surfaces as non-recoverable.
        let mut policy = BuildRetryPolicy::new();
        for _ in 0..BuildRetryPolicy::MAX_SIGNER_BUILD_RETRIES {
            assert!(policy.should_retry(&signer_failure()));
        }
        assert!(!policy.should_retry(&signer_failure()));

        // A structural build error is never recoverable by retrying.
        let mut policy = BuildRetryPolicy::new();
        assert!(!policy.should_retry(&BuildAttemptError::from(
            TransactionBuilderError::UnsupportedSignatureType
        )));
    }

    #[tokio::test]
    async fn build_attempt_produces_signed_eip1559_envelope() {
        use alloy::consensus::{transaction::SignerRecoverable as _, Transaction as _};

        let task = mk_task(Arc::new(MockStateStore::new()), BTreeMap::new());
        let wallet_address = test_signer().address();

        let gas = GasParams {
            gas_limit: 100_000,
            max_fee_per_gas: 30_000_000_000,
            max_priority_fee_per_gas: 1_000_000_000,
        };

        let envelope = task
            .build_attempt(wallet_address, Nonce(9), 1337, gas)
            .await
            .expect("attempt should build");

        assert!(matches!(envelope, TxEnvelope::Eip1559(_)));
        assert_eq!(envelope.nonce(), 9);
        assert_eq!(envelope.chain_id(), Some(1337));
        assert_eq!(envelope.gas_limit(), 100_000);
        assert_eq!(envelope.max_fee_per_gas(), 30_000_000_000);
        assert_eq!(envelope.max_priority_fee_per_gas(), Some(1_000_000_000));
        assert_eq!(envelope.value(), mk_job().eth_value);
        assert_eq!(envelope.input().as_ref(), mk_job().calldata.as_ref());
        assert_eq!(envelope.to(), Some(mk_job().contract_address.into_alloy()));
        assert_eq!(envelope.recover_signer().unwrap(), wallet_address);
    }

    #[tokio::test]
    async fn build_next_attempt_with_new_nonce_uses_assigned_nonce_and_default_wallet() {
        use alloy::{
            consensus::Transaction as _, node_bindings::Anvil, providers::ProviderBuilder,
            rpc::types::TransactionRequest,
        };

        let anvil = Anvil::new().spawn();
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
        let wallet_address = signer.address();
        let provider = ProviderBuilder::new()
            .wallet(EthereumWallet::from(signer))
            .connect_http(anvil.endpoint_url());

        // Bump the sender's nonce to 2 by mining two transactions.
        for _ in 0..2 {
            let tx = TransactionRequest::default()
                .to(anvil.addresses()[1])
                .value(U256::from(1));
            provider
                .send_transaction(tx)
                .await
                .unwrap()
                .get_receipt()
                .await
                .unwrap();
        }

        let expected_wallet: agglayer_types::Address = wallet_address.into();
        let mut store = MockStateStore::new();
        store
            .expect_max_settlement_nonce_for_wallet()
            .once()
            .withf(move |recorded_wallet| *recorded_wallet == expected_wallet)
            .return_once(|_| Ok(Some(Nonce(6))));

        let task = SettlementTask {
            id: mk_job_id(1),
            job: mk_job(),
            tx_config: Arc::new(SettlementTransactionConfig::default()),
            provider: Arc::new(provider),
            store: Arc::new(store),
            wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
            control: mk_control(),
            attempts: BTreeMap::new(),
        };

        let (used_wallet, nonce, attempt_number, envelope) = task
            .build_next_attempt_with_new_nonce()
            .await
            .expect("attempt should build");

        assert_eq!(used_wallet, wallet_address);
        assert_eq!(nonce, Nonce(7));
        assert_eq!(attempt_number, SettlementAttemptNumber(0));
        assert_eq!(envelope.nonce(), 7);
        assert_eq!(envelope.to(), Some(mk_job().contract_address.into_alloy()));
        assert_eq!(envelope.chain_id(), Some(anvil.chain_id()));
        // Fees are within the configured bounds (defaults: floor 0, ceiling 100 gwei).
        assert!(envelope.max_fee_per_gas() <= 100_000_000_000);
    }

    #[test]
    fn polling_signals_skip_the_retry_warning_log() {
        assert!(!WaitForSettlementError::NotIncludedYet.needs_warning_log());
        assert!(!WaitForSettlementError::NotSettledYet.needs_warning_log());
        assert!(
            WaitForSettlementError::Transport(TransportErrorKind::custom_str("connection reset"))
                .needs_warning_log()
        );
    }

    #[test]
    fn bump_fee_enforces_minimum_replacement_bump_with_default_multiplier() {
        // Default multiplier is 1.0; the helper must still bump by >= 10%.
        assert_eq!(bump_fee(100, 0, Multiplier::ONE, 0, u128::MAX), Some(110));
    }

    #[test]
    fn bump_fee_tracks_rising_fresh_estimate() {
        // A higher fresh estimate wins over prev * effective_multiplier.
        assert_eq!(
            bump_fee(100, 200, Multiplier::from_u64_per_1000(1100), 0, u128::MAX),
            Some(200)
        );
    }

    #[test]
    fn bump_fee_returns_none_when_ceiling_forbids_strict_bump() {
        // Ceiling 105 caps below prev * 1.10 = 110, so no valid replacement.
        assert_eq!(bump_fee(100, 0, Multiplier::ONE, 0, 105), None);
    }

    #[test]
    fn bump_fee_applies_floor_and_honours_larger_configured_multiplier() {
        // Floor raises the result above the minimum bump.
        assert_eq!(bump_fee(100, 0, Multiplier::ONE, 500, u128::MAX), Some(500));
        // A configured multiplier larger than the 10% minimum is used as-is.
        assert_eq!(
            bump_fee(100, 0, Multiplier::from_u64_per_1000(2000), 0, u128::MAX),
            Some(200)
        );
    }

    #[tokio::test]
    async fn build_next_attempt_with_nonce_bumps_fees_over_previous_attempt() {
        use alloy::consensus::Transaction as _;

        let anvil = Anvil::new().spawn();
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
        let wallet_address = signer.address();
        let provider = ProviderBuilder::new()
            .wallet(EthereumWallet::from(signer))
            .connect_http(anvil.endpoint_url());

        // A previous attempt for nonce 4 with known fees to bump from.
        let nonce = Nonce(4);
        let previous = SettlementAttempt {
            sender_wallet: wallet_address.into(),
            nonce,
            hash: mk_tx_hash(1),
            submission_time: SystemTime::UNIX_EPOCH,
            max_fee_per_gas: 30_000_000_000,
            max_priority_fee_per_gas: 1_000_000_000,
        };
        let attempts = BTreeMap::from([(
            (wallet_address, nonce),
            BTreeMap::from([(
                SettlementAttemptNumber(0),
                ActiveSettlementAttempt {
                    attempt: previous,
                    result: None,
                },
            )]),
        )]);

        let task = SettlementTask {
            id: mk_job_id(1),
            job: mk_job(),
            tx_config: Arc::new(SettlementTransactionConfig::default()),
            provider: Arc::new(provider),
            store: Arc::new(MockStateStore::new()),
            wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
            control: mk_control(),
            attempts,
        };

        let (attempt_number, envelope) = task
            .build_next_attempt_with_nonce(wallet_address, nonce)
            .await
            .expect("build should not fail")
            .expect("bump should produce an attempt below the ceiling");

        assert_eq!(attempt_number, SettlementAttemptNumber(1));
        assert_eq!(envelope.nonce(), 4);
        assert_eq!(envelope.chain_id(), Some(anvil.chain_id()));
        // Strictly bumped by >= 10% over the previous attempt on both fields.
        assert!(envelope.max_fee_per_gas() >= 33_000_000_000);
        assert!(envelope.max_priority_fee_per_gas().unwrap() >= 1_100_000_000);
        assert!(envelope.max_priority_fee_per_gas().unwrap() <= envelope.max_fee_per_gas());
    }

    #[tokio::test]
    async fn build_next_attempt_with_nonce_returns_none_at_ceiling() {
        let anvil = Anvil::new().spawn();
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
        let wallet_address = signer.address();
        let provider = ProviderBuilder::new()
            .wallet(EthereumWallet::from(signer))
            .connect_http(anvil.endpoint_url());

        let nonce = Nonce(4);
        let previous = SettlementAttempt {
            sender_wallet: wallet_address.into(),
            nonce,
            hash: mk_tx_hash(1),
            submission_time: SystemTime::UNIX_EPOCH,
            max_fee_per_gas: 30_000_000_000,
            max_priority_fee_per_gas: 1_000_000_000,
        };
        let attempts = BTreeMap::from([(
            (wallet_address, nonce),
            BTreeMap::from([(
                SettlementAttemptNumber(0),
                ActiveSettlementAttempt {
                    attempt: previous,
                    result: None,
                },
            )]),
        )]);

        // The sole attempt is still pending (`result: None` above): a live tx
        // sits in the mempool, and the ceilings are pinned to its fees, so no
        // strict bump is possible and waiting (not re-broadcasting) is correct.
        let config = SettlementTransactionConfig {
            max_fee_per_gas_ceiling: 30_000_000_000,
            max_priority_fee_per_gas_ceiling: 1_000_000_000,
            ..SettlementTransactionConfig::default()
        };

        let task = SettlementTask {
            id: mk_job_id(1),
            job: mk_job(),
            tx_config: Arc::new(config),
            provider: Arc::new(provider),
            store: Arc::new(MockStateStore::new()),
            wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
            control: mk_control(),
            attempts,
        };

        let result = task
            .build_next_attempt_with_nonce(wallet_address, nonce)
            .await
            .expect("build should not fail");
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn build_next_attempt_with_nonce_rebroadcasts_errored_attempt_at_ceiling() {
        use alloy::consensus::Transaction as _;

        let anvil = Anvil::new().spawn();
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
        let wallet_address = signer.address();
        let provider = ProviderBuilder::new()
            .wallet(EthereumWallet::from(signer))
            .connect_http(anvil.endpoint_url());

        // The only attempt for this nonce errored on broadcast (no live tx in the
        // mempool) and sits at the fee ceiling. There is nothing to replace, so
        // the task must re-broadcast on the same nonce at freshly-resolved fees
        // rather than stall by bumping past the ceiling.
        let nonce = Nonce(4);
        let previous = SettlementAttempt {
            sender_wallet: wallet_address.into(),
            nonce,
            hash: mk_tx_hash(1),
            submission_time: SystemTime::UNIX_EPOCH,
            max_fee_per_gas: 30_000_000_000,
            max_priority_fee_per_gas: 1_000_000_000,
        };
        let attempts = BTreeMap::from([(
            (wallet_address, nonce),
            BTreeMap::from([(
                SettlementAttemptNumber(0),
                ActiveSettlementAttempt {
                    attempt: previous,
                    result: Some(mk_client_error(7)),
                },
            )]),
        )]);

        // Ceilings pinned to the previous fees: a strict bump is impossible.
        let config = SettlementTransactionConfig {
            max_fee_per_gas_ceiling: 30_000_000_000,
            max_priority_fee_per_gas_ceiling: 1_000_000_000,
            ..SettlementTransactionConfig::default()
        };

        let task = SettlementTask {
            id: mk_job_id(1),
            job: mk_job(),
            tx_config: Arc::new(config),
            provider: Arc::new(provider),
            store: Arc::new(MockStateStore::new()),
            wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
            control: mk_control(),
            attempts,
        };

        let (attempt_number, envelope) = task
            .build_next_attempt_with_nonce(wallet_address, nonce)
            .await
            .expect("build should not fail")
            .expect("an errored attempt has no live tx to replace, so it must re-broadcast");

        assert_eq!(attempt_number, SettlementAttemptNumber(1));
        assert_eq!(envelope.nonce(), 4);
        assert!(matches!(envelope, TxEnvelope::Eip1559(_)));
        // Re-broadcast at base fees, within the configured ceiling; no strict bump.
        assert!(envelope.max_fee_per_gas() <= 30_000_000_000);
        assert!(envelope.max_priority_fee_per_gas().unwrap() <= envelope.max_fee_per_gas());
    }

    #[tokio::test]
    async fn build_next_attempt_with_nonce_bumps_over_live_tx_ignoring_errored_ceiling_attempt() {
        use alloy::consensus::Transaction as _;

        let anvil = Anvil::new().spawn();
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
        let wallet_address = signer.address();
        let provider = ProviderBuilder::new()
            .wallet(EthereumWallet::from(signer))
            .connect_http(anvil.endpoint_url());

        let nonce = Nonce(4);
        // Live tx: a pending attempt well below the ceiling.
        let pending = SettlementAttempt {
            sender_wallet: wallet_address.into(),
            nonce,
            hash: mk_tx_hash(1),
            submission_time: SystemTime::UNIX_EPOCH,
            max_fee_per_gas: 10_000_000_000,
            max_priority_fee_per_gas: 1_000_000_000,
        };
        // A newer attempt that errored on broadcast at the ceiling (no live tx).
        let errored = SettlementAttempt {
            sender_wallet: wallet_address.into(),
            nonce,
            hash: mk_tx_hash(2),
            submission_time: SystemTime::UNIX_EPOCH,
            max_fee_per_gas: 30_000_000_000,
            max_priority_fee_per_gas: 1_000_000_000,
        };
        let attempts = BTreeMap::from([(
            (wallet_address, nonce),
            BTreeMap::from([
                (
                    SettlementAttemptNumber(0),
                    ActiveSettlementAttempt {
                        attempt: pending,
                        result: None,
                    },
                ),
                (
                    SettlementAttemptNumber(1),
                    ActiveSettlementAttempt {
                        attempt: errored,
                        result: Some(mk_client_error(9)),
                    },
                ),
            ]),
        )]);

        // Ceiling at the errored attempt's fees: bumping over *it* is impossible,
        // but the live pending tx (10 gwei) can still be out-bid below the ceiling.
        let config = SettlementTransactionConfig {
            max_fee_per_gas_ceiling: 30_000_000_000,
            max_priority_fee_per_gas_ceiling: 30_000_000_000,
            ..SettlementTransactionConfig::default()
        };

        let task = SettlementTask {
            id: mk_job_id(1),
            job: mk_job(),
            tx_config: Arc::new(config),
            provider: Arc::new(provider),
            store: Arc::new(MockStateStore::new()),
            wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
            control: mk_control(),
            attempts,
        };

        let (attempt_number, envelope) = task
            .build_next_attempt_with_nonce(wallet_address, nonce)
            .await
            .expect("build should not fail")
            .expect("a valid replacement over the live pending tx is possible below the ceiling");

        assert_eq!(attempt_number, SettlementAttemptNumber(2));
        assert_eq!(envelope.nonce(), 4);
        // Bumped >= 10% over the *pending* tx (10 gwei), not the errored 30 gwei one.
        assert!(envelope.max_fee_per_gas() >= 11_000_000_000);
        assert!(envelope.max_fee_per_gas() <= 30_000_000_000);
        assert!(envelope.max_priority_fee_per_gas().unwrap() <= envelope.max_fee_per_gas());
    }

    #[test]
    fn latest_pending_attempt_fees_for_nonce_ignores_errored_and_unknown() {
        let wallet = Address::from([7; 20]);
        let nonce = Nonce(3);
        let errored_only_nonce = Nonce(5);

        // A live pending tx, plus a higher-numbered attempt that errored on
        // broadcast (no live tx) at higher fees.
        let mut pending = mk_active_attempt(wallet, nonce, mk_tx_hash(1), None);
        pending.attempt.max_fee_per_gas = 10_000_000_000;
        pending.attempt.max_priority_fee_per_gas = 1_000_000_000;
        let mut errored_newer =
            mk_active_attempt(wallet, nonce, mk_tx_hash(2), Some(mk_client_error(9)));
        errored_newer.attempt.max_fee_per_gas = 99_000_000_000;
        errored_newer.attempt.max_priority_fee_per_gas = 9_000_000_000;

        // A separate nonce whose only attempt errored.
        let errored_only = mk_active_attempt(
            wallet,
            errored_only_nonce,
            mk_tx_hash(3),
            Some(mk_client_error(2)),
        );

        let attempts = BTreeMap::from([
            (
                (wallet, nonce),
                BTreeMap::from([
                    (SettlementAttemptNumber(4), errored_newer),
                    (SettlementAttemptNumber(1), pending),
                ]),
            ),
            (
                (wallet, errored_only_nonce),
                BTreeMap::from([(SettlementAttemptNumber(2), errored_only)]),
            ),
        ]);
        let task = mk_task(Arc::new(MockStateStore::new()), attempts);

        // The higher-numbered errored attempt is ignored; the live pending tx wins.
        assert_eq!(
            task.latest_pending_attempt_fees_for_nonce(wallet, nonce),
            Some((10_000_000_000, 1_000_000_000))
        );
        // A nonce whose only attempt errored has no live tx.
        assert_eq!(
            task.latest_pending_attempt_fees_for_nonce(wallet, errored_only_nonce),
            None
        );
        // Unknown nonce -> None.
        assert_eq!(
            task.latest_pending_attempt_fees_for_nonce(wallet, Nonce(999)),
            None
        );
    }
}
