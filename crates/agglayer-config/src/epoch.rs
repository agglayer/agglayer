use std::time::Duration;

use serde::{Deserialize, Serialize};

/// The Epoch configuration.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Epoch {
    TimeClock(TimeClockConfig),
}

impl Default for Epoch {
    fn default() -> Self {
        Self::TimeClock(TimeClockConfig::default())
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
