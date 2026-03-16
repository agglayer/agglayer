use std::time::SystemTime;

use alloy::primitives::Bytes;

use crate::{Address, SettlementTxHash, B256, U256};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, derive_more::Display)]
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
    pub contract_address: Address,
    pub calldata: Bytes,
    pub eth_value: U256,
    pub gas_limit: u128,
    pub max_fee_per_gas_ceiling: u128,
    pub max_fee_per_gas_floor: u128,
    pub max_fee_per_gas_increase_percents: u32,
    pub max_priority_fee_per_gas_ceiling: u128,
    pub max_priority_fee_per_gas_floor: u128,
    pub max_priority_fee_per_gas_increase_percents: u32,
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
    NonceAlreadyUsed,
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
    pub block_hash: B256,
    pub block_number: u64,
    pub tx_hash: SettlementTxHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractCallOutcome {
    Success,
    Revert,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettlementAttempt {
    pub sender_wallet: Address,
    pub nonce: Nonce,
    pub max_fee_per_gas: u128,
    pub max_priority_fee_per_gas: u128,
    pub hash: SettlementTxHash,
    pub submission_time: SystemTime,
    pub result: Option<SettlementJobResult>,
}
