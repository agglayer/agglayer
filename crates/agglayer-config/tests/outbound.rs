use std::time::Duration;

use agglayer_config::{outbound::OutboundConfig, Multiplier};
use insta::assert_toml_snapshot;

#[test]
fn deserialize_default_outbound_config() {
    let input = "./tests/fixtures/outbound/default.toml";
    let content = std::fs::read_to_string(input).unwrap();
    let config: OutboundConfig = toml::from_str(&content).unwrap();

    // Assert config matches default
    assert_eq!(config, OutboundConfig::default());

    assert_toml_snapshot!(config);
}

#[test]
fn deserialize_custom_outbound_config() {
    let input = "./tests/fixtures/outbound/custom.toml";
    let content = std::fs::read_to_string(input).unwrap();
    let config: OutboundConfig = toml::from_str(&content).unwrap();

    // Assert custom values
    assert_eq!(config.rpc.settle.max_retries, 5);
    assert_eq!(config.rpc.settle.retry_interval, Duration::from_secs(10));
    assert_eq!(config.rpc.settle.confirmations, 3);
    assert_eq!(
        config.rpc.settle.settlement_timeout,
        Duration::from_secs(30 * 60)
    );
    assert_eq!(config.rpc.settle.gas_multiplier_factor, 120);
    assert_eq!(
        config.rpc.settle.gas_price.multiplier,
        Multiplier::try_from(1.5).unwrap()
    );
    assert_eq!(config.rpc.settle.gas_price.floor, 1_000_000_000);
    assert_eq!(config.rpc.settle.gas_price.ceiling, 100_000_000_000);

    assert_toml_snapshot!(config);
}
