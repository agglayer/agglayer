use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;
use nu_ansi_term::Color::Red;

#[derive(Debug, clap::Parser)]
pub struct Elf {}

const PACKAGE_NAME: &str = "pessimistic-proof-program";
const EXPECTED_RUSTUP_TOOLCHAIN: &str = "succinct";

impl Elf {
    pub fn run(&self) -> Result<()> {
        let mut command = Command::new(which::which("rustup")?);
        let result = command.args(["toolchain", "list"]).output()?;

        let output = String::from_utf8(result.stdout).unwrap();
        if !output.contains(EXPECTED_RUSTUP_TOOLCHAIN) {
            eprintln!(
                "{} {} {}",
                Red.paint("You need to install the"),
                Red.bold().paint(EXPECTED_RUSTUP_TOOLCHAIN),
                Red.paint("toolchain before trying to generate elf")
            );
            std::process::exit(1);
        }

        eprintln!("Generating ELF...");
        let metadata_cmd = cargo_metadata::MetadataCommand::new();
        let metadata = metadata_cmd.exec().unwrap();

        let mut command = Command::new(which::which("cargo")?);

        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../crates/")
            .join(PACKAGE_NAME);

        let build_target = "riscv32im-succinct-zkvm-elf";
        let rust_flags = [
            "-C",
            "passes=loweratomic",
            "-C",
            "link-arg=-Ttext=0x00200800",
            "-C",
            "panic=abort",
        ];

        let cargo_args = vec!["build", "--release", "--target", build_target, "--locked"];

        let result = command
            .env("RUSTUP_TOOLCHAIN", "succinct")
            .env("CARGO_ENCODED_RUSTFLAGS", rust_flags.join("\x1f"))
            .args(&cargo_args)
            .current_dir(path);

        let result = result.status()?;
        anyhow::ensure!(result.success(), "cargo command failed");

        let elf_path = metadata
            .target_directory
            .join(build_target)
            .join("release")
            .join(PACKAGE_NAME);

        let elf_dir = metadata.target_directory.parent().unwrap().join("elf");
        fs::create_dir_all(&elf_dir)?;
        let result_elf_path = elf_dir.join("riscv32im-succinct-zkvm-elf");
        fs::copy(elf_path, result_elf_path)?;

        Ok(())
    }
}
