/// ELF of the pessimistic proof program
const ELF: &[u8] = pessimistic_proof::ELF;

mod certifier;
mod settlement_client;

#[cfg(any(test, feature = "testutils"))]
pub mod testutils;

pub use certifier::CertifierClient;
pub use settlement_client::RpcSettlementClient;
