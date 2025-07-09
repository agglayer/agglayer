/// ELF of the pessimistic proof program
const ELF: &[u8] = pessimistic_proof::ELF;

mod certifier;
mod proof;
mod settlement_client;

pub use certifier::CertifierClient;
pub use settlement_client::RpcSettlementClient;
