use std::{collections::BTreeMap, time::Duration};

use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;

pub type NetworkId = u32;

/// Time-based rate limiting
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TimeRateLimit {
    /// Don't apply any rate limiting, allowing the client to make requests as
    /// often as desired.
    Unlimited,

    /// Apply rate limit of `max_per_interval` events in given `time_interval`.
    #[serde(untagged, rename_all = "kebab-case")]
    Limited {
        max_per_interval: u32,

        #[serde(with = "crate::with::HumanDuration")]
        time_interval: Duration,
    },
}

impl TimeRateLimit {
    /// Default rate limiting for the `sendTx` call.
    pub const fn send_tx_default() -> Self {
        Self::Unlimited
    }

    /// Create a time-based rate limiting
    pub const fn limited(max_per_interval: u32, time_interval: Duration) -> Self {
        Self::Limited {
            max_per_interval,
            time_interval,
        }
    }
}

/// Rate limiting override for each endpoint
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
struct RateLimitOverride {
    send_tx: Option<TimeRateLimit>,
}

/// Rate limiting configuration for a single network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkRateLimitingConfig<'a> {
    /// Rate limit for `sendTx` for given network.
    pub send_tx: &'a TimeRateLimit,
}

/// Full rate limiting config.
/// Contains the defaults and the per-network overrides.
#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct RateLimitingConfig {
    /// Settlement rate limiting for the `sendTx` call.
    #[serde(default = "TimeRateLimit::send_tx_default")]
    send_tx: TimeRateLimit,

    /// Per-network rate limiting overrides.
    #[serde(default)]
    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
    network: BTreeMap<NetworkId, RateLimitOverride>,
}

impl RateLimitingConfig {
    /// Default rate limiting configuration.
    pub const DEFAULT: Self = Self::new(TimeRateLimit::send_tx_default());

    /// New rate limiting config with no network-specific settings.
    pub const fn new(send_tx: TimeRateLimit) -> Self {
        let network = BTreeMap::new();
        Self { send_tx, network }
    }

    /// Override `sendTx`setting for given network.
    pub fn with_send_tx_override(mut self, nid: NetworkId, limit: TimeRateLimit) -> Self {
        self.network.entry(nid).or_default().send_tx = Some(limit);
        self
    }

    /// Get rate limiting configuration for given network.
    pub fn config_for(&self, network_id: NetworkId) -> NetworkRateLimitingConfig {
        let overrides = self.override_for(network_id);
        let send_tx = overrides
            .and_then(|l| l.send_tx.as_ref())
            .unwrap_or(&self.send_tx);
        NetworkRateLimitingConfig { send_tx }
    }

    fn override_for(&self, nid: NetworkId) -> Option<&RateLimitOverride> {
        self.network.get(&nid)
    }
}

impl Default for RateLimitingConfig {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn default_config() {
        let config_str = "send-tx = \"unlimited\"";
        let parsed_default_config: RateLimitingConfig = toml::from_str(config_str).unwrap();
        assert_eq!(parsed_default_config, RateLimitingConfig::DEFAULT);

        let empty_config: RateLimitingConfig = toml::from_str("").unwrap();
        assert_eq!(empty_config, RateLimitingConfig::DEFAULT);
    }

    #[test]
    fn unlimited() {
        let config_str = "send-tx = \"unlimited\"";
        let config: RateLimitingConfig = toml::from_str(config_str).unwrap();
        let expected = RateLimitingConfig::new(TimeRateLimit::Unlimited);
        assert_eq!(config, expected);
    }

    #[rstest::rstest]
    #[case(4, "1h 20min", 80 * 60)]
    #[case(2, "30min", 1800)]
    #[case(50, "90s", 90)]
    #[case(0, "2min", 120)]
    fn basic(#[case] limit: u32, #[case] interval_str: String, #[case] interval_secs: u64) {
        #[rustfmt::skip]
        let config_str = format!(
            "[send-tx]\n\
            max-per-interval = {limit}\n\
            time-interval = {interval_str:?}\n"
        );
        let config: RateLimitingConfig = toml::from_str(&config_str).unwrap();
        let expected = RateLimitingConfig::new(TimeRateLimit::Limited {
            max_per_interval: limit,
            time_interval: Duration::from_secs(interval_secs),
        });
        assert_eq!(config, expected);
    }

    #[test]
    fn top_level_and_override() {
        #[rustfmt::skip]
        let config_str = "[send-tx]\n\
            max-per-interval = 3\n\
            time-interval = \"30min\"\n\
            [network.1.send-tx]\n\
            max-per-interval = 4\n\
            time-interval = \"40min\"\n";
        let config: RateLimitingConfig = toml::from_str(config_str).unwrap();

        let default_send_tx_limit = TimeRateLimit::limited(3, Duration::from_secs(1800));
        let network_1_send_tx_limit = TimeRateLimit::limited(4, Duration::from_secs(2400));
        let network_1_override = RateLimitOverride {
            send_tx: Some(network_1_send_tx_limit.clone()),
        };

        let expected = RateLimitingConfig {
            send_tx: default_send_tx_limit.clone(),
            network: BTreeMap::from_iter([(1, network_1_override)]),
        };

        assert_eq!(config, expected);
        assert_eq!(config.config_for(1).send_tx, &network_1_send_tx_limit);
        assert_eq!(config.config_for(2).send_tx, &default_send_tx_limit);
        assert_eq!(config.config_for(1337).send_tx, &default_send_tx_limit);
    }
}
