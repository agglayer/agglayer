use std::time::Duration;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The Epoch configuration.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum Epoch {
    TimeClock(TimeClockConfig),
}

impl Default for Epoch {
    fn default() -> Self {
        Self::TimeClock(TimeClockConfig::default())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimeClockConfig {
    #[serde(
        default = "default_epoch_duration",
        serialize_with = "serialize_duration",
        deserialize_with = "deserialize_duration",
        rename = "EpochDuration"
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
