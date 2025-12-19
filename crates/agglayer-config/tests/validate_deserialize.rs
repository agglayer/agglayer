use std::path::Path;

use agglayer_config::Config;
use insta::assert_toml_snapshot;
use pretty_assertions::assert_eq;

#[test]
fn empty_rpcs() {
    let input = "./tests/fixtures/valide_config/empty_rpcs.toml";

    let config = Config::try_load(Path::new(input)).unwrap();

    assert_toml_snapshot!(config, {
        ".storage.*" => agglayer_config::redact_storage_path(),
    });
}

#[test]
fn max_rpc_request_size() {
    let input = "./tests/fixtures/valide_config/max_rpc_request_size.toml";

    let config = Config::try_load(Path::new(input)).unwrap();

    assert_toml_snapshot!(config, {
        ".storage.*" => agglayer_config::redact_storage_path(),
    });

    assert_eq!(config.rpc.max_request_body_size, 100 * 1024 * 1024);
}

#[test]
fn grpc_max_decoding_message_size() {
    let input = "./tests/fixtures/valide_config/grpc_max_decoding_message_size.toml";

    let config = Config::try_load(Path::new(input)).unwrap();

    assert_toml_snapshot!(config, {
        ".storage.*" => agglayer_config::redact_storage_path(),
    });

    assert_eq!(config.grpc.max_decoding_message_size, 100 * 1024 * 1024);
}
