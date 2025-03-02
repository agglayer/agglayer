use vergen_git2::{Emitter, Git2Builder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Emitter::new()
        .add_instructions(
            &Git2Builder::default()
                .describe(true, true, None)
                .commit_timestamp(true)
                .build()?,
        )?
        .emit()?;
    Ok(())
}
