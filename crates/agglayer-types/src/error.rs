use agglayer_interop_types::LocalExitRoot;
use agglayer_primitives::SignatureError;
use agglayer_tries::error::SmtError;
use pessimistic_proof::{error::ProofVerificationError, ProofError};
use serde::{Deserialize, Serialize};
use unified_bridge::{GlobalIndex, LocalExitTreeError, NetworkId, TokenInfo};

use crate::{Digest, GenerationType};

#[derive(Debug, thiserror::Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename = "agglayer_types::Error")]
pub enum Error {
    /// The imported bridge exits should refer to one and the same L1 info root.
    #[error("Imported bridge exits refer to multiple L1 info root")]
    MultipleL1InfoRoot,
    /// The certificate refers to a new local exit root which differ from the
    /// one computed by the agglayer.
    #[error(
        "Mismatch on the certificate new local exit root. declared: {declared:?}, computed: \
         {computed:?}"
    )]
    MismatchNewLocalExitRoot {
        computed: LocalExitRoot,
        declared: LocalExitRoot,
    },
    /// The given token balance cannot overflow.
    #[error("Token balance cannot overflow. token: {0:?}")]
    BalanceOverflow(TokenInfo),
    /// The given token balance cannot be negative.
    #[error("Token balance cannot be negative. token: {0:?}")]
    BalanceUnderflow(TokenInfo),
    /// The balance proof for the given token cannot be generated.
    #[error("Unable to generate the balance proof. token: {token:?}, error: {source}")]
    BalanceProofGenerationFailed { source: SmtError, token: TokenInfo },
    /// The nullifier path for the given imported bridge exit cannot be
    /// generated.
    #[error(
        "Unable to generate the nullifier path. global_index: {global_index:?}, error: {source}"
    )]
    NullifierPathGenerationFailed {
        source: SmtError,
        global_index: GlobalIndex,
    },
    /// The operation cannot be applied on the local exit tree.
    #[error(transparent)]
    InvalidLocalExitTreeOperation(#[from] LocalExitTreeError),
    #[error(
        "Incorrect L1 Info Root for the leaf count {leaf_count}. declared: {declared}, retrieved \
         from L1: {retrieved}"
    )]
    /// Invalid or unsettled L1 Info Root
    L1InfoRootIncorrect {
        leaf_count: u32,
        declared: Digest,
        retrieved: Digest,
    },
    #[error(
        "Incorrect declared L1 Info Tree information: l1_leaf: {l1_leaf:?}, l1_root: \
         {l1_info_root:?}"
    )]
    InconsistentL1InfoTreeInformation {
        l1_leaf: Option<u32>,
        l1_info_root: Option<Digest>,
    },
    /// The operation cannot be applied on the smt.
    #[error(transparent)]
    InvalidSmtOperation(#[from] SmtError),

    /// Inconsistent GERs
    #[error("Inconsistent GER")]
    InconsistentGlobalExitRoot,

    #[error("AggchainVkey missing")]
    MissingAggchainVkey,

    #[error(
        "Invalid custom chain data length expected at least {expected_at_least}, actual {actual}"
    )]
    InvalidCustomChainDataLength {
        expected_at_least: usize,
        actual: usize,
    },

    /// The certificate refers to a prev local exit root which differ from the
    /// one computed by the agglayer.
    #[error(
        "Mismatch on the certificate prev local exit root. declared: {declared:?}, computed: \
         {computed:?}"
    )]
    MismatchPrevLocalExitRoot {
        computed: LocalExitRoot,
        declared: LocalExitRoot,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, thiserror::Error, PartialEq, Eq)]
pub enum CertificateStatusError {
    /// Failure on the pessimistic proof execution, either natively or in the
    /// prover.
    #[error("({generation_type}) proof generation error: {}", source.to_string())]
    ProofGenerationError {
        generation_type: GenerationType,
        source: ProofError,
    },

    /// Failure on the proof verification.
    #[error("Proof verification failed")]
    ProofVerificationFailed(#[source] ProofVerificationError),

    /// Failure on the pessimistic proof witness generation from the
    /// [`LocalNetworkStateData`] and the provided [`Certificate`].
    #[error("Cannot produce local network state from certificate")]
    TypeConversionError(#[source] Error),

    #[error("Trusted sequencer address not found for network: {0}")]
    TrustedSequencerNotFound(NetworkId),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Settlement error: {0}")]
    SettlementError(String),

    #[error("Pre certification error: {0}")]
    PreCertificationError(String),

    #[error("Certification error: {0}")]
    CertificationError(String),

    #[error("L1 Info root not found for l1 leaf count: {0}")]
    L1InfoRootNotFound(u32),

    #[error("Last pessimistic root not found for network: {0}")]
    LastPessimisticRootNotFound(NetworkId),
}

#[derive(Debug, thiserror::Error)]
pub enum SignerError {
    #[error("Signature not provided")]
    Missing,

    #[error("Signature recovery error")]
    Recovery(#[source] SignatureError),
}
