use agglayer_primitives::{Address, Digest, Signature, B256};
use agglayer_tries::roots::LocalExitRoot;
use bytemuck::{Pod, Zeroable};
use hex_literal::hex;
use serde::{Deserialize, Serialize};
use thiserror::Error;
#[cfg(not(target_os = "zkvm"))]
use tracing::warn;
#[cfg(target_os = "zkvm")]
use unified_bridge::AggchainProofPublicValues;
use unified_bridge::{
    CommitmentVersion, Error, GlobalIndex, ImportedBridgeExitCommitmentValues, LocalExitTreeError,
    NetworkId, TokenInfo,
};

use crate::{
    aggchain_proof::AggchainData,
    local_state::{
        commitment::{PessimisticRoot, SignatureCommitmentValues, StateCommitment},
        NetworkState,
    },
    multi_batch_header::MultiBatchHeader,
};

/// Refers to the commitment on the imported bridge exits involved in the
/// aggchain proof public values (`commit_imported_bridge_exits` field).
/// This constant defines which commitment version is expected to verify the
/// aggchain proof.
pub const IMPORTED_BRIDGE_EXIT_COMMITMENT_VERSION: CommitmentVersion = CommitmentVersion::V3;

// Compile-time size assertion for PessimisticProofOutputZeroCopy
const _PESSIMISTIC_PROOF_OUTPUT_SIZE: () = {
    // 6 * 32-byte fields + 1 u32 field = 192 + 4 = 196 bytes
    assert!(std::mem::size_of::<PessimisticProofOutputZeroCopy>() == 196);
    assert!(std::mem::align_of::<PessimisticProofOutputZeroCopy>() == 4); // u32
                                                                          // alignment
};

/// Represents all errors that can occur while generating the proof.
///
/// Several commitments are declared either by the chains (e.g., the local exit
/// root) or by the agglayer (e.g., the balance and nullifier root), and are
/// later re-computed by the prover to ensure that they match the witness data.
/// Consequently, several errors highlight a mismatch between what is *declared*
/// as witness and what is *computed* by the prover.
#[derive(Clone, Error, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProofError {
    // Note: The following arms are no longer generated but may be present in
    //       storage having been produced by an alder version of the node.
    #[error("Invalid previous local exit root. declared: {declared}, computed: {computed}")]
    InvalidPreviousLocalExitRoot { declared: Digest, computed: Digest },
    #[error("Invalid previous balance root. declared: {declared}, computed: {computed}")]
    InvalidPreviousBalanceRoot { declared: Digest, computed: Digest },
    #[error("Invalid previous nullifier root. declared: {declared}, computed: {computed}")]
    InvalidPreviousNullifierRoot { declared: Digest, computed: Digest },
    #[error("Invalid new local exit root. declared: {declared}, computed: {computed}")]
    InvalidNewLocalExitRoot { declared: Digest, computed: Digest },
    #[error("Invalid new balance root. declared: {declared}, computed: {computed}")]
    InvalidNewBalanceRoot { declared: Digest, computed: Digest },
    #[error("Invalid new nullifier root. declared: {declared}, computed: {computed}")]
    InvalidNewNullifierRoot { declared: Digest, computed: Digest },

    /// The provided imported bridge exit is invalid.
    #[error("Invalid imported bridge exit. global index: {global_index:?}, error: {source}")]
    InvalidImportedBridgeExit {
        source: Error,
        global_index: GlobalIndex,
    },

    /// The commitment to the list of imported bridge exits is invalid.
    #[error(
        "Invalid commitment on the imported bridge exits. declared: {declared}, computed: \
         {computed}"
    )]
    InvalidImportedExitsRoot { declared: Digest, computed: Digest },

    // Note: No longer produced, present for storage compatibility.
    #[error("Mismatch between the imported bridge exits list and its commitment.")]
    MismatchImportedExitsRoot,

    /// The provided nullifier path is invalid.
    #[error("Invalid nullifier path.")]
    InvalidNullifierPath,

    /// The provided balance path is invalid.
    #[error("Invalid balance path.")]
    InvalidBalancePath,

    /// The imported bridge exit led to balance overflow.
    #[error("Balance overflow in bridge exit.")]
    BalanceOverflowInBridgeExit,

    /// The bridge exit led to balance underflow.
    #[error("Balance underflow in bridge exit.")]
    BalanceUnderflowInBridgeExit,

    /// The provided bridge exit goes to the sender's own network which is not
    /// permitted.
    #[error("Cannot perform bridge exit to the same network as the origin.")]
    CannotExitToSameNetwork,

    /// The provided bridge exit message is invalid.
    #[error("Invalid message origin network.")]
    InvalidMessageOriginNetwork,

    /// The token address is zero if and only if it refers to the L1 native eth.
    #[error("Invalid L1 TokenInfo. TokenInfo: {0:?}")]
    InvalidL1TokenInfo(TokenInfo),

    /// The provided token is missing a balance proof.
    #[error("Missing token balance proof. TokenInfo: {0:?}")]
    MissingTokenBalanceProof(TokenInfo),

    /// The provided token comes with multiple balance proofs.
    #[error("Duplicate token in balance proofs. TokenInfo: {0:?}")]
    DuplicateTokenBalanceProof(TokenInfo),

    /// The signature on the state transition is invalid.
    #[error("Invalid signature.")]
    InvalidSignature,

    /// The signer recovered from the signature differs from the one declared as
    /// witness.
    #[error("Invalid signer. declared: {declared}, recovered: {recovered}")]
    InvalidSigner {
        declared: Address,
        recovered: Address,
    },

    /// The operation cannot be applied on the local exit tree.
    #[error(transparent)]
    InvalidLocalExitTreeOperation(#[from] LocalExitTreeError),

    /// Unknown error.
    #[error("Unknown error: {0}")]
    Unknown(String),

    /// The previous pessimistic root is not re-computable.
    #[error(
        "Invalid previous pessimistic root. declared: {declared}, ppr_v2: {computed_v2}, ppr_v3: \
         {computed_v3}"
    )]
    InvalidPreviousPessimisticRoot {
        declared: Digest,
        computed_v2: Digest,
        computed_v3: Digest,
    },

    /// The signature is on a payload that is with an inconsistent version.
    #[error("Inconsistent signed payload version.")]
    InconsistentSignedPayload,

    /// Height overflow.
    #[error("Height overflow")]
    HeightOverflow,
}

