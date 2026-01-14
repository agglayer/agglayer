use std::time::Duration;

use agglayer_config::{
    settlement_service::{SettlementConfig, SettlementPolicy, SettlementTransactionConfig},
    Multiplier,
};
use agglayer_primitives::U256;
use insta::assert_toml_snapshot;

#[test]
fn deserialize_default_settlement_tx_config() {
    let input = "./tests/fixtures/settlement/default.toml";
    let content = std::fs::read_to_string(input).unwrap();
    let config: SettlementTransactionConfig = toml::from_str(&content).unwrap();

    // Assert default values
    assert_eq!(config.max_expected_retries, 16384);
    assert_eq!(
        config.retry_on_transient_failure.initial_interval,
        Duration::from_secs(10)
    );
    assert_eq!(config.settlement_policy, SettlementPolicy::SafeBlock);
    assert_eq!(config.settlement_policy.confirmations(), None);
    assert_eq!(config.gas_limit_ceiling, U256::from(60_000_000_u64));
    assert_eq!(config.gas_price_ceiling, 100_000_000_000_u128);
    assert_eq!(config.gas_price_floor, 0);
    assert_eq!(config.gas_limit_multiplier_factor, Multiplier::default());
    assert_eq!(config.gas_price_multiplier_factor, Multiplier::default());

    assert_toml_snapshot!(config);
}

#[test]
fn deserialize_custom_config_1() {
    let input = "./tests/fixtures/settlement/custom_tx_config_1.toml";
    let content = std::fs::read_to_string(input).unwrap();
    let config: SettlementTransactionConfig = toml::from_str(&content).unwrap();

    // Assert custom certificate values
    assert_eq!(config.max_expected_retries, 2048);
    assert_eq!(
        config.retry_on_transient_failure.initial_interval,
        Duration::from_secs(15)
    );
    assert_eq!(config.settlement_policy, SettlementPolicy::FinalizedBlock);
    assert_eq!(config.settlement_policy.confirmations(), None);
    assert_eq!(config.gas_limit_ceiling, U256::from(100_000_000_u64));
    assert_eq!(config.gas_price_ceiling, 200_000_000_000_u128);
    assert_eq!(config.gas_price_floor, 5_000_000_000_u128);

    // Assert multipliers
    assert_eq!(config.gas_limit_multiplier_factor.as_f64(), 1.1);
    assert_eq!(config.gas_price_multiplier_factor.as_f64(), 1.2);

    assert_toml_snapshot!(config);
}

#[test]
fn deserialize_custom_config_2() {
    let input = "./tests/fixtures/settlement/custom_tx_config_2.toml";
    let content = std::fs::read_to_string(input).unwrap();
    let config: SettlementTransactionConfig = toml::from_str(&content).unwrap();

    // Assert custom validium values
    assert_eq!(config.max_expected_retries, 512);
    assert_eq!(
        config.retry_on_transient_failure.initial_interval,
        Duration::from_secs(5)
    );
    assert_eq!(
        config.settlement_policy,
        SettlementPolicy::LatestBlock { confirmations: 16 }
    );
    assert_eq!(config.settlement_policy.confirmations(), Some(16));
    assert_eq!(config.gas_limit_ceiling, U256::from(30_000_000_u64));
    assert_eq!(config.gas_price_ceiling, 50_000_000_000_u128);
    assert_eq!(config.gas_price_floor, 1_000_000_000_u128);

    // Assert multipliers
    assert_eq!(config.gas_limit_multiplier_factor.as_f64(), 1.05);
    assert_eq!(config.gas_price_multiplier_factor.as_f64(), 1.1);

    assert_toml_snapshot!(config);
}

#[test]
fn deserialize_full_settlement_config() {
    let input = "./tests/fixtures/settlement/full_settlement_config.toml";
    let content = std::fs::read_to_string(input).unwrap();
    let config: SettlementConfig = toml::from_str(&content).unwrap();

    // Assert certificate config
    assert_eq!(
        config.pessimistic_proof_tx_config.max_expected_retries,
        2048
    );
    assert_eq!(
        config
            .pessimistic_proof_tx_config
            .retry_on_transient_failure
            .initial_interval,
        Duration::from_secs(15)
    );
    assert_eq!(
        config.pessimistic_proof_tx_config.settlement_policy,
        SettlementPolicy::FinalizedBlock
    );
    assert_eq!(
        config
            .pessimistic_proof_tx_config
            .settlement_policy
            .confirmations(),
        None
    );
    assert_eq!(
        config.pessimistic_proof_tx_config.gas_limit_ceiling,
        U256::from(100_000_000_u64)
    );
    assert_eq!(
        config.pessimistic_proof_tx_config.gas_price_ceiling,
        200_000_000_000_u128
    );

    // Assert certificate multipliers
    assert_eq!(
        config
            .pessimistic_proof_tx_config
            .gas_limit_multiplier_factor
            .as_f64(),
        1.1
    );
    assert_eq!(
        config
            .pessimistic_proof_tx_config
            .gas_price_multiplier_factor
            .as_f64(),
        1.2
    );

    // Assert validium config
    assert_eq!(config.validium_tx_config.max_expected_retries, 512);
    assert_eq!(
        config
            .validium_tx_config
            .retry_on_transient_failure
            .initial_interval,
        Duration::from_secs(5)
    );
    assert_eq!(
        config.validium_tx_config.settlement_policy,
        SettlementPolicy::LatestBlock { confirmations: 16 }
    );
    assert_eq!(
        config.validium_tx_config.settlement_policy.confirmations(),
        Some(16)
    );
    assert_eq!(
        config.validium_tx_config.gas_limit_ceiling,
        U256::from(30_000_000_u64)
    );
    assert_eq!(
        config.validium_tx_config.gas_price_floor,
        2_000_000_000_u128
    );
    assert_eq!(
        config.validium_tx_config.gas_price_ceiling,
        50_000_000_000_u128
    );

    // Assert validium multipliers
    assert_eq!(
        config
            .validium_tx_config
            .gas_limit_multiplier_factor
            .as_f64(),
        1.05
    );

    assert_toml_snapshot!(config);
}

