use std::time::SystemTime;

use alloy::primitives::Bytes;

use crate::{Address, SettlementTxHash, B256, U256};

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    derive_more::Display,
    derive_more::From,
    derive_more::Into,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(transparent)]
pub struct SettlementJobId(ulid::Ulid);

impl SettlementJobId {
    pub const BYTE_LEN: usize = std::mem::size_of::<u128>();

    pub const fn new(value: ulid::Ulid) -> Self {
        Self(value)
    }

    pub const fn as_ulid(&self) -> &ulid::Ulid {
        &self.0
    }

    pub const fn into_ulid(self) -> ulid::Ulid {
        self.0
    }

    pub const fn from_be_bytes(bytes: [u8; Self::BYTE_LEN]) -> Self {
        Self(ulid::Ulid::from_bytes(bytes))
    }

    pub const fn to_be_bytes(&self) -> [u8; Self::BYTE_LEN] {
        self.0.to_bytes()
    }
}

impl From<u128> for SettlementJobId {
    fn from(value: u128) -> Self {
        Self(ulid::Ulid::from(value))
    }
}

#[cfg(feature = "testutils")]
impl<'a> arbitrary::Arbitrary<'a> for SettlementJobId {
    fn arbitrary(input: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self(ulid::Ulid::from(
            <u128 as arbitrary::Arbitrary>::arbitrary(input)?,
        )))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, derive_more::Display)]
pub struct SettlementAttemptNumber(pub u64);

impl From<u64> for SettlementAttemptNumber {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettlementJobResult {
    pub wallet: Address,
    pub nonce: Nonce,
    pub attempt_number: SettlementAttemptNumber,
    pub contract_call_result: ContractCallResult,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SettlementAttemptResult {
    ClientError(ClientError),
    ContractCall(ContractCallResult),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientError {
    pub kind: ClientErrorType,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientErrorType {
    Unknown,
    NonceAlreadyUsed,
    SettlementSucceededElsewhere,
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

    pub fn settlement_succeeded_elsewhere(tx_hash: SettlementTxHash) -> Self {
        Self {
            kind: ClientErrorType::SettlementSucceededElsewhere,
            message: format!("Settlement succeeded in transaction {tx_hash}"),
        }
    }
}

impl SettlementAttemptResult {
    /// May `replacement` overwrite `self`? Only a stronger result replaces a
    /// weaker one.
    pub fn can_be_replaced_by(&self, replacement: &Self) -> bool {
        match (self, replacement) {
            (Self::ClientError(_), Self::ContractCall(_)) => true,
            (Self::ClientError(existing_error), Self::ClientError(replacement_error)) => {
                existing_error.kind == ClientErrorType::Unknown
                    && matches!(
                        replacement_error.kind,
                        ClientErrorType::NonceAlreadyUsed
                            | ClientErrorType::SettlementSucceededElsewhere
                    )
            }
            _ => false,
        }
    }

    /// True for "nonce used elsewhere" / "settled elsewhere" results: notes
    /// that another tx handled the attempt. They never overwrite a real
    /// result.
    pub fn is_resolved_elsewhere(&self) -> bool {
        matches!(
            self,
            Self::ClientError(ClientError {
                kind: ClientErrorType::NonceAlreadyUsed
                    | ClientErrorType::SettlementSucceededElsewhere,
                ..
            })
        )
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
    pub hash: SettlementTxHash,
    pub submission_time: SystemTime,
    /// `max_fee_per_gas` (wei) of the signed attempt; the baseline a retry
    /// bumps from.
    pub max_fee_per_gas: u128,
    /// `max_priority_fee_per_gas` (wei) of the signed attempt; the baseline a
    /// retry bumps from.
    pub max_priority_fee_per_gas: u128,
}

#[cfg(test)]
mod tests;
