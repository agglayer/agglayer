use std::{num::NonZeroU64, time::Duration};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The Epoch configuration.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Epoch {
    #[serde(alias = "TimeClock")]
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
        serialize_with = "serialize_duration",
        deserialize_with = "deserialize_duration",
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

fn default_block_epoch_duration() -> NonZeroU64 {
    NonZeroU64::new(8).unwrap()
}

const fn default_genesis_block() -> u64 {
    0
}

fn default_epoch_duration() -> Duration {
    Duration::from_secs(5)
}

fn serialize_duration<S>(value: &Duration, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u64(value.as_secs())
}

fn deserialize_duration<'de, D>(d: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let seconds = u64::deserialize(d)?;

    Ok(Duration::from_secs(seconds))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_epoch() {
        let epoch = Epoch::TimeClock(TimeClockConfig::default());
        let serialized = serde_json::to_string(&epoch).unwrap();

        assert_eq!(serialized, r#"{"time-clock":{"epoch-duration":5}}"#);
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
