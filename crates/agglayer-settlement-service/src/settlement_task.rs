use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, OnceLock},
    time::{Duration, SystemTime},
};

use agglayer_config::{settlement_service::SettlementTransactionConfig, Multiplier};
use agglayer_types::{Digest, SettlementTxHash};
use alloy::{
    consensus::{EthereumTxEnvelope, TxEip4844Variant},
    primitives::{Address, BlockHash, Bytes, U128, U256},
};
use tokio::sync::mpsc;
use tracing::warn;
use ulid::Ulid;

type TxEnvelope = EthereumTxEnvelope<TxEip4844Variant>;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SettlementAttemptNumber(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, derive_more::Display)]
pub struct Nonce(pub u64);

impl Nonce {
    pub fn previous(&self) -> Option<Nonce> {
        self.0.checked_sub(1).map(Nonce)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettlementJob {
    contract_address: Address,
    calldata: Bytes,
    eth_value: U256,

    num_confirmations: u32,
    gas_limit: U128,
    max_fee_per_gas_ceiling: U128,
    max_fee_per_gas_floor: U128,
    max_fee_per_gas_multiplier: Multiplier,
    max_priority_fee_per_gas_ceiling: U128,
    max_priority_fee_per_gas_floor: U128,
    max_priority_fee_per_gas_multiplier: Multiplier,

    settlement_config: Arc<SettlementTransactionConfig>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SettlementJobResult {
    ClientError(ClientError),
    ContractCall(ContractCallResult),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientError {
    pub kind: ClientErrorType,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClientErrorType {
    Unknown,
    // Not needed for now, might re-add later: Transient,
    // Not needed for now, might re-add later: Permanent,
    NonceAlreadyUsed, // TODO: Set it only when the other tx that used the nonce is finalized
}

impl ClientError {
    pub fn nonce_already_used(address: Address, nonce: Nonce, tx_hash: SettlementTxHash) -> Self {
        Self {
            kind: ClientErrorType::NonceAlreadyUsed,
            message: format!(
                "Nonce already used: for {address}/{nonce}, the settled tx is {tx_hash}"
            ),
        }
    }

    pub fn timeout_waiting_for_inclusion() -> Self {
        Self {
            kind: ClientErrorType::Unknown,
            message: "Timeout waiting for inclusion on L1".to_string(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractCallResult {
    pub outcome: ContractCallOutcome,
    pub metadata: Bytes,
    pub block_hash: BlockHash,
    pub block_number: u64,
    pub tx_hash: SettlementTxHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractCallOutcome {
    Success,
    Revert,
}

pub enum StoredSettlementJob {
    Pending(SettlementTask),
    Completed(SettlementJob, SettlementJobResult),
}

pub enum TaskAdminCommand {
    Abort,
    ReloadAndRestart,
}

pub struct SettlementAttempt {
    pub sender_wallet: Address,
    pub nonce: Nonce,
    pub max_fee_per_gas: u128,
    pub max_priority_fee_per_gas: u128,
    pub hash: SettlementTxHash,
    pub submission_time: SystemTime,
    pub result: Option<SettlementJobResult>,
}
pub struct SettlementTask {
    id: Ulid,
    job: SettlementJob,
    admin_commands: mpsc::Receiver<TaskAdminCommand>,
    attempts: BTreeMap<(Address, Nonce), BTreeMap<SettlementAttemptNumber, SettlementAttempt>>,
}

static ID_GENERATOR: OnceLock<std::sync::Mutex<ulid::Generator>> = OnceLock::new();

impl SettlementTask {
    pub async fn create(
        job: SettlementJob,
        admin_commands: mpsc::Receiver<TaskAdminCommand>,
    ) -> eyre::Result<(Ulid, Self)> {
        let id = loop {
            if let Ok(id) = ID_GENERATOR
                .get_or_init(|| std::sync::Mutex::new(ulid::Generator::new()))
                .lock()
                .unwrap()
                .generate()
            {
                break id;
            }
            tokio::time::sleep(std::time::Duration::from_micros(100)).await;
        };
        let this = Self {
            id,
            job,
            admin_commands,
            attempts: BTreeMap::new(),
        };
        this.save_settlement_job_to_db().await?;
        Ok((id, this))
    }

    pub async fn load(
        id: Ulid,
        admin_commands: mpsc::Receiver<TaskAdminCommand>,
    ) -> eyre::Result<StoredSettlementJob> {
        let (job, result) = Self::load_settlement_job_from_db(id).await?;
        if let Some(result) = result {
            Ok(StoredSettlementJob::Completed(job, result))
        } else {
            let mut this = SettlementTask {
                id,
                job,
                admin_commands,
                attempts: BTreeMap::new(),
            };
            this.load_settlement_attempts_from_db().await?;
            Ok(StoredSettlementJob::Pending(this))
        }
    }

    pub async fn run(&mut self) -> SettlementJobResult {
        'start: loop {
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
                if let Some(tx_hash) = self.tx_hash_on_l1_for_nonce(wallet, nonce).await {
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
                    let Some(settled_result) = self.wait_for_settlement_of(tx_hash).await else {
                        continue 'start; // reorg
                    };
                    if settled_result != tx_result {
                        continue 'start; // reorg
                    }
                    self.write_job_successful_to_db(
                        wallet,
                        nonce,
                        attempt_number,
                        tx_result.clone(),
                    )
                    .await;
                    return SettlementJobResult::ContractCall(tx_result);
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
                            if self
                                .tx_hash_on_l1_for_nonce(wallet, previous_nonce)
                                .await
                                .is_none()
                            {
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
                // We can wait for finalization without submiting a new attempt.
                for (wallet, nonce) in self.all_used_nonces() {
                    if let Some(tx_hash) = nonces_used_externally.remove(&(wallet, nonce)) {
                        if self.wait_for_settlement_of(tx_hash).await.is_none() {
                            continue 'start; // reorg
                        }
                        self.write_nonce_used_externally_to_db(wallet, nonce, tx_hash)
                            .await;
                    } else if let Some((attempt_number, tx_hash, result)) =
                        reverts.remove(&(wallet, nonce))
                    {
                        let Some(settled_result) = self.wait_for_settlement_of(tx_hash).await
                        else {
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
                let earliest_revert_result = reverts
                    .values()
                    .map(|(_, _, result)| result)
                    .min_by_key(|result| result.block_number)
                    .unwrap(); // No panic: we checked `!reverts.is_empty()` above.
                self.write_job_revert_to_db(earliest_revert_result).await;
                return SettlementJobResult::ContractCall(earliest_revert_result.clone());
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

    async fn save_attempt_to_db_and_submit_to_l1(
        &self,
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
                    .find(|(_, attempt)| attempt.hash == tx_hash)
                    .map(|(attempt_number, _)| *attempt_number)
            })
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
        // settled, just included) Use retry_callback_until_success as needed
        // XREF: https://github.com/agglayer/agglayer/issues/1314
        todo!()
    }

    async fn tx_hash_on_l1_for_nonce(
        &self,
        _wallet: Address,
        _nonce: Nonce,
    ) -> Option<SettlementTxHash> {
        // TODO: return tx hash if the nonce is already included on L1, and None
        // otherwise Use retry_callback_until_success as needed
        todo!()
    }

    async fn current_result_on_l1_for(
        &self,
        _tx_hash: SettlementTxHash,
    ) -> Option<ContractCallResult> {
        // TODO: return the result on L1 if the tx_hash is already included on L1, and
        // None otherwise Use retry_callback_until_success as needed
        // XREF: https://github.com/agglayer/agglayer/issues/1382
        todo!()
    }

    async fn wait_for_settlement_of(
        &self,
        _tx_hash: SettlementTxHash,
    ) -> Option<ContractCallResult> {
        // TODO: Wait for the settlement of tx_hash, and then return its result on L1.
        // If a reorg is detected during the waiting, return None.
        // XREF: https://github.com/agglayer/agglayer/issues/1316
        todo!()
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
        // TODO: Save the settlement job contents to L1
        // XREF: https://github.com/agglayer/agglayer/issues/1381
        todo!()
    }

    async fn load_settlement_job_from_db(
        _id: Ulid,
    ) -> eyre::Result<(SettlementJob, Option<SettlementJobResult>)> {
        // TODO: Load a settlement job's contents from L1, including its result if it is
        // completed.
        // XREF: https://github.com/agglayer/agglayer/issues/1381
        todo!()
    }

    async fn load_settlement_attempts_from_db(&mut self) -> eyre::Result<()> {
        // TODO: Load all the settlement attempts related to self into self
        // XREF: https://github.com/agglayer/agglayer/issues/1312
        todo!()
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
        &self,
        _attempt_number: SettlementAttemptNumber,
        _result: ClientError,
    ) {
        // TODO: Record a settlement attempt as being "client error" to db
        // XREF: https://github.com/agglayer/agglayer/issues/1317
        todo!()
    }

    async fn write_nonce_revert_to_db(
        &self,
        _wallet: Address,
        _nonce: Nonce,
        _attempt_number: SettlementAttemptNumber,
        _result: ContractCallResult,
    ) {
        // TODO: Record a nonce as having seen a revert to db. `attempt_number` is the
        // attempt that got included on L1, and all other attempts with the same nonce
        // should be marked as nonce already used.
        // XREF: https://github.com/agglayer/agglayer/issues/1317
        todo!()
    }

    async fn write_nonce_used_externally_to_db(
        &self,
        _wallet: Address,
        _nonce: Nonce,
        _tx_hash: SettlementTxHash,
    ) {
        // TODO: Record a nonce as having been used externally to db. All attempts with
        // this nonce should be marked as nonce already used.
        // XREF: https://github.com/agglayer/agglayer/issues/1317
        todo!()
    }

    async fn write_job_successful_to_db(
        &self,
        _wallet: Address,
        _nonce: Nonce,
        _attempt_number: SettlementAttemptNumber,
        _tx_result: ContractCallResult,
    ) {
        // TODO: Record a settlement job as successful to db, with the given result. All
        // attempts with the same nonce should be marked as nonce already used, and
        // attempts with different nonces as having seen a successful settlement
        // elsewhere.
        // XREF: https://github.com/agglayer/agglayer/issues/1317
        todo!()
    }

    async fn write_job_revert_to_db(&self, _result: &ContractCallResult) {
        // TODO: Record a settlement job as reverted to db, with the given result. All
        // attempts should already have been marked as nonce already used or revert, so
        // no need to update them, but maybe run a sanity-check.
        // XREF: https://github.com/agglayer/agglayer/issues/1317
        todo!()
    }
}