/// Zero-copy representation of PessimisticProofOutput for bytemuck operations.
/// This struct has a stable C-compatible memory layout with fixed-size fields.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct PessimisticProofOutputZeroCopy {
    /// The previous local exit root (32 bytes)
    pub prev_local_exit_root: [u8; 32],
    /// The previous pessimistic root (32 bytes)
    pub prev_pessimistic_root: [u8; 32],
    /// The l1 info root (32 bytes)
    pub l1_info_root: [u8; 32],
    /// The aggchain hash (32 bytes)
    pub aggchain_hash: [u8; 32],
    /// The new local exit root (32 bytes)
    pub new_local_exit_root: [u8; 32],
    /// The new pessimistic root (32 bytes)
    pub new_pessimistic_root: [u8; 32],
    /// The origin network (u32)
    pub origin_network: u32,
}

impl From<&PessimisticProofOutput> for PessimisticProofOutputZeroCopy {
    fn from(output: &PessimisticProofOutput) -> Self {
        // Convert LocalExitRoot to Digest first, then extract bytes
        let prev_digest: Digest = output.prev_local_exit_root.into();
        let new_digest: Digest = output.new_local_exit_root.into();

        Self {
            prev_local_exit_root: prev_digest.0,
            prev_pessimistic_root: output.prev_pessimistic_root.0,
            l1_info_root: output.l1_info_root.0,
            aggchain_hash: output.aggchain_hash.0,
            new_local_exit_root: new_digest.0,
            new_pessimistic_root: output.new_pessimistic_root.0,
            origin_network: output.origin_network.to_u32(),
        }
    }
}

impl From<&PessimisticProofOutputZeroCopy> for PessimisticProofOutput {
    fn from(zc: &PessimisticProofOutputZeroCopy) -> Self {
        Self {
            prev_local_exit_root: LocalExitRoot::new(Digest(zc.prev_local_exit_root)),
            prev_pessimistic_root: Digest(zc.prev_pessimistic_root),
            l1_info_root: Digest(zc.l1_info_root),
            origin_network: NetworkId::new(zc.origin_network),
            aggchain_hash: Digest(zc.aggchain_hash),
            new_local_exit_root: LocalExitRoot::new(Digest(zc.new_local_exit_root)),
            new_pessimistic_root: Digest(zc.new_pessimistic_root),
        }
    }
}

