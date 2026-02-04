use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, OnceLock},
    time::SystemTime,
};

use agglayer_config::{settlement_service::SettlementTransactionConfig, Multiplier};
use agglayer_types::SettlementTxHash;
use alloy::primitives::{Address, BlockHash, Bytes, U128, U256};
use tokio::sync::mpsc;
use ulid::Ulid;

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
    attempts: HashMap<SettlementAttemptNumber, SettlementAttempt>,
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
            attempts: HashMap::new(),
        };
        this.write_settlement_job_to_db().await?;
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
                attempts: HashMap::new(),
            };
            this.load_settlement_attempts_from_db().await?;
            Ok(StoredSettlementJob::Pending(this))
        }
    }

    pub async fn run(&mut self) -> SettlementJobResult {
        loop {
            // Is any of the previous settlement attempts included on L1?
            let included_attempts = self.attempts_included_on_l1().await;
            let earliest_included_tx = included_attempts
                .into_iter()
                .min_by_key(|(_, result)| result.block_number);
            // TODO: Also filter by index-in-block
            if let Some((attempt, result)) = earliest_included_tx {
                // We know that only the first tx has a chance to settle, as any other tx
                // included later would necessarily conflict with the first included tx.
                let included_tx_hash = self.attempt(attempt).hash;
                if !self.wait_for_finalization(included_tx_hash).await {
                    // TODO: admin commands handling
                    // Finalization failed, go back to the topmost loop
                    continue;
                } else {
                    // Finalization succeeded
                    let result = SettlementJobResult::ContractCall(result);
                    self.write_attempt_result_to_db(attempt, &result).await;
                    return result;
                }
            }

            // Is there a settlement attempt without a result recorded in database yet?
            let pending_nonces = self.lowest_nonces_with_pending_attempts();
            let next_attempt = if pending_nonces.is_empty() {
                self.build_next_attempt_with_same_nonce()
            } else {
                // Wait for any of the pending attempts' nonces to be ready to be filled on L1
                // (ie. previous nonce is filled)
                let previous_nonces = pending_nonces
                    .iter()
                    .map(|(address, nonce)| {
                        nonce.previous().map(|prev_nonce| (*address, prev_nonce))
                    })
                    .collect::<Option<HashSet<_>>>();
                if let Some(previous_nonces) = previous_nonces {
                    // None means at least one nonce is 0
                    self.wait_for_any_nonce_to_be_filled(&previous_nonces).await;
                    // TODO: admin commands handling
                }

                // Wait for any of the pending attempts' nonces to be used on L1
                let filled = tokio::time::timeout(
                    self.next_non_inclusion_timeout(),
                    self.wait_for_any_nonce_to_be_filled(&pending_nonces),
                )
                .await;
                // TODO: admin commands handling

                // Confirm that the reason the nonce was used is because the attempt got
                // included on L1
                match filled {
                    Ok((address, nonce, tx_hash)) => {
                        if let Some(attempt_number) = self.settlement_attempt_number_for(tx_hash) {
                            let attempt = self.attempt(attempt_number);
                            assert_eq!(attempt.hash, tx_hash);
                            assert_eq!(attempt.sender_wallet, address);
                            assert_eq!(attempt.nonce, nonce);
                            // A pending attempt got included, go back to the topmost loop to wait
                            // for finalization
                            continue;
                        } else {
                            // Nonce used but the tx was not one of our attempts, the nonce was used
                            // by something else
                            self.write_nonce_used_on_l1_to_db(address, nonce, tx_hash)
                                .await;
                            self.build_next_attempt_with_new_nonce()
                        }
                    }
                    Err(error) => {
                        let _: tokio::time::error::Elapsed = error;
                        // Timeout expired, we can create a new attempt
                        for (address, nonce) in pending_nonces.iter() {
                            self.write_timeout_to_db(*address, *nonce).await;
                        }
                        self.build_next_attempt_with_same_nonce()
                    }
                }
            };

            self.write_attempt_to_db(&next_attempt).await;
            self.submit_attempt_to_l1(&next_attempt).await;
            // Loop back to the topmost loop, checking for inclusion on L1
        }
    }

    fn attempt(&self, attempt_number: SettlementAttemptNumber) -> &SettlementAttempt {
        self.attempts
            .get(&attempt_number)
            .expect("Attempt number not found internally to the SettlementTask itself")
    }

    fn settlement_attempt_number_for(
        &self,
        tx_hash: SettlementTxHash,
    ) -> Option<SettlementAttemptNumber> {
        for (attempt_number, attempt) in self.attempts.iter() {
            if attempt.hash == tx_hash {
                return Some(*attempt_number);
            }
        }
        None
    }

    fn lowest_nonces_with_pending_attempts(&self) -> HashSet<(Address, Nonce)> {
        let addresses = self
            .attempts
            .values()
            .map(|attempt| attempt.sender_wallet)
            .collect::<HashSet<Address>>();
        addresses
            .into_iter()
            .filter_map(|address| {
                self.attempts
                    .iter()
                    .filter(|(_, attempt)| {
                        attempt.sender_wallet == address && attempt.result.is_none()
                    })
                    .map(|(_, attempt)| (attempt.sender_wallet, attempt.nonce))
                    .min_by_key(|(_, nonce)| *nonce)
            })
            .collect()
    }

    fn next_non_inclusion_timeout(&self) -> std::time::Duration {
        // TODO: count the number of attempts with a result, and figure out what the
        // next non-inclusion timeout should be
        // XREF: TODO: no specific issue to tag yet
        todo!()
    }

    fn build_next_attempt_with_same_nonce(&mut self) -> SettlementAttemptNumber {
        // TODO: create the next attempt with the same nonce as the latest attempt (or
        // the next available nonce if no attempt yet)
        // XREF: TODO: no specific issue to tag yet
        todo!()
    }

    fn build_next_attempt_with_new_nonce(&mut self) -> SettlementAttemptNumber {
        // TODO: create the next attempt with a new nonce (latest nonce + 1)
        // XREF: TODO: no specific issue to tag yet
        todo!()
    }

    async fn write_settlement_job_to_db(&self) -> eyre::Result<()> {
        // TODO: write settlement job data to rocksdb
        // XREF: https://github.com/agglayer/agglayer/issues/1320
        todo!()
    }

    async fn load_settlement_job_from_db(
        _id: Ulid,
    ) -> eyre::Result<(SettlementJob, Option<SettlementJobResult>)> {
        // TODO: load settlement job data from rocksdb
        // XREF: TODO: no specific issue to tag yet
        todo!()
    }

    async fn load_settlement_attempts_from_db(&mut self) -> eyre::Result<()> {
        // TODO: load settlement attempts data from rocksdb
        // XREF: https://github.com/agglayer/agglayer/issues/1312
        todo!()
    }

    async fn write_attempt_to_db(&self, _attempt_number: &SettlementAttemptNumber) {
        // TODO: write settlement attempt data to rocksdb
        // XREF: https://github.com/agglayer/agglayer/issues/1320
        todo!()
    }

    async fn write_nonce_used_on_l1_to_db(
        &self,
        address: Address,
        nonce: Nonce,
        tx_hash: SettlementTxHash,
    ) {
        let result = SettlementJobResult::ClientError(ClientError::nonce_already_used(
            address, nonce, tx_hash,
        ));
        for (attempt_number, attempt) in self.attempts.iter() {
            if attempt.sender_wallet == address && attempt.nonce == nonce {
                self.write_attempt_result_to_db(*attempt_number, &result)
                    .await;
            }
        }
    }

    async fn write_timeout_to_db(&self, address: Address, nonce: Nonce) {
        let result = SettlementJobResult::ClientError(ClientError::timeout_waiting_for_inclusion());
        for (attempt_number, attempt) in self.attempts.iter() {
            if attempt.sender_wallet == address && attempt.nonce == nonce {
                self.write_attempt_result_to_db(*attempt_number, &result)
                    .await;
            }
        }
    }

    async fn submit_attempt_to_l1(&self, _attempt_number: &SettlementAttemptNumber) {
        // TODO: submit the given attempt to L1
        // XREF: https://github.com/agglayer/agglayer/issues/1321
        todo!()
    }

    async fn wait_for_any_nonce_to_be_filled(
        &self,
        _nonces: &HashSet<(Address, Nonce)>,
    ) -> (Address, Nonce, SettlementTxHash) {
        // TODO: wait for any of the given nonces to be filled on L1
        // XREF: https://github.com/agglayer/agglayer/issues/1314
        todo!()
    }

    async fn attempts_included_on_l1(&self) -> Vec<(SettlementAttemptNumber, ContractCallResult)> {
        // TODO: check which attempts have been included on L1, going over ALL previous
        // attempts, included any attempt that was originally marked as "in error" (but
        // only transient ones, not permanent ones)
        // XREF: https://github.com/agglayer/agglayer/issues/1313
        // TODO: also submit with new nonce instead of with old nonce if all previous
        // attempts are seen as permfail here. If even one attempt is marked as
        // tempfail, then even if we don't have access to the wallet any longer
        // we should probably just wait and not retry with new nonce, to avoid
        // race conditions. Note that ReorganizedResult should count as
        // transient until the nonce is used again. Maybe ReorganizedResult
        // should actually have an additional field that indicates what the new
        // result post-reorg would be? That feels like maybe the safest option.
        // But this also raises the question, what if a reorg converts a permanent
        // failure into an actual success? We need to give some more thought to
        // reorgs, or maybe wait for finalization of the permfail before writing
        // it to db?
        todo!()
    }

    async fn is_attempt_included_on_l1(&self, _attempt_number: SettlementAttemptNumber) -> bool {
        // TODO: check whether the given attempt has been included on L1
        // XREF: TODO: no specific issue to tag yet
        todo!()
    }

    async fn wait_for_finalization(&self, _tx: SettlementTxHash) -> bool {
        // TODO: wait for finalization of the given transaction on L1
        // XREF: https://github.com/agglayer/agglayer/issues/1316
        todo!()
    }

    async fn write_attempt_result_to_db(
        &self,
        _attempt_number: SettlementAttemptNumber,
        _result: &SettlementJobResult,
    ) {
        // TODO: write attempt result to rocksdb
        // XREF: https://github.com/agglayer/agglayer/issues/1317
        todo!()
    }
}
