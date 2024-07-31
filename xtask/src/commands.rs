use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{bail, Context, Result};
use cargo_metadata::{camino::Utf8PathBuf, Metadata};
use nu_ansi_term::Color::Red;

/// This command generates an ELF file from the `pessimistic-proof-program`.
#[derive(Debug, clap::Parser)]
pub struct Elf {
    #[clap(skip = EXPECTED_RUSTUP_TOOLCHAIN)]
    toolchain: &'static str,
    #[clap(skip = PACKAGE_NAME)]
    package_name: &'static str,
    #[clap(skip = TARGET_DIR)]
    target_dir: &'static str,
}

const PACKAGE_NAME: &str = "pessimistic-proof-program";
const TARGET_DIR: &str = "elf";
const EXPECTED_RUSTUP_TOOLCHAIN: &str = "succinct";

impl Elf {
    fn path(&self) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../crates/")
            .join(self.package_name)
    }

    fn get_metadata(&self) -> Result<Metadata> {
        Ok(cargo_metadata::MetadataCommand::new()
            .manifest_path(self.path().join("./Cargo.toml"))
            .exec()?)
    }

    /// Run the Elf generation command.
    pub fn run(&self) -> Result<Utf8PathBuf> {
        let mut command = Command::new(which::which("rustup")?);
        let result = command.args(["toolchain", "list"]).output()?;

        let output = String::from_utf8(result.stdout).unwrap();
        if !output.contains(self.toolchain) {
            eprintln!(
                "{} {} {}",
                Red.paint("You need to install the"),
                Red.bold().paint(self.toolchain),
                Red.paint("toolchain before trying to generate elf")
            );

            bail!("Toolchain not found");
        }

        eprintln!("Generating ELF...");

        let path = self.path();

        let metadata = self.get_metadata()?;

        let mut command = Command::new(which::which("cargo")?);

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
            .env("RUSTUP_TOOLCHAIN", self.toolchain)
            .env("CARGO_ENCODED_RUSTFLAGS", rust_flags.join("\x1f"))
            .args(&cargo_args)
            .current_dir(&path);

        let Ok(result) = result.status() else {
            bail!(
                "Cargo command failed to execute with {} package",
                self.package_name
            );
        };

        anyhow::ensure!(result.success(), "cargo command failed");

        let elf_path = metadata
            .target_directory
            .join(build_target)
            .join("release")
            .join(self.package_name);

        let elf_dir = self.get_elf_dir(metadata)?;

        fs::create_dir_all(&elf_dir).with_context(|| "Failed to create ELF directory")?;
        let result_elf_path = elf_dir.join(build_target);
        fs::copy(elf_path, &result_elf_path)
            .with_context(|| format!("Failed to copy ELF file {}", result_elf_path))?;

        Ok(result_elf_path)
    }

    fn get_elf_dir(&self, metadata: Metadata) -> anyhow::Result<Utf8PathBuf> {
        Ok(metadata
            .target_directory
            .parent()
            .with_context(|| "Failed to get target directory")?
            .join(self.target_dir))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_command_with_wrong_toolchain() -> anyhow::Result<()> {
        assert!(Elf {
            toolchain: "wrong_toolchain",
            package_name: "",
            target_dir: "",
        }
        .run()
        .is_err());

        Ok(())
    }

    #[test]
    fn run_command_with_wrong_package() -> anyhow::Result<()> {
        assert!(Elf {
            toolchain: EXPECTED_RUSTUP_TOOLCHAIN,
            package_name: "wrong_package_name",
            target_dir: "",
        }
        .run()
        .is_err());

        Ok(())
    }

    #[test]
    fn run_command_successfully() -> anyhow::Result<()> {
        let target_dir = "run_command_successfully";
        let cmd = Elf {
            toolchain: EXPECTED_RUSTUP_TOOLCHAIN,
            package_name: PACKAGE_NAME,
            target_dir,
        };
        let result = cmd.run();
        assert!(result.is_ok());

        let result = result?;
        assert!(result.is_file());

        let metadata = cmd.get_metadata()?;

        std::fs::remove_dir_all(cmd.get_elf_dir(metadata)?)?;
        Ok(())
    }
}
