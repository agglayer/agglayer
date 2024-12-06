use std::path::Path;

use agglayer_config::Config;
use insta::assert_toml_snapshot;
use pretty_assertions::assert_eq;

#[test]
fn empty_rpcs() {
    let input = "./tests/fixtures/valide_config/empty_rpcs.toml";

    let config = Config::try_load(Path::new(input)).unwrap();

    assert_toml_snapshot!(config, {
        ".storage.*" => insta::dynamic_redaction(|value, path| {
            if path.to_string() != "storage.db-path" {
                if let insta::internals::Content::String(path) = value {
                    return insta::internals::Content::String(path.replace(Path::new("./").canonicalize().unwrap().to_str().unwrap(), "/tmp/agglayer"));
                }
            }

            value
        }),
    });
}

#[test]
fn max_rpc_request_size() {
    let input = "./tests/fixtures/valide_config/max_rpc_request_size.toml";

    let config = Config::try_load(Path::new(input)).unwrap();

    assert_toml_snapshot!(config, {
        ".storage.*" => insta::dynamic_redaction(|value, path| {
            if path.to_string() != "storage.db-path" {
                if let insta::internals::Content::String(path) = value {
                    return insta::internals::Content::String(path.replace(Path::new("./").canonicalize().unwrap().to_str().unwrap(), "/tmp/agglayer"));
                }
            }

            value
        }),
    });

    assert_eq!(config.rpc.max_request_body_size, 100 * 1024 * 1024);
}
