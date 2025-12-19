use std::path::Path;

use agglayer_config::Config;
use insta::assert_toml_snapshot;

#[test]
fn backup_enabled() {
    let input = "./tests/fixtures/valide_config/backup_enabled.toml";

    let config = Config::try_load(Path::new(input)).unwrap();

    assert_toml_snapshot!(config, {
        ".storage.*" => agglayer_config::redact_storage_path(),
    });
}
