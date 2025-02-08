/// ELF of the pessimistic proof program
const ELF: &[u8] =
    include_bytes!("../../pessimistic-proof-program/elf/riscv32im-succinct-zkvm-elf");

mod certifier;
mod packer;
mod proof;

pub use certifier::CertifierClient;
pub use packer::EpochPackerClient;
