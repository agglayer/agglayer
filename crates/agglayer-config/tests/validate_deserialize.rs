use std::path::Path;

use agglayer_config::{assert_toml_snapshot, Config};
use pretty_assertions::assert_eq;

#[test]
fn removed_sendtx_options_are_rejected() {
    // `full-node-rpcs`, `l2` and `rate-limiting` only configured the removed
    // `interop_sendTx` flow. Their removal is a breaking change: configs
    // still carrying them are rejected at load time.
    let input = "./tests/fixtures/valide_config/removed_sendtx_options.toml";

    let error = Config::try_load(Path::new(input)).unwrap_err();

    assert!(
        error.to_string().contains("unknown field"),
        "unexpected error: {error}"
    );
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
