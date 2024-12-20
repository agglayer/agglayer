use vergen_git2::{BuildBuilder, Emitter, Git2Builder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Emitter::new()
        .add_instructions(&BuildBuilder::default().build_timestamp(true).build()?)?
        .add_instructions(&Git2Builder::default().describe(true, true, None).build()?)?
        .emit()?;
    Ok(())
}
