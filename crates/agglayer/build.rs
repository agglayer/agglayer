use eyre::eyre;
use vergen_git2::{Emitter, Git2Builder};

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    // Rebuild `version()` when the build-time version overrides change. These
    // are supplied by the Docker image build, where no `.git` is available.
    println!("cargo:rerun-if-env-changed=AGGLAYER_BUILD_DESCRIBE");
    println!("cargo:rerun-if-env-changed=AGGLAYER_BUILD_TIMESTAMP");

    Emitter::new()
        .add_instructions(
            &Git2Builder::default()
                .describe(true, true, None)
                .commit_timestamp(true)
                .build()?,
        )
        .map_err(|e| eyre!(e))?
        .emit()
        .map_err(|e| eyre!(e))?;
    Ok(())
}
