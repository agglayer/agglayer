use std::time::Duration;

use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use serde_with::DurationSeconds;

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ShutdownConfig {
    #[serde(default = "default_shutdown_runtime_timeout")]
    #[serde_as(as = "DurationSeconds")]
    pub runtime_timeout: Duration,
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            runtime_timeout: default_shutdown_runtime_timeout(),
        }
    }
}

const fn default_shutdown_runtime_timeout() -> Duration {
    Duration::from_secs(5)
}
