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
pub struct EpochNumber(u64);

impl EpochNumber {
    pub const ZERO: EpochNumber = EpochNumber(0);
    pub const ONE: EpochNumber = EpochNumber(1);

    pub const fn new(epoch: u64) -> EpochNumber {
        EpochNumber(epoch)
    }

    #[must_use = "The value of the next epoch is returned but not used"]
    pub const fn next(&self) -> EpochNumber {
        EpochNumber(self.0.checked_add(1).expect("Epoch number overflow"))
    }

    pub const fn increment(&mut self) {
        *self = self.next();
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "testutils", derive(arbitrary::Arbitrary))]
pub struct EpochConfiguration {
    /// The genesis block where the AggLayer starts.
    pub genesis_block: u64,
    /// The duration of an epoch in blocks.
    pub epoch_duration: u64,
}
