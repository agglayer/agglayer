use agglayer_elf_build::ProgramBuilder;

const SP1_GUEST_DOCKER_TAG: &str =
    "v6.2.2@sha256:af8c2f46bb5e417c1c2fb939cd4397c75aa4f2439a9fd7c1f9ce2e0a693f6f6b";

pub fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    ProgramBuilder::new("crates/pessimistic-proof-program")?
        .docker_tag(SP1_GUEST_DOCKER_TAG)
        .run()?;
    Ok(())
}
