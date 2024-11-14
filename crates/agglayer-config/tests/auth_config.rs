use std::path::Path;

use agglayer_config::Config;
use insta::assert_toml_snapshot;

#[test]
fn auth_kebab_case() {
    let input = "./tests/fixtures/valide_config/auth_kebab_case.toml";

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
