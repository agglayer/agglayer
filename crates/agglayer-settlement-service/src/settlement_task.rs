use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, OnceLock},
    time::{Duration, SystemTime},
};

use agglayer_config::settlement_service::{SettlementPolicy, SettlementTransactionConfig};
use agglayer_storage::stores::{SettlementReader, SettlementWriter};
use agglayer_types::{
    ClientError, ClientErrorType, ContractCallOutcome, ContractCallResult, Digest, Nonce,
    SettlementAttempt, SettlementAttemptNumber, SettlementAttemptResult, SettlementJob,
    SettlementJobId, SettlementJobResult, SettlementTxHash,
};
use alloy::{
    consensus::{BlockHeader as _, EthereumTxEnvelope, TxEip4844Variant},
    eips::BlockNumberOrTag,
    network::{BlockResponse as _, ReceiptResponse as _},
    primitives::{Address, TxHash},
    providers::Provider,
    transports::TransportError,
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, warn};

use crate::utils::RetryCallbackError;

type TxEnvelope = EthereumTxEnvelope<TxEip4844Variant>;

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

#[derive(Debug)]
enum WaitForSettlementError {
    NotSettledYet,
    Transport(TransportError),
}

impl WaitForSettlementError {
    fn is_transient(&self) -> bool {
        match self {
            Self::NotSettledYet => true,
            Self::Transport(error) => crate::utils::is_transient_alloy_error(error),
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
    pub fn new(
        parent_cancellation_token: &CancellationToken,
        admin_commands: mpsc::Sender<TaskAdminCommand>,
        admin_command_receiver: mpsc::Receiver<TaskAdminCommand>,
    ) -> (Self, TaskControl) {
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

pub struct SettlementTask<L1Provider, SettlementStore> {
    id: SettlementJobId,
    job: SettlementJob,
    tx_config: Arc<SettlementTransactionConfig>,
    provider: Arc<L1Provider>,
    store: Arc<SettlementStore>,
    control: TaskControl,
    attempts:
        BTreeMap<(Address, Nonce), BTreeMap<SettlementAttemptNumber, ActiveSettlementAttempt>>,
}

static ID_GENERATOR: OnceLock<std::sync::Mutex<ulid::Generator>> = OnceLock::new();

impl<L1Provider: Provider + 'static, SettlementStore: SettlementReader + SettlementWriter>
    SettlementTask<L1Provider, SettlementStore>
{
    pub async fn create(
        job: SettlementJob,
        tx_config: Arc<SettlementTransactionConfig>,
        provider: Arc<L1Provider>,
        store: Arc<SettlementStore>,
        control: TaskControl,
    ) -> eyre::Result<(SettlementJobId, Self)> {
        let id = loop {
            if let Ok(id) = ID_GENERATOR
                .get_or_init(|| std::sync::Mutex::new(ulid::Generator::new()))
                .lock()
                .unwrap()
                .generate()
            {
                break SettlementJobId::from(id);
            }
            tokio::time::sleep(std::time::Duration::from_micros(100)).await;
        };
        let this = Self {
            id,
            job,
            tx_config,
            provider,
            store,
            control,
            attempts: BTreeMap::new(),
        };
        this.save_settlement_job_to_db().await?;
        Ok((id, this))
    }

    pub async fn load(
        id: SettlementJobId,
        tx_config: Arc<SettlementTransactionConfig>,
        provider: Arc<L1Provider>,
        store: Arc<SettlementStore>,
        control: TaskControl,
    ) -> eyre::Result<StoredSettlementJob<L1Provider, SettlementStore>> {
        let (job, result) = Self::load_settlement_job_from_db(id).await?;
        if let Some(result) = result {
            Ok(StoredSettlementJob::Completed(job, result))
        } else {
            let mut this = SettlementTask {
                id,
                job,
                tx_config,
                provider,
                store,
                control,
                attempts: BTreeMap::new(),
            };
            this.load_settlement_attempts_from_db().await?;
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
            let mut all_nonces_seen_on_l1 = true;
            let mut need_to_submit_attempt_with_new_nonce = true;
            'nonces: for (wallet, nonce) in self.all_used_nonces() {
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
                    let Some(tx_result) = self.current_result_on_l1_for(tx_hash).await else {
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
                    let (attempt_number, tx) = self.build_next_attempt_with_nonce(wallet, nonce);
                    self.save_attempt_to_db_and_submit_to_l1(wallet, nonce, attempt_number, tx)
                        .await;
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
                let (wallet, nonce, attempt_number, tx) = self.build_next_attempt_with_new_nonce();
                self.save_attempt_to_db_and_submit_to_l1(wallet, nonce, attempt_number, tx)
                    .await;
            }
            // We now are sure we did at least one step to make things move forward. Wait
            // for the next external event or for the next deadline.
            let timeout = self
                .next_overall_deadline()
                .expect("There is at least one attempt but no deadline")
                .duration_since(SystemTime::now())
                .unwrap_or_else(|_| Duration::from_secs(0));
            let _ = tokio::time::timeout(timeout, self.wait_for_any_nonce_on_l1()).await;
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

    async fn save_attempt_to_db_and_submit_to_l1(
        &mut self,
        wallet: Address,
        nonce: Nonce,
        attempt_number: SettlementAttemptNumber,
        tx: TxEnvelope,
    ) {
        let tx_hash = SettlementTxHash::from(Digest::from(*tx.tx_hash()));
        self.save_attempt_to_db(wallet, nonce, attempt_number, tx_hash)
            .await;
        if let Err(error) = self.submit_attempt_to_l1(tx).await {
            warn!(?error, "Failed to submit settlement attempt to L1");
            self.write_client_error_to_db(
                attempt_number,
                ClientError {
                    kind: ClientErrorType::Unknown,
                    message: format!("Failed to submit settlement attempt to L1: {error:?}"),
                },
            )
            .await;
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

    fn is_wallet_privkey_known(&self, _wallet: Address) -> bool {
        // TODO: tie with the configuration
        todo!()
    }

    fn next_attempt_deadline_for_nonce(&self, _wallet: Address, _nonce: Nonce) -> SystemTime {
        // TODO: use already-available timeout config to define the next attempt
        // deadline, considering both RPC-level retry for ClientErrors and
        // non-inclusion-level retry for the others
        todo!()
    }

    fn next_overall_deadline(&self) -> Option<SystemTime> {
        self.attempts
            .keys()
            .map(|(wallet, nonce)| self.next_attempt_deadline_for_nonce(*wallet, *nonce))
            .min()
    }

    async fn wait_for_any_nonce_on_l1(&self) {
        // TODO: wait for any nonce from our known list to be included on L1 (not
        // settled, just included) Use retry_alloy_callback_until_success as needed
        // XREF: https://github.com/agglayer/agglayer/issues/1314
        todo!()
    }

    async fn tx_hash_on_l1_for_nonce(
        &self,
        wallet: Address,
        nonce: Nonce,
    ) -> Result<Option<SettlementTxHash>, RetryCallbackError<TransportError>> {
        crate::utils::retry_alloy_callback_until_success(
            &self.tx_config.retry_on_transient_failure,
            &self.control.cancellation_token,
            || crate::utils::tx_hash_on_l1_for_nonce(self.provider.as_ref(), wallet, nonce),
        )
        .await
    }

    async fn current_result_on_l1_for(
        &self,
        _tx_hash: SettlementTxHash,
    ) -> Option<ContractCallResult> {
        // TODO: return the result on L1 if the tx_hash is already included on L1, and
        // None otherwise Use retry_alloy_callback_until_success as needed
        // XREF: https://github.com/agglayer/agglayer/issues/1382
        todo!()
    }

    async fn wait_for_settlement_of(
        &self,
        tx_hash: SettlementTxHash,
    ) -> Result<Option<ContractCallResult>, RetryCallbackError<TransportError>> {
        // Let the shared retry helper own polling; this callback only distinguishes
        // "not settled yet" from terminal reorg and RPC outcomes.
        let result = crate::utils::retry_callback_until_success(
            &self.tx_config.retry_on_not_included_on_l1,
            &self.control.cancellation_token,
            || self.check_settlement_once(tx_hash),
            WaitForSettlementError::is_transient,
        )
        .await;

        result.map_err(|error| match error {
            RetryCallbackError::Cancelled => RetryCallbackError::Cancelled,
            RetryCallbackError::Error(WaitForSettlementError::Transport(error)) => {
                RetryCallbackError::Error(error)
            }
            RetryCallbackError::Error(WaitForSettlementError::NotSettledYet) => {
                unreachable!("not-settled-yet errors are always transient")
            }
        })
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

        Ok(self.current_result_on_l1_for(tx_hash).await)
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

    fn build_next_attempt_with_nonce(
        &self,
        _wallet: Address,
        _nonce: Nonce,
    ) -> (SettlementAttemptNumber, TxEnvelope) {
        // TODO: Build the next attempt with correct gas and other params. Use https://docs.rs/alloy/latest/alloy/rpc/types/struct.TransactionRequest.html#method.build
        // XREF: https://github.com/agglayer/agglayer/issues/1319
        todo!()
    }

    fn build_next_attempt_with_new_nonce(
        &self,
    ) -> (Address, Nonce, SettlementAttemptNumber, TxEnvelope) {
        // TODO: Build the next attempt with correct gas and other params. Use https://docs.rs/alloy/latest/alloy/rpc/types/struct.TransactionRequest.html#method.build
        // XREF: https://github.com/agglayer/agglayer/issues/1318
        todo!()
    }

    async fn submit_attempt_to_l1(&self, _tx: TxEnvelope) -> eyre::Result<()> {
        // TODO: Submit attempt to L1. Use https://docs.rs/alloy/latest/alloy/providers/trait.Provider.html#method.send_tx_envelope
        // XREF: https://github.com/agglayer/agglayer/issues/1321
        todo!()
    }

    async fn save_settlement_job_to_db(&self) -> eyre::Result<()> {
        // TODO: Save the settlement job contents to DB
        // XREF: https://github.com/agglayer/agglayer/issues/1381
        todo!()
    }

    async fn load_settlement_job_from_db(
        _id: SettlementJobId,
    ) -> eyre::Result<(SettlementJob, Option<SettlementJobResult>)> {
        // TODO: Load a settlement job's contents from DB, including its
        // result if it is completed.
        // XREF: https://github.com/agglayer/agglayer/issues/1381
        todo!()
    }

    async fn load_settlement_attempts_from_db(&mut self) -> eyre::Result<()> {
        let mut results_by_attempt_number = BTreeMap::new();
        for (attempt_number, result) in self.store.list_settlement_attempt_results(&self.id)? {
            let attempt_number = SettlementAttemptNumber(attempt_number);
            if results_by_attempt_number
                .insert(attempt_number, result)
                .is_some()
            {
                eyre::bail!(
                    "Duplicate settlement attempt result {attempt_number} for job {}",
                    self.id,
                );
            }
        }

        let mut loaded_attempt_numbers = BTreeSet::new();
        let mut loaded_attempts: BTreeMap<
            (Address, Nonce),
            BTreeMap<SettlementAttemptNumber, ActiveSettlementAttempt>,
        > = BTreeMap::new();
        for (attempt_number, attempt) in self.store.list_settlement_attempts(&self.id)? {
            let attempt_number = SettlementAttemptNumber(attempt_number);
            if !loaded_attempt_numbers.insert(attempt_number) {
                eyre::bail!(
                    "Duplicate settlement attempt {attempt_number} for job {}",
                    self.id,
                );
            }

            let result = results_by_attempt_number.remove(&attempt_number);
            loaded_attempts
                .entry((attempt.sender_wallet.into_alloy(), attempt.nonce))
                .or_default()
                .insert(attempt_number, ActiveSettlementAttempt { attempt, result });
        }

        if let Some((attempt_number, _)) = results_by_attempt_number.first_key_value() {
            eyre::bail!(
                "Settlement attempt result {attempt_number} exists for job {} without a recorded \
                 settlement attempt",
                self.id,
            );
        }

        self.attempts = loaded_attempts;
        Ok(())
    }

    async fn save_attempt_to_db(
        &self,
        _wallet: Address,
        _nonce: Nonce,
        _attempt_number: SettlementAttemptNumber,
        _tx: SettlementTxHash,
    ) {
        // TODO: Save a new settlement attempt to db
        // XREF: https://github.com/agglayer/agglayer/issues/1320
        todo!()
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

    async fn write_job_result_to_db(
        &mut self,
        wallet: Address,
        nonce: Nonce,
        attempt_number: SettlementAttemptNumber,
        tx_result: ContractCallResult,
    ) -> SettlementJobResult {
        // TODO: Handle interrupted completion writes in the resumption path.
        // Attempt results are persisted before the terminal job result below; if
        // the process stops in between, loading the pending job must resume these
        // writes before considering any new settlement submission.
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
                panic!(
                    "Settlement task {} tried to replace conflicting result for attempt {}",
                    self.id, attempt_number
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
    use std::{collections::BTreeMap, sync::Arc};

    use agglayer_storage::tests::mocks::MockStateStore;
    use agglayer_types::{
        ClientError, ClientErrorType, ContractCallOutcome, Digest, SettlementAttemptResult, B256,
        U256,
    };
    use alloy::providers::ProviderBuilder;
    use tokio::sync::mpsc;

    use super::*;

    fn mk_provider() -> impl Provider + 'static {
        ProviderBuilder::new().connect_http(
            "http://127.0.0.1:0"
                .parse()
                .expect("test provider URL should parse"),
        )
    }

    fn mk_job_id(seed: u128) -> SettlementJobId {
        SettlementJobId::from(ulid::Ulid::from(seed))
    }

    fn mk_control() -> TaskControl {
        let (admin_sender, admin_receiver) = mpsc::channel(1);
        let (_handle, control) =
            TaskControlHandle::new(&CancellationToken::new(), admin_sender, admin_receiver);
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

    fn mk_contract_call_result(seed: u8, outcome: ContractCallOutcome) -> ContractCallResult {
        ContractCallResult {
            outcome,
            metadata: vec![seed, seed.wrapping_add(1)].into(),
            block_hash: B256::from([seed.wrapping_add(2); 32]),
            block_number: seed as u64,
            tx_hash: mk_tx_hash(seed.wrapping_add(3)),
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
        attempts: BTreeMap<
            (Address, Nonce),
            BTreeMap<SettlementAttemptNumber, ActiveSettlementAttempt>,
        >,
    ) -> SettlementTask<impl Provider + 'static, MockStateStore> {
        mk_task_with_id(mk_job_id(1), store, attempts)
    }

    fn mk_task_with_id(
        job_id: SettlementJobId,
        store: Arc<MockStateStore>,
        attempts: BTreeMap<
            (Address, Nonce),
            BTreeMap<SettlementAttemptNumber, ActiveSettlementAttempt>,
        >,
    ) -> SettlementTask<impl Provider + 'static, MockStateStore> {
        SettlementTask {
            id: job_id,
            job: mk_job(),
            tx_config: Arc::new(SettlementTransactionConfig::default()),
            provider: Arc::new(mk_provider()),
            store,
            control: mk_control(),
            attempts,
        }
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

    #[tokio::test]
    async fn load_settlement_attempts_from_db_hydrates_attempts_and_results() {
        let job_id = mk_job_id(1);
        let wallet = Address::repeat_byte(2);
        let other_wallet = Address::repeat_byte(3);
        let nonce = Nonce(7);
        let other_nonce = Nonce(8);
        let pending_attempt = mk_stored_attempt(1, wallet, nonce);
        let completed_attempt = mk_stored_attempt(2, wallet, nonce);
        let other_attempt = mk_stored_attempt(3, other_wallet, other_nonce);
        let completed_result = mk_client_error(4);

        let attempts_for_store = vec![
            (1, pending_attempt.clone()),
            (2, completed_attempt.clone()),
            (3, other_attempt.clone()),
        ];
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
            .await
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

        let attempts_for_other_nonce = task
            .attempts
            .get(&(other_wallet, other_nonce))
            .expect("other wallet nonce should be loaded");
        let loaded_other = attempts_for_other_nonce
            .get(&SettlementAttemptNumber(3))
            .expect("other attempt should be loaded");
        assert_eq!(loaded_other.attempt, other_attempt);
        assert_eq!(loaded_other.result, None);
    }

    #[tokio::test]
    async fn load_settlement_attempts_from_db_rejects_result_without_attempt() {
        let job_id = mk_job_id(2);
        let result = mk_client_error(5);
        let mut store = MockStateStore::new();
        let expected_job_id = job_id;
        store
            .expect_list_settlement_attempt_results()
            .once()
            .withf(move |requested_job_id| requested_job_id == &expected_job_id)
            .return_once(move |_| Ok(vec![(7, result)]));
        let expected_job_id = job_id;
        store
            .expect_list_settlement_attempts()
            .once()
            .withf(move |requested_job_id| requested_job_id == &expected_job_id)
            .return_once(|_| Ok(Vec::new()));

        let mut task = mk_task_with_id(job_id, Arc::new(store), BTreeMap::new());

        let error = task
            .load_settlement_attempts_from_db()
            .await
            .expect_err("orphaned attempt result should fail hydration");

        assert!(error
            .to_string()
            .contains("without a recorded settlement attempt"));
        assert!(task.attempts.is_empty());
    }
}
