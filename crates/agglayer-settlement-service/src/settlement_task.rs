use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
    time::SystemTime,
};

use agglayer_config::Multiplier;
use agglayer_types::SettlementTxHash;
use alloy::primitives::{Address, BlockHash, Bytes, U128, U256};
use tokio::sync::mpsc;
use ulid::Ulid;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SettlementAttemptNumber(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SettlementJobResult {
    ClientError(ClientError),
    ContractCall(ContractCallResult),
    Reorganized(ReorganizedResult),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientError {
    pub kind: ClientErrorType,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClientErrorType {
    Transient,
    Permanent,
}

impl ClientError {
    pub fn nonce_already_used() -> Self {
        Self {
            kind: ClientErrorType::Permanent,
            message: "Nonce already used".to_string(),
        }
    }

    pub fn timeout_waiting_for_inclusion() -> Self {
        Self {
            kind: ClientErrorType::Transient,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReorganizedResult {
    pub reorg_detection_time: SystemTime,
    pub previous_result: Box<SettlementJobResult>,
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
    pub gas_limit: u128,
    pub max_fee_per_gas: u128,
    pub max_priority_fee_per_gas: u128,
    pub hash: SettlementTxHash,
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
            if let Some((attempt, result)) = earliest_included_tx {
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
            let pending_attempts = self.pending_attempts_with_lowest_nonces();
            let next_attempt = if pending_attempts.is_empty() {
                self.build_next_attempt_with_same_nonce()
            } else {
                // Wait for any of the pending attempts' nonces to be ready to be filled on L1
                // (ie. previous nonce is filled)
                let previous_nonces = pending_attempts
                    .iter()
                    .map(|(attempt_number, address, nonce)| {
                        nonce
                            .previous()
                            .map(|prev_nonce| (*attempt_number, *address, prev_nonce))
                    })
                    .collect::<Option<Vec<_>>>();
                if let Some(previous_nonces) = previous_nonces {
                    // None means at least one nonce is 0
                    self.wait_for_any_nonce_to_be_filled(&previous_nonces).await;
                    // TODO: admin commands handling
                }

                // Wait for any of the pending attempts' nonces to be used on L1
                let pending_nonces = pending_attempts
                    .iter()
                    .map(|(attempt_number, address, nonce)| (*attempt_number, *address, *nonce))
                    .collect::<Vec<_>>();
                let filled = tokio::time::timeout(
                    self.next_non_inclusion_timeout(),
                    self.wait_for_any_nonce_to_be_filled(&pending_nonces),
                )
                .await;
                // TODO: admin commands handling

                // Confirm that the reason the nonce was used is because the attempt got
                // included on L1
                match filled {
                    Ok(attempt_number) if self.is_attempt_included_on_l1(attempt_number).await => {
                        // A pending attempt got included, go back to the topmost loop to wait for
                        // finalization
                        continue;
                    }
                    Ok(attempt_number) => {
                        // Nonce used but attempt not included, the nonce was used by something else
                        let result =
                            SettlementJobResult::ClientError(ClientError::nonce_already_used());
                        self.write_attempt_result_to_db(attempt_number, &result)
                            .await;
                        self.build_next_attempt_with_new_nonce()
                    }
                    Err(error) => {
                        let _: tokio::time::error::Elapsed = error;
                        // Timeout expired, we can create a new attempt
                        let result = SettlementJobResult::ClientError(
                            ClientError::timeout_waiting_for_inclusion(),
                        );
                        for (attempt_number, _, _) in pending_nonces.iter() {
                            self.write_attempt_result_to_db(*attempt_number, &result)
                                .await;
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

    fn pending_attempts_with_lowest_nonces(
        &self,
    ) -> Vec<(SettlementAttemptNumber, Address, Nonce)> {
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
                    .map(|(attempt_number, attempt)| {
                        (*attempt_number, attempt.sender_wallet, attempt.nonce)
                    })
                    .min_by_key(|(_, _, nonce)| *nonce)
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

    async fn submit_attempt_to_l1(&self, _attempt_number: &SettlementAttemptNumber) {
        // TODO: submit the given attempt to L1
        // XREF: https://github.com/agglayer/agglayer/issues/1321
        todo!()
    }

    async fn wait_for_any_nonce_to_be_filled(
        &self,
        _nonces: &[(SettlementAttemptNumber, Address, Nonce)],
    ) -> SettlementAttemptNumber {
        // TODO: wait for any of the given nonces to be filled on L1
        // XREF: https://github.com/agglayer/agglayer/issues/1314
        todo!()
    }

    async fn attempts_included_on_l1(&self) -> Vec<(SettlementAttemptNumber, ContractCallResult)> {
        // TODO: check which attempts have been included on L1, going over ALL previous
        // attempts
        // XREF: https://github.com/agglayer/agglayer/issues/1313
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
