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
    assert_eq!(config.rpc.settle_tx.max_retries, 5);
    assert_eq!(config.rpc.settle_tx.retry_interval, Duration::from_secs(10));
    assert_eq!(config.rpc.settle_tx.confirmations, 3);

    assert_eq!(config.rpc.settle_tx.gas_multiplier_factor, 120);
    assert_eq!(
        config.rpc.settle_tx.gas_price.multiplier,
        Multiplier::try_from(1.5).unwrap()
    );
    assert_eq!(config.rpc.settle_tx.gas_price.floor, 1_000_000_000);
    assert_eq!(config.rpc.settle_tx.gas_price.ceiling, 123_000_000_000);

    assert_eq!(config.rpc.settle_cert.max_retries, 15);
    assert_eq!(
        config.rpc.settle_cert.retry_interval,
        Duration::from_secs(110)
    );
    assert_eq!(config.rpc.settle_cert.confirmations, 13);

    assert_eq!(config.rpc.settle_cert.gas_multiplier_factor, 1120);
    assert_eq!(
        config.rpc.settle_cert.gas_price.multiplier,
        Multiplier::try_from(11.5).unwrap()
    );
    assert_eq!(config.rpc.settle_cert.gas_price.floor, 11_000_000_000);
    assert_eq!(config.rpc.settle_cert.gas_price.ceiling, 1_123_000_000_000);

    assert_toml_snapshot!(config);
}
