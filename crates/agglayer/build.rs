use eyre::eyre;
use vergen_git2::{Emitter, Git2Builder};

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
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
