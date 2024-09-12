use std::{collections::BTreeMap, time::Duration};

use serde::Deserialize;

pub type RollupId = u32;

/// Time-based rate limiting
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TimeRateLimit {
    Unlimited,

    #[serde(untagged, rename_all = "kebab-case")]
    Limited {
        max_per_interval: u32,
        #[serde(with = "humantime_serde")]
        time_interval: Duration,
    },
}

impl TimeRateLimit {
    /// Default rate limiting for the `sendTx` call.
    pub const fn send_tx_default() -> Self {
        Self::limited(1, Duration::from_secs(60 * 60))
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
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
struct RateLimitOverride {
    send_tx: Option<TimeRateLimit>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(try_from = "BTreeMap<String, RateLimitOverride>")]
pub struct PerNetworkRateLimitOverride(BTreeMap<RollupId, RateLimitOverride>);

impl PerNetworkRateLimitOverride {
    pub const fn new() -> Self {
        Self(BTreeMap::new())
    }
}

impl TryFrom<BTreeMap<String, RateLimitOverride>> for PerNetworkRateLimitOverride {
    type Error = <RollupId as std::str::FromStr>::Err;

    fn try_from(value: BTreeMap<String, RateLimitOverride>) -> Result<Self, Self::Error> {
        let overrides = value
            .into_iter()
            .map(|(k, v)| Ok((k.parse::<RollupId>()?, v)))
            .collect::<Result<Self, Self::Error>>()?;
        Ok(overrides)
    }
}

impl FromIterator<(RollupId, RateLimitOverride)> for PerNetworkRateLimitOverride {
    fn from_iter<T: IntoIterator<Item = (RollupId, RateLimitOverride)>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

/// Full rate limiting config.
/// Contains the defaults and the per-network overrides.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct RateLimitingConfig {
    /// Settlement rate limiting for the `sendTx` call.
    #[serde(default = "TimeRateLimit::send_tx_default")]
    send_tx: TimeRateLimit,

    /// Per-network rate limiting overrides.
    #[serde(default)]
    network: PerNetworkRateLimitOverride,
}

impl RateLimitingConfig {
    /// Default rate limiting configuration.
    pub const DEFAULT: Self = Self::new(TimeRateLimit::send_tx_default());

    /// New rate limiting config with no network-specific settings.
    pub const fn new(send_tx: TimeRateLimit) -> Self {
        Self {
            send_tx,
            network: PerNetworkRateLimitOverride::new(),
        }
    }

    /// Override `sendTx`setting for given network
    pub fn with_send_tx_override(mut self, nid: RollupId, limit: TimeRateLimit) -> Self {
        let limit_override = RateLimitOverride {
            send_tx: Some(limit),
        };
        let _ = self.network.0.insert(nid, limit_override);
        self
    }

    /// Get rate limiting for the `sendTx` call for given network.
    pub fn send_tx_limit(&self, nid: RollupId) -> &TimeRateLimit {
        self.override_for(nid)
            .and_then(|l| l.send_tx.as_ref())
            .unwrap_or(&self.send_tx)
    }

    fn override_for(&self, nid: RollupId) -> Option<&RateLimitOverride> {
        self.network.0.get(&nid)
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
        #[rustfmt::skip]
        let config_str = "[send-tx]\n\
            max-per-interval = 1\n\
            time-interval = \"1h\"\n";
        let parsed_default_config: RateLimitingConfig = toml::from_str(config_str).unwrap();
        assert_eq!(parsed_default_config, RateLimitingConfig::DEFAULT);

        let empty_config: RateLimitingConfig = toml::from_str("").unwrap();
        assert_eq!(empty_config, RateLimitingConfig::DEFAULT);
    }

    #[test]
    fn unlimited() {
        let config_str = "send-tx = \"unlimited\"";
        let config: RateLimitingConfig = toml::from_str(&config_str).unwrap();
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
        let config: RateLimitingConfig = toml::from_str(&config_str).unwrap();

        let default_send_tx_limit = TimeRateLimit::limited(3, Duration::from_secs(1800));
        let network_1_send_tx_limit = TimeRateLimit::limited(4, Duration::from_secs(2400));
        let network_1_override = RateLimitOverride {
            send_tx: Some(network_1_send_tx_limit.clone()),
        };

        let expected = RateLimitingConfig {
            send_tx: default_send_tx_limit.clone(),
            network: PerNetworkRateLimitOverride::from_iter([(1, network_1_override)]),
        };

        assert_eq!(config, expected);
        assert_eq!(config.send_tx_limit(1), &network_1_send_tx_limit);
        assert_eq!(config.send_tx_limit(2), &default_send_tx_limit);
        assert_eq!(config.send_tx_limit(1337), &default_send_tx_limit);
    }
}