/// Outputs of the pessimistic proof.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PessimisticProofOutput {
    /// The previous local exit root.
    pub prev_local_exit_root: LocalExitRoot,
    /// The previous pessimistic root.
    pub prev_pessimistic_root: Digest,
    /// The l1 info root against which we prove the inclusion of the imported
    /// bridge exits.
    pub l1_info_root: Digest,
    /// The origin network of the pessimistic proof.
    pub origin_network: NetworkId,
    /// The aggchain hash.
    pub aggchain_hash: Digest,
    /// The new local exit root.
    pub new_local_exit_root: LocalExitRoot,
    /// The new pessimistic root.
    pub new_pessimistic_root: Digest,
}

impl PessimisticProofOutput {
    /// Convert to zero-copy representation for efficient serialization.
    pub fn to_zero_copy(&self) -> PessimisticProofOutputZeroCopy {
        self.into()
    }

    /// Serialize to zero-copy bytes using bytemuck.
    /// This avoids the overhead of bincode encoding for better SP1 performance.
    pub fn to_bytes_zero_copy(&self) -> Vec<u8> {
        let zero_copy = self.to_zero_copy();
        bytemuck::bytes_of(&zero_copy).to_vec()
    }

    /// Deserialize from zero-copy bytes using bytemuck.
    pub fn from_bytes_zero_copy(data: &[u8]) -> Result<Self, bytemuck::PodCastError> {
        if data.len() != std::mem::size_of::<PessimisticProofOutputZeroCopy>() {
            return Err(bytemuck::PodCastError::SizeMismatch);
        }
        let zero_copy = bytemuck::try_from_bytes::<PessimisticProofOutputZeroCopy>(data)?;
        Ok(zero_copy.into())
    }
}

pub const EMPTY_LER: LocalExitRoot = LocalExitRoot::new(Digest(hex!(
    "27ae5ba08d7291c96c8cbddcc148bf48a6d68c7974b94356f53754ef6171d757"
)));

pub const EMPTY_PP_ROOT_V2: Digest = Digest(hex!(
    "c89c9c0f2ebd19afa9e5910097c43e56fb4aff3a06ddee8d7c9bae09bc769184"
));

/// Proves that the given [`MultiBatchHeader`] can be applied on the given
/// [`NetworkState`].
pub fn generate_pessimistic_proof(
    initial_network_state: NetworkState,
    batch_header: &MultiBatchHeader,
) -> Result<(PessimisticProofOutput, StateCommitment), ProofError> {
    // Get the initial state commitment
    let initial_state_commitment = initial_network_state.get_state_commitment();
    let mut network_state: NetworkState = initial_network_state;
    let final_state_commitment = network_state.apply_batch_header(batch_header)?;

    // verify the consensus
    let target_pp_root_version = verify_consensus(
        batch_header,
        &initial_state_commitment,
        &final_state_commitment,
    )?;

    let height = batch_header
        .height
        .checked_add(1)
        .ok_or(ProofError::HeightOverflow)?;

    let new_pessimistic_root = PessimisticRoot {
        balance_root: final_state_commitment.balance_root,
        nullifier_root: final_state_commitment.nullifier_root,
        ler_leaf_count: final_state_commitment.ler_leaf_count,
        height,
        origin_network: batch_header.origin_network,
    }
    .compute_pp_root(target_pp_root_version);

    Ok((
        PessimisticProofOutput {
            prev_local_exit_root: zero_if_empty_local_exit_root(initial_state_commitment.exit_root),
            prev_pessimistic_root: batch_header.prev_pessimistic_root,
            l1_info_root: batch_header.l1_info_root,
            origin_network: batch_header.origin_network,
            aggchain_hash: batch_header.aggchain_proof.aggchain_hash(),
            new_local_exit_root: zero_if_empty_local_exit_root(final_state_commitment.exit_root),
            new_pessimistic_root,
        },
        final_state_commitment,
    ))
}

