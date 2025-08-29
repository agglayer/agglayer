pub fn main() -> eyre::Result<()> {
    agglayer_elf_build::build_program("crates/pessimistic-proof-program").map(drop)
}
