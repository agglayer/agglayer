use alloy::primitives::BlockNumber;

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    derive_more::Display,
    derive_more::From,
    serde::Deserialize,
    serde::Serialize,
)]
#[cfg_attr(feature = "testutils", derive(arbitrary::Arbitrary))]
#[serde(transparent)]
pub struct SettlementBlockNumber(u64);

impl SettlementBlockNumber {
    pub const ZERO: SettlementBlockNumber = SettlementBlockNumber::new(0);

    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    pub const fn new(block_no: BlockNumber) -> SettlementBlockNumber {
        SettlementBlockNumber(block_no)
    }
}

impl Into<BlockNumber> for SettlementBlockNumber {
    fn into(self) -> BlockNumber {
        self.0
    }
}