// NOTE: Hack to comply with the L1 contracts which assume `0x00..00` for the
// empty roots of the different trees involved. Therefore, we do
// one mapping of empty tree hash <> 0x00..0 on the public inputs.
pub fn zero_if_empty_local_exit_root(root: LocalExitRoot) -> LocalExitRoot {
    if root == EMPTY_LER {
        LocalExitRoot::default()
    } else {
        root
    }
}

/// Verify the signature or aggchain proof
/// Returns the pp root version on success.
pub fn verify_consensus(
    multi_batch_header: &MultiBatchHeader,
    initial_state_commitment: &StateCommitment,
    final_state_commitment: &StateCommitment,
) -> Result<CommitmentVersion, ProofError> {
    // Verify initial state commitment and PP root matches
    let base_pp_root_version = PessimisticRoot {
        balance_root: initial_state_commitment.balance_root,
        nullifier_root: initial_state_commitment.nullifier_root,
        ler_leaf_count: initial_state_commitment.ler_leaf_count,
        height: multi_batch_header.height,
        origin_network: multi_batch_header.origin_network,
    }
    .infer_settled_pp_root_version(multi_batch_header.prev_pessimistic_root)?;

    let commit_imported_bridge_exits = ImportedBridgeExitCommitmentValues {
        claims: multi_batch_header
            .imported_bridge_exits
            .iter()
            .map(|(exit, _)| exit.to_indexed_exit_hash())
            .collect(),
    };

    // Verify the aggchain proof which can be either one signature or one sp1 proof.
    // NOTE: The STARK is verified exclusively within the SP1 VM.
    let target_pp_root_version = match &multi_batch_header.aggchain_proof {
        AggchainData::ECDSA { signer, signature } => {
            let verify_signature = |digest: Digest, signature: &Signature| {
                signature
                    .recover_address_from_prehash(&B256::new(digest.0))
                    .map_err(|_| ProofError::InvalidSignature)
            };

            let signature_commitment = SignatureCommitmentValues {
                new_local_exit_root: final_state_commitment.exit_root,
                commit_imported_bridge_exits,
                height: multi_batch_header.height,
            };

            let target_pp_root_version = {
                if signer
                    == &verify_signature(
                        signature_commitment.commitment(CommitmentVersion::V3),
                        signature,
                    )?
                {
                    CommitmentVersion::V3
                } else if signer
                    == &verify_signature(
                        signature_commitment.commitment(CommitmentVersion::V2),
                        signature,
                    )?
                {
                    CommitmentVersion::V2
                } else {
                    return Err(ProofError::InvalidSignature);
                }
            };

            match (base_pp_root_version, target_pp_root_version) {
                // From V2 to V2: OK
                (CommitmentVersion::V2, CommitmentVersion::V2) => {}
                // From V3 to V3: OK
                (CommitmentVersion::V3, CommitmentVersion::V3) => {}
                // From V2 to V3: OK (migration)
                (CommitmentVersion::V2, CommitmentVersion::V3) => {}
                // Inconsistent signed payload.
                _ => return Err(ProofError::InconsistentSignedPayload),
            }

            target_pp_root_version
        }
        #[cfg(not(target_os = "zkvm"))]
        AggchainData::Generic { .. } => {
            // NOTE: No stark verification in the native rust code due to
            // the sp1_zkvm::lib::verify::verify_sp1_proof syscall
            warn!("verify_sp1_proof is not callable outside of SP1");
            CommitmentVersion::V3
        }
        #[cfg(target_os = "zkvm")]
        AggchainData::Generic {
            aggchain_vkey,
            aggchain_params,
        } => {
            let aggchain_proof_public_values = AggchainProofPublicValues {
                prev_local_exit_root: initial_state_commitment.exit_root.into(),
                new_local_exit_root: final_state_commitment.exit_root.into(),
                l1_info_root: multi_batch_header.l1_info_root,
                origin_network: multi_batch_header.origin_network,
                aggchain_params: *aggchain_params,
                commit_imported_bridge_exits: commit_imported_bridge_exits
                    .commitment(IMPORTED_BRIDGE_EXIT_COMMITMENT_VERSION),
            };

            eprintln!("AP public values: {aggchain_proof_public_values:?}");

            sp1_zkvm::lib::verify::verify_sp1_proof(
                aggchain_vkey,
                &aggchain_proof_public_values.hash().into(),
            );

            CommitmentVersion::V3
        }
    };

    Ok(target_pp_root_version)
}
