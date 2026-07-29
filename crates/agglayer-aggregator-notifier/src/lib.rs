/// ELF of the pessimistic proof program
const ELF: &[u8] = pessimistic_proof::ELF;

mod certifier;

pub use certifier::CertifierClient;
