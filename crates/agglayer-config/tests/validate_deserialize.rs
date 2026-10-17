use std::path::Path;

use agglayer_config::{assert_toml_snapshot, Config};
use pretty_assertions::assert_eq;

#[test]
fn empty_rpcs() {
    let input = "./tests/fixtures/valide_config/empty_rpcs.toml";

    let config = Config::try_load(Path::new(input)).unwrap();

    assert!(config.succinct_cluster.is_empty());

    assert_toml_snapshot!(config);
}

#[test]
fn max_rpc_request_size() {
    let input = "./tests/fixtures/valide_config/max_rpc_request_size.toml";

    let config = Config::try_load(Path::new(input)).unwrap();

    assert_toml_snapshot!(config);

    assert_eq!(config.rpc.max_request_body_size, 100 * 1024 * 1024);
}

#[test]
fn grpc_max_decoding_message_size() {
    let input = "./tests/fixtures/valide_config/grpc_max_decoding_message_size.toml";

    let config = Config::try_load(Path::new(input)).unwrap();

    assert_toml_snapshot!(config);

    assert_eq!(config.grpc.max_decoding_message_size, 100 * 1024 * 1024);
}

#[test]
fn succinct_cluster_config_round_trips() -> Result<(), Box<dyn std::error::Error>> {
    let config: Config = toml::from_str(
        r#"
        [succinct-cluster]
        48 = "https://private.example/"
        "#,
    )?;
    assert!(!config.succinct_cluster.is_empty());
    assert_eq!(
        config
            .succinct_cluster
            .get(&48)
            .map(|endpoint| endpoint.as_str()),
        Some("https://private.example/")
    );
    assert_eq!(
        toml::from_str::<Config>(&toml::to_string(&config)?)?,
        config
    );
    Ok(())
}

#[test]
fn succinct_cluster_rejects_invalid_entries() {
    for entry in [
        r#"unknown = "https://cluster.example/""#,
        r#"-1 = "https://cluster.example/""#,
        r#"4294967296 = "https://cluster.example/""#,
        r#"48 = "not-a-url""#,
        r#"default = "https://default.example/""#,
    ] {
        assert!(
            toml::from_str::<Config>(&format!("[succinct-cluster]\n{entry}")).is_err(),
            "{entry}"
        );
    }
}
