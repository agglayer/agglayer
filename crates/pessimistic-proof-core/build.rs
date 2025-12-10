use std::{env, fs, path::Path};

use semver::Version;
use toml::Value;

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    let cargo_toml_path = Path::new("../pessimistic-proof-program/Cargo.toml");
    println!("cargo:rerun-if-changed={}", cargo_toml_path.display());
    let cargo_toml = fs::read_to_string(cargo_toml_path).expect("Failed to read Cargo.toml");
    let parsed_toml: Value = toml::from_str(&cargo_toml).expect("Failed to parse Cargo.toml");

    let version: Version = parsed_toml
        .get("package")
        .and_then(|pkg| pkg.get("version"))
        .and_then(|v| {
            v.as_str()
                .map(Version::parse)
                .transpose()
                .expect("Unable to extract version")
        })
        .expect("Unable to extract version");

    let major_version = version.major.to_string();

    let dest_path = Path::new(&env::var_os("OUT_DIR").expect("OUT_DIR not set")).join("version.rs");
    fs::write(
        &dest_path,
        format!("pub const PESSIMISTIC_PROOF_PROGRAM_VERSION: u32 = {major_version};\n"),
    )
    .expect("Failed to write pessimistic-proof-core version.rs");
}
