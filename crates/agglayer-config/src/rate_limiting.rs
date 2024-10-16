use std::{collections::BTreeMap, time::Duration};

use serde::{Deserialize, Serialize};

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

        #[serde(with = "humantime_serde")]
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

/// Epoch-based rate limiting
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EpochRateLimit {
    /// Don't apply any rate limiting, allowing the client to make requests as
    /// often as desired.
    Unlimited,

    /// Apply rate limit of `max_per_epoch` events in any given epoch.
    #[serde(untagged, rename_all = "kebab-case")]
    Limited { max_per_epoch: u32 },
}

impl EpochRateLimit {
    /// Default rate limiting for the `sendCertificate` call.
    pub const fn send_certificate_default() -> Self {
        Self::limited(1)
    }

    /// Create a time-based rate limiting
    pub const fn limited(max_per_epoch: u32) -> Self {
        Self::Limited { max_per_epoch }
    }
}

/// Rate limiting override for each endpoint
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
struct RateLimitOverride {
    send_tx: Option<TimeRateLimit>,
    send_certificate: Option<EpochRateLimit>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(try_from = "BTreeMap<String, RateLimitOverride>")]
pub struct PerNetworkRateLimitOverride(BTreeMap<NetworkId, RateLimitOverride>);

impl PerNetworkRateLimitOverride {
    pub const fn new() -> Self {
        Self(BTreeMap::new())
    }
}

impl TryFrom<BTreeMap<String, RateLimitOverride>> for PerNetworkRateLimitOverride {
    type Error = <NetworkId as std::str::FromStr>::Err;

    fn try_from(overrides: BTreeMap<String, RateLimitOverride>) -> Result<Self, Self::Error> {
        overrides
            .into_iter()
            .map(|(k, v)| Ok((k.parse::<NetworkId>()?, v)))
            .collect::<Result<Self, Self::Error>>()
    }
}

impl FromIterator<(NetworkId, RateLimitOverride)> for PerNetworkRateLimitOverride {
    fn from_iter<T: IntoIterator<Item = (NetworkId, RateLimitOverride)>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

/// Rate limiting configuration for a single network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkRateLimitingConfig<'a> {
    /// Rate limit for `sendTx` for given network.
    pub send_tx: &'a TimeRateLimit,

    /// Rate limit for `sendCertificate` for given network.
    pub send_certificate: &'a EpochRateLimit,
}

/// Full rate limiting config.
/// Contains the defaults and the per-network overrides.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct RateLimitingConfig {
    /// Settlement rate limiting for the `sendTx` call.
    #[serde(default = "TimeRateLimit::send_tx_default")]
    send_tx: TimeRateLimit,

    /// Settlement rate limiting for the `sendCertificate` call.
    #[serde(default = "EpochRateLimit::send_certificate_default")]
    send_certificate: EpochRateLimit,

    /// Per-network rate limiting overrides.
    #[serde(default)]
    network: PerNetworkRateLimitOverride,
}

impl RateLimitingConfig {
    /// Default rate limiting configuration.
    pub const DEFAULT: Self = Self::new(
        TimeRateLimit::send_tx_default(),
        EpochRateLimit::send_certificate_default(),
    );

    /// New rate limiting config with no network-specific settings.
    pub const fn new(send_tx: TimeRateLimit, send_certificate: EpochRateLimit) -> Self {
        Self {
            send_tx,
            send_certificate,
            network: PerNetworkRateLimitOverride::new(),
        }
    }

    /// Override `sendTx`setting for given network.
    pub fn with_send_tx_override(mut self, nid: NetworkId, limit: TimeRateLimit) -> Self {
        self.network.0.entry(nid).or_default().send_tx = Some(limit);
        self
    }

    /// Get rate limiting configuration for given network.
    pub fn config_for(&self, network_id: NetworkId) -> NetworkRateLimitingConfig {
        let overrides = self.override_for(network_id);
        let send_tx = overrides
            .and_then(|l| l.send_tx.as_ref())
            .unwrap_or(&self.send_tx);
        let send_certificate = overrides
            .and_then(|l| l.send_certificate.as_ref())
            .unwrap_or(&self.send_certificate);
        NetworkRateLimitingConfig {
            send_tx,
            send_certificate,
        }
    }

    fn override_for(&self, nid: NetworkId) -> Option<&RateLimitOverride> {
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
        let config_str = "send-tx = \"unlimited\"\n\
            \n\
            [send-certificate]\n\
            max-per-epoch = 1\n";
        let parsed_default_config: RateLimitingConfig = toml::from_str(config_str).unwrap();
        assert_eq!(parsed_default_config, RateLimitingConfig::DEFAULT);

        let empty_config: RateLimitingConfig = toml::from_str("").unwrap();
        assert_eq!(empty_config, RateLimitingConfig::DEFAULT);
    }

    #[rstest::rstest]
    #[case(4, "1h 20min", 80 * 60)]
    #[case(2, "30min", 1800)]
    #[case(50, "90s", 90)]
    #[case(0, "2min", 120)]
    fn basic_send_tx(#[case] limit: u32, #[case] interval_str: String, #[case] interval_secs: u64) {
        #[rustfmt::skip]
        let config_str = format!(
            "[send-tx]\n\
            max-per-interval = {limit}\n\
            time-interval = {interval_str:?}\n"
        );
        let config: RateLimitingConfig = toml::from_str(&config_str).unwrap();
        let expected = RateLimitingConfig::new(
            TimeRateLimit::Limited {
                max_per_interval: limit,
                time_interval: Duration::from_secs(interval_secs),
            },
            EpochRateLimit::send_certificate_default(),
        );
        assert_eq!(config, expected);
    }

    #[test]
    fn top_level_and_override() {
        #[rustfmt::skip]
        let config_str = "[send-tx]\n\
            max-per-interval = 3\n\
            time-interval = \"30min\"\n\
            \n\
            [send-certificate]\n\
            max-per-epoch = 7\n\
            \n\
            [network.1.send-tx]\n\
            max-per-interval = 4\n\
            time-interval = \"40min\"\n\
            \n\
            [network.1.send-certificate]\n\
            max-per-epoch = 12\n";
        let config: RateLimitingConfig = toml::from_str(&config_str).unwrap();

        let default_send_tx_limit = TimeRateLimit::limited(3, Duration::from_secs(1800));
        let default_send_cert_limit = EpochRateLimit::limited(7);
        let network_1_send_tx_limit = TimeRateLimit::limited(4, Duration::from_secs(2400));
        let network_1_send_cert_limit = EpochRateLimit::limited(12);
        let network_1_override = RateLimitOverride {
            send_tx: Some(network_1_send_tx_limit.clone()),
            send_certificate: Some(network_1_send_cert_limit.clone()),
        };

        let expected = RateLimitingConfig {
            send_tx: default_send_tx_limit.clone(),
            send_certificate: default_send_cert_limit.clone(),
            network: PerNetworkRateLimitOverride::from_iter([(1, network_1_override)]),
        };

        assert_eq!(config, expected);

        assert_eq!(config.config_for(1).send_tx, &network_1_send_tx_limit);
        assert_eq!(
            config.config_for(1).send_certificate,
            &network_1_send_cert_limit
        );

        assert_eq!(config.config_for(2).send_tx, &default_send_tx_limit);
        assert_eq!(
            config.config_for(2).send_certificate,
            &default_send_cert_limit
        );

        assert_eq!(
            config.config_for(1337).send_certificate,
            &default_send_cert_limit
        );
    }
}
