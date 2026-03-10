use std::path::Path;

use agglayer_config::{AuthConfig, Config};
use insta::assert_toml_snapshot;

#[test]
fn auth_legacy() {
    let input = "./tests/fixtures/auth/legacy.toml";

    let config = Config::try_load(Path::new(input)).unwrap();

    let AuthConfig::GcpKms(gkms) = &config.auth else {
        panic!("Expected GKMS config to be present");
    };

    assert_eq!(gkms.project_id, Some("project-id_test".into()));
    assert_eq!(gkms.location, Some("location-test".into()));
    assert_eq!(gkms.keyring, Some("keyring-test".into()));
    assert_eq!(gkms.pp_settlement_key_name, Some("key-name-test".into()));
    assert_eq!(gkms.pp_settlement_key_version, Some(2));
    assert_eq!(gkms.tx_settlement_key_name, Some("key-name-test".into()));
    assert_eq!(gkms.tx_settlement_key_version, Some(2));

    assert_toml_snapshot!(config, {
        ".storage.*" => agglayer_config::redact_storage_path(),
    });
}

#[test]
fn auth_transition() {
    let input = "./tests/fixtures/auth/transition.toml";

    let config = Config::try_load(Path::new(input)).unwrap();

    let AuthConfig::GcpKms(gkms) = &config.auth else {
        panic!("Expected GKMS config to be present");
    };

    assert_eq!(gkms.project_id, Some("project-id_test".into()));
    assert_eq!(gkms.location, Some("location-test".into()));
    assert_eq!(gkms.keyring, Some("keyring-test".into()));
    assert_eq!(
        gkms.pp_settlement_key_name,
        Some("pp-settlement-key-name-test".into())
    );
    assert_eq!(gkms.pp_settlement_key_version, Some(3));
    assert_eq!(
        gkms.tx_settlement_key_name,
        Some("tx-settlement-key-name-test".into())
    );
    assert_eq!(gkms.tx_settlement_key_version, Some(4));

    assert_toml_snapshot!(config, {
        ".storage.*" => agglayer_config::redact_storage_path(),
    });
}

#[test]
fn auth_update() {
    let input = "./tests/fixtures/auth/update.toml";

    let config = Config::try_load(Path::new(input)).unwrap();

    let AuthConfig::GcpKms(gkms) = &config.auth else {
        panic!("Expected GKMS config to be present");
    };

    assert_eq!(gkms.project_id, Some("project-id_test".into()));
    assert_eq!(gkms.location, Some("location-test".into()));
    assert_eq!(gkms.keyring, Some("keyring-test".into()));
    assert_eq!(
        gkms.pp_settlement_key_name,
        Some("pp-settlement-key-name-test".into())
    );
    assert_eq!(gkms.pp_settlement_key_version, Some(3));
    assert_eq!(
        gkms.tx_settlement_key_name,
        Some("tx-settlement-key-name-test".into())
    );
    assert_eq!(gkms.tx_settlement_key_version, Some(4));

    assert_toml_snapshot!(config, {
        ".storage.*" => agglayer_config::redact_storage_path(),
    });
}

#[test]
fn auth_distinct_pp_and_tx_settlement_keys_are_preserved() {
    let input = "./tests/fixtures/auth/distinct_settlement_keys.toml";

    let config = Config::try_load(Path::new(input)).unwrap();

    let AuthConfig::GcpKms(gkms) = &config.auth else {
        panic!("Expected GCP KMS config to be present");
    };

    assert_eq!(
        gkms.pp_settlement_key_name,
        Some("pp-distinct-key-name".into())
    );
    assert_eq!(gkms.pp_settlement_key_version, Some(11));

    assert_eq!(
        gkms.tx_settlement_key_name,
        Some("tx-distinct-key-name".into())
    );
    assert_eq!(gkms.tx_settlement_key_version, Some(22));

    assert_toml_snapshot!(config, {
        ".storage.*" => agglayer_config::redact_storage_path(),
    });
}
