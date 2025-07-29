use crate::{Digest, B256};

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    derive_more::AsRef,
    derive_more::Display,
    derive_more::From,
    derive_more::Into,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(transparent)]
pub struct SettlementTxHash(Digest);

impl SettlementTxHash {
    pub const fn for_tests() -> Self {
        SettlementTxHash(Digest::ZERO)
    }

    pub const fn new(hash: Digest) -> Self {
        SettlementTxHash(hash)
    }
}

impl From<B256> for SettlementTxHash {
    fn from(hash: B256) -> Self {
        SettlementTxHash(Digest::from(hash))
    }
}

impl From<SettlementTxHash> for B256 {
    fn from(tx_hash: SettlementTxHash) -> Self {
        tx_hash.0.as_bytes().into()
    }
}
