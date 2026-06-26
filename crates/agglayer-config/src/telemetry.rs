use std::{net::SocketAddr, time::Duration};

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::DEFAULT_IP;

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct TelemetryConfig {
    #[serde(
        rename = "prometheus-addr",
        alias = "PrometheusAddr",
        default = "default_metrics_api_addr"
    )]
    pub addr: SocketAddr,

    /// Duration above which a synchronous RocksDB operation is recorded as slow
    /// (and may be blocking a Tokio worker thread).
    #[serde_as(as = "crate::with::HumanDuration")]
    #[serde(default = "default_slow_storage_op_threshold")]
    pub slow_storage_op_threshold: Duration,

    /// Interval at which the runtime scheduler-lag probe samples its timer.
    #[serde_as(as = "crate::with::HumanDuration")]
    #[serde(default = "default_scheduler_lag_probe_interval")]
    pub scheduler_lag_probe_interval: Duration,

    /// Scheduler lag above which a WARN is emitted. On an otherwise idle node
    /// this indicates worker threads were blocked long enough to delay
    /// unrelated work.
    #[serde_as(as = "crate::with::HumanDuration")]
    #[serde(default = "default_scheduler_lag_warn_threshold")]
    pub scheduler_lag_warn_threshold: Duration,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            addr: default_metrics_api_addr(),
            slow_storage_op_threshold: default_slow_storage_op_threshold(),
            scheduler_lag_probe_interval: default_scheduler_lag_probe_interval(),
            scheduler_lag_warn_threshold: default_scheduler_lag_warn_threshold(),
        }
    }
}

const fn default_metrics_api_addr() -> SocketAddr {
    SocketAddr::V4(std::net::SocketAddrV4::new(DEFAULT_IP, 3000))
}

const fn default_slow_storage_op_threshold() -> Duration {
    Duration::from_millis(25)
}

const fn default_scheduler_lag_probe_interval() -> Duration {
    Duration::from_millis(500)
}

const fn default_scheduler_lag_warn_threshold() -> Duration {
    Duration::from_millis(100)
}
