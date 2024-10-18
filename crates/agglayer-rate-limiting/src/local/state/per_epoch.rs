use std::num::NonZeroU32;

pub use agglayer_types::EpochNumber;
use agglayer_utils::{log_assert, log_assert_eq};

/// Information tracked by a per-epoch rate limiter.
#[derive(Clone, Debug, PartialEq, Eq)]
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
        log_assert_eq!(epoch, self.epoch, "Limiter epoch out of date");
        log_assert!(self.num_events < self.max_events());
        self.num_events += 1;
    }

    fn limited_info(&self, epoch: EpochNumber) -> Self::LimitedInfo {
        log_assert_eq!(epoch, self.epoch, "Limiter epoch out of date");
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
    pub max_per_epoch: u32,

    /// Current epoch.
    pub epoch: EpochNumber,
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::{EpochNumber, PerEpochState};
    use crate::local::{limiter::RateLimiterCore, state::RawState};

    impl PerEpochState {
        /// Prune the state twice, checking the operation is idempotent.
        fn prune_twice(&mut self, epoch: EpochNumber) {
            self.prune(epoch);
            let after_one_prune = self.clone();
            self.prune(epoch);
            assert_eq!(self, &after_one_prune);
        }
    }

    #[test]
    fn raw_state_update() {
        let mut state = PerEpochState::new(NonZeroU32::MAX);

        state.prune_twice(3);
        assert_eq!(state.epoch, 3);
        assert_eq!(state.num_events, 0);

        state.record(3);
        assert_eq!(state.epoch, 3);
        assert_eq!(state.num_events, 1);

        state.record(3);
        assert_eq!(state.epoch, 3);
        assert_eq!(state.num_events, 2);

        state.prune_twice(2);
        assert_eq!(state.epoch, 3);
        assert_eq!(state.num_events, 2);

        state.prune_twice(4);
        assert_eq!(state.epoch, 4);
        assert_eq!(state.num_events, 0);
    }

    type TestLimiter = RateLimiterCore<PerEpochState>;

    #[test]
    fn limiter_basic_operation() {
        let epoch_events_and_expected_outcomes = [
            (2, true),
            (2, true),
            (3, true),
            (3, true),
            (3, true),
            (3, false),
            (3, false),
            (7, true),
            (9, true),
            (9, true),
            (9, true),
            (9, false),
            (10, true),
        ];

        let mut limiter = TestLimiter::new(PerEpochState::new(3.try_into().unwrap()));

        for (epoch, ok) in epoch_events_and_expected_outcomes {
            let result = limiter.reserve(epoch);
            assert_eq!(result.is_ok(), ok, "Failed in epoch {epoch}");
            let _ = result.map_or((), |slot| limiter.record(epoch, slot));
        }
    }

    #[test]
    fn with_reservations() {
        let mut limiter = TestLimiter::new(PerEpochState::new(3.try_into().unwrap()));

        let slot = limiter.reserve(5).unwrap();
        assert_eq!(limiter.query(5), 1);
        limiter.record(5, slot);

        let slot0 = limiter.reserve(5).unwrap();
        let slot1 = limiter.reserve(5).unwrap();
        let _err = limiter.reserve(5).unwrap_err();

        limiter.release(slot1);
        assert_eq!(limiter.query(5), 2);

        limiter.record(5, slot0);
        assert_eq!(limiter.query(5), 2);

        assert_eq!(limiter.query(6), 0);
    }

    #[test]
    fn limiting_error() {
        let mut limiter = TestLimiter::new(PerEpochState::new(1.try_into().unwrap()));

        let slot = limiter.reserve(5).unwrap();
        let err = limiter.reserve(5).unwrap_err();
        limiter.release(slot);

        assert_eq!(
            err,
            super::RateLimited {
                max_per_epoch: 1,
                epoch: 5
            },
        );
    }
}