#[test]
fn test_finality_immediate() {
    // Test with explicit confirmations
    let toml = r#"
        [settlement-policy.latest-block]
        confirmations = 10
    "#;

    let config: SettlementTransactionConfig = toml::from_str(toml).unwrap();

    assert_eq!(
        config.settlement_policy,
        SettlementPolicy::LatestBlock { confirmations: 10 }
    );
    assert_eq!(config.settlement_policy.confirmations(), Some(10));
}

#[test]
fn test_finality_immediate_default_confirmations() {
    // Test with default confirmations (when not specified)
    let toml = r#"
        settlement-policy = { latest-block = {} }
    "#;

    let config: SettlementTransactionConfig = toml::from_str(toml).unwrap();

    assert_eq!(
        config.settlement_policy,
        SettlementPolicy::LatestBlock { confirmations: 0 }
    );
    assert_eq!(config.settlement_policy.confirmations(), Some(0));
}

#[test]
fn test_finality_immediate_compact_syntax() {
    // Test the compact "latest-block/N" syntax
    let toml = r#"
        settlement-policy = "latest-block/10"
    "#;

    let config: SettlementTransactionConfig = toml::from_str(toml).unwrap();

    assert_eq!(
        config.settlement_policy,
        SettlementPolicy::LatestBlock { confirmations: 10 }
    );
    assert_eq!(config.settlement_policy.confirmations(), Some(10));
}

#[test]
fn test_finality_immediate_compact_syntax_default() {
    // Test "latest-block" without confirmations uses default
    let toml = r#"
        settlement-policy = "latest-block"
    "#;

    let config: SettlementTransactionConfig = toml::from_str(toml).unwrap();

    assert_eq!(
        config.settlement_policy,
        SettlementPolicy::LatestBlock { confirmations: 0 }
    );
    assert_eq!(config.settlement_policy.confirmations(), Some(0));
}

#[test]
fn test_settlement_policy_safe() {
    let input = "./tests/fixtures/settlement/settlement_policy_safe.toml";
    let content = std::fs::read_to_string(input).unwrap();
    let config: SettlementTransactionConfig = toml::from_str(&content).unwrap();

    assert_eq!(config.settlement_policy, SettlementPolicy::SafeBlock);
    assert_eq!(config.settlement_policy.confirmations(), None);

    assert_toml_snapshot!(config);
}

#[test]
fn test_finality_finalized() {
    let toml = r#"
        settlement-policy = "finalized-block"
    "#;

    let config: SettlementTransactionConfig = toml::from_str(toml).unwrap();

    assert_eq!(config.settlement_policy, SettlementPolicy::FinalizedBlock);
}

#[test]
fn test_finality_default_is_justified() {
    let toml = r#""#;

    let config: SettlementTransactionConfig = toml::from_str(toml).unwrap();

    assert_eq!(config.settlement_policy, SettlementPolicy::SafeBlock);
}

#[test]
fn test_settlement_transaction_config_defaults() {
    let config = SettlementTransactionConfig::default();

    // Test retry configuration
    assert_eq!(config.max_expected_retries, 16384);
    assert_eq!(
        config.retry_on_transient_failure.initial_interval,
        Duration::from_secs(10)
    );

    // Test finality - SafeBlock is the default, no confirmations needed
    assert_eq!(config.settlement_policy, SettlementPolicy::SafeBlock);
    assert_eq!(config.settlement_policy.confirmations(), None);

    // Test gas configuration
    assert_eq!(config.gas_limit_multiplier_factor.as_f64(), 1.0);
    assert_eq!(config.gas_limit_ceiling, U256::from(60_000_000_u64));
    assert_eq!(config.gas_price_multiplier_factor.as_f64(), 1.0);
    assert_eq!(config.gas_price_floor, 0_u128);
    assert_eq!(config.gas_price_ceiling, 100_000_000_000_u128); // 100 gwei

    assert_toml_snapshot!(config);
}

#[test]
fn test_gas_price_with_units() {
    let toml = r#"
        gas-price-floor = "5gwei"
        gas-price-ceiling = "200gwei"
    "#;

    let config: SettlementTransactionConfig = toml::from_str(toml).unwrap();

    assert_eq!(config.gas_price_floor, 5_000_000_000_u128);
    assert_eq!(config.gas_price_ceiling, 200_000_000_000_u128);
}

#[test]
fn test_human_duration_formats() {
    let toml = r#"
        retry-on-transient-failure.initial-interval = "30s"
    "#;

    let config: SettlementTransactionConfig = toml::from_str(toml).unwrap();
    assert_eq!(
        config.retry_on_transient_failure.initial_interval,
        Duration::from_secs(30)
    );
}
