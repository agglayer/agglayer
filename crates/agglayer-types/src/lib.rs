pub use agglayer_interop_types::{aggchain_proof, bincode, NetworkId};
pub use agglayer_primitives::{self as primitives, Address, Digest, Signature, B256, U256, U512};
use agglayer_tries::roots::LocalExitRoot;
pub use pessimistic_proof::proof::Proof;

pub mod aggchain_data;

mod certificate;
mod epoch;
mod error;
mod local_network_state;
pub mod network_info;
mod proof_modes;
mod rpc_error_code;
mod settlement;

#[cfg(feature = "testutils")]
pub mod testutils {
    pub use agglayer_sp1::testutils::{EMPTY_ELF, EMPTY_ELF_V5};

    pub use crate::certificate::{compute_signature_info, dummy_sp1_stark_proof_with_version};
}
pub use certificate::{
    Certificate, CertificateHeader, CertificateId, CertificateIndex, CertificateStatus, Height,
    Metadata, SettlementTxHash,
};
pub use epoch::{EpochConfiguration, EpochNumber};
pub use error::{CertificateStatusError, Error, SignerError};
pub use local_network_state::{L1WitnessCtx, LocalNetworkStateData, PessimisticRootInput};
pub use network_info::{NetworkInfo, NetworkStatus, NetworkType, SettledClaim};
pub use proof_modes::{ExecutionMode, GenerationType};
pub use rpc_error_code::RpcErrorCode;
pub use settlement::{
    ClientError, ClientErrorType, ContractCallOutcome, ContractCallResult, Nonce,
    SettlementAttempt, SettlementAttemptNumber, SettlementAttemptResult, SettlementJob,
    SettlementJobId, SettlementJobResult,
};
