use std::{num::NonZeroU64, time::Duration};

use serde::{Deserialize, Serialize};

/// The Epoch configuration.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Epoch {
    TimeClock(TimeClockConfig),
    BlockClock(BlockClockConfig),
}

impl Default for Epoch {
    fn default() -> Self {
        Self::BlockClock(BlockClockConfig::default())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct BlockClockConfig {
    #[serde(default = "default_block_epoch_duration")]
    pub epoch_duration: NonZeroU64,

    #[serde(default = "default_genesis_block")]
    pub genesis_block: u64,
}

impl Default for BlockClockConfig {
    fn default() -> Self {
        Self {
            epoch_duration: default_block_epoch_duration(),
            genesis_block: default_genesis_block(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct TimeClockConfig {
    #[serde(
        default = "default_epoch_duration",
        with = "crate::with::HumanDuration",
        alias = "EpochDuration"
    )]
    pub epoch_duration: Duration,
}

impl Default for TimeClockConfig {
    fn default() -> Self {
        Self {
            epoch_duration: default_epoch_duration(),
        }
    }
}

// We estimate the block time of L1 to 10min.
// The goal is to have an epoch duration of 1h.
// So we need 6 blocks per epoch.
fn default_block_epoch_duration() -> NonZeroU64 {
    NonZeroU64::new(6).unwrap()
}

const fn default_genesis_block() -> u64 {
    0
}

fn default_epoch_duration() -> Duration {
    Duration::from_secs(60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_epoch() {
        let epoch = Epoch::TimeClock(TimeClockConfig::default());
        let serialized = serde_json::to_string(&epoch).unwrap();

        assert_eq!(serialized, r#"{"time-clock":{"epoch-duration":"1m"}}"#);
    }

    #[test]
    fn deserialize_epoch() {
        let config = r#"{"time-clock":{"epoch-duration":3600}}"#;

        let expected_duration = Duration::from_secs(3600);
        let epoch: Epoch = serde_json::from_str(config).unwrap();

        assert!(
            matches!(epoch, Epoch::TimeClock(TimeClockConfig { epoch_duration }) if epoch_duration == expected_duration)
        );
    }
}
