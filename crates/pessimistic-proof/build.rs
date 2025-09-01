pub fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    agglayer_elf_build::build_program("crates/pessimistic-proof-program")?;
    Ok(())
}
