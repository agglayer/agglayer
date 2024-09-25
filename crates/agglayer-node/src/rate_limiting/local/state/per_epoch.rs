use std::num::NonZeroU32;

pub use agglayer_types::EpochNumber;

/// Information tracked by a per-epoch rate limiter.
pub struct PerEpochState {
    /// Number of events recorded in the current epoch.
    num_events: usize,

    /// The latest epoch processed by the rate limiter.
    epoch: EpochNumber,

    /// The maximum number of events per epoch.
    max_per_epoch: NonZeroU32,
}

impl PerEpochState {
    /// New per-epoch rate limiter state.
    pub fn new(max_per_epoch: NonZeroU32) -> Self {
        Self {
            num_events: 0,
            epoch: 0,
            max_per_epoch,
        }
    }
}

impl super::RawState for PerEpochState {
    type Instant = EpochNumber;

    type LimitedInfo = RateLimited;

    fn prune(&mut self, epoch: EpochNumber) {
        if self.epoch < epoch {
            self.epoch = epoch;
            self.num_events = 0;
        }
    }

    fn query(&self) -> usize {
        self.num_events
    }

    fn record(&mut self, epoch: EpochNumber) {
        assert_eq!(epoch, self.epoch, "Limiter epoch out of date");
        assert!(self.num_events < self.max_events());
        self.num_events += 1;
    }

    fn limited_info(&self, epoch: EpochNumber) -> Self::LimitedInfo {
        assert_eq!(epoch, self.epoch, "Limiter epoch out of date");
        RateLimited {
            max_per_epoch: self.max_per_epoch.get(),
            epoch,
        }
    }

    fn max_events(&self) -> usize {
        self.max_per_epoch.get() as usize
    }
}

#[derive(Clone, Eq, PartialEq, Debug, serde::Serialize, thiserror::Error)]
#[serde(rename_all = "kebab-case")]
#[error("Limit reached")]
pub struct RateLimited {
    /// Maximum number of events per epoch.
    max_per_epoch: u32,

    /// Current epoch.
    epoch: EpochNumber,
}
