pub fn main() -> agglayer_elf_build::Result<()> {
    agglayer_elf_build::build_program("crates/pessimistic-proof-program").map(drop)
}
