use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};

/// Configuration of the communication with the L2 nodes.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct L2 {
    #[serde(default = "L2::default_rpc_timeout")]
    #[serde_as(as = "DurationSeconds")]
    pub rpc_timeout: Duration,
}

impl L2 {
    const fn default_rpc_timeout() -> Duration {
        Duration::from_secs(45)
    }
}

impl Default for L2 {
    fn default() -> Self {
        Self {
            rpc_timeout: Self::default_rpc_timeout(),
        }
    }
}
