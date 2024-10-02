pub use bincode::Options;
use reth_primitives::Address;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    bridge_exit::{NetworkId, TokenInfo},
    imported_bridge_exit,
    keccak::Hash,
    keccak::{keccak256_combine, Digest},
    local_exit_tree::hasher::Keccak256Hasher,
    local_state::{LocalNetworkState, StateCommitment},
    multi_batch_header::MultiBatchHeader,
};

/// Several commitments are declared either by the chains (e.g., the local exit
/// root) or by the agglayer (e.g., the balance and nullifier root), and are
/// later re-computed by the prover to ensure that they match the witness data.
/// Consequently, several errors highlight a mismatch between what is *declared*
/// as witness and what is *computed* by the prover.
#[derive(Clone, Error, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProofError {
    /// The previous local exit root declared by the chain does not match the
    /// one computed by the prover.
    #[error("Invalid previous local exit root. declared: {declared}, computed: {computed}")]
    InvalidPreviousLocalExitRoot { declared: Hash, computed: Hash },
    /// The previous balance root declared by the agglayer does not match the
    /// one computed by the prover.
    #[error("Invalid previous balance root. declared: {declared}, computed: {computed}")]
    InvalidPreviousBalanceRoot { declared: Hash, computed: Hash },
    /// The previous nullifier root declared by the agglayer does not match the
    /// one computed by the prover.
    #[error("Invalid previous nullifier root. declared: {declared}, computed: {computed}")]
    InvalidPreviousNullifierRoot { declared: Hash, computed: Hash },
    /// The new local exit root declared by the chain does not match the
    /// one computed by the prover.
    #[error("Invalid new local exit root. declared: {declared}, computed: {computed}")]
    InvalidNewLocalExitRoot { declared: Hash, computed: Hash },
    /// The new balance root declared by the agglayer does not match the
    /// one computed by the prover.
    #[error("Invalid new balance root. declared: {declared}, computed: {computed}")]
    InvalidNewBalanceRoot { declared: Hash, computed: Hash },
    /// The new nullifier root declared by the agglayer does not match the
    /// one computed by the prover.
    #[error("Invalid new nullifier root. declared: {declared}, computed: {computed}")]
    InvalidNewNullifierRoot { declared: Hash, computed: Hash },
    #[error("Invalid imported bridge exit merkle path.")]
    InvalidImportedBridgeExitMerklePath,
    #[error("Invalid imported bridge exit root.")]
    InvalidImportedBridgeExitRoot,
    #[error("Invalid imported bridge exit: {0}")]
    InvalidImportedBridgeExit(#[from] imported_bridge_exit::Error),
    #[error("Missing token balance proof.")]
    MissingTokenBalanceProof,
    #[error("Invalid nullifier path.")]
    InvalidNullifierPath,
    #[error("Invalid balance path.")]
    InvalidBalancePath,
    #[error("Balance overflow in bridge exit.")]
    BalanceOverflowInBridgeExit,
    #[error("Balance underflow in bridge exit.")]
    BalanceUnderflowInBridgeExit,
    #[error("Exit to same network.")]
    ExitToSameNetwork,
    #[error("Invalid imported exits root.")]
    InvalidImportedExitsRoot,
    #[error("Invalid signature.")]
    InvalidSignature,
    #[error("Invalid signer. Witness: {witness:?}, Recovered: {recovered:?}")]
    InvalidSigner {
        witness: Address,
        recovered: Address,
    },
    #[error("Invalid message origin network.")]
    InvalidMessageOriginNetwork,
    #[error("Invalid ETH network.")]
    InvalidEthNetwork,
    #[error("Invalid imported bridge exit network.")]
    InvalidImportedBridgeExitNetwork,
    #[error("Duplicate token {0:?} in balance proofs")]
    DuplicateTokenBalanceProof(TokenInfo),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Outputs of the pessimistic proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PessimisticProofOutput {
    /// The previous local exit root.
    pub prev_local_exit_root: Digest,
    /// The previous pessimistic root.
    pub prev_pessimistic_root: Digest,
    /// The l1 info root against which we prove the inclusion of the imported
    /// bridge exits.
    pub l1_info_root: Digest,
    /// The origin network of the pessimistic proof.
    pub origin_network: NetworkId,
    /// The consensus hash.
    pub consensus_hash: Digest,
    /// The new local exit root.
    pub new_local_exit_root: Digest,
    /// The new pessimistic root which commits to the balance and nullifier
    /// tree.
    pub new_pessimistic_root: Digest,
}

impl PessimisticProofOutput {
    pub fn bincode_options() -> impl bincode::Options {
        bincode::DefaultOptions::new()
            .with_big_endian()
            .with_fixint_encoding()
    }
}

const PESSIMISTIC_CONSENSUS_TYPE: u32 = 0;

/// Proves that the given [`MultiBatchHeader`] can be applied on the given
/// [`LocalNetworkState`].
pub fn generate_pessimistic_proof(
    initial_network_state: LocalNetworkState,
    batch_header: &MultiBatchHeader<Keccak256Hasher>,
) -> Result<PessimisticProofOutput, ProofError> {
    let StateCommitment {
        exit_root: prev_ler,
        balance_root: prev_lbr,
        nullifier_root: prev_nr,
    } = initial_network_state.roots();
    let prev_pessimistic_root = keccak256_combine([prev_lbr, prev_nr]);

    let consensus_hash = keccak256_combine([
        &PESSIMISTIC_CONSENSUS_TYPE.to_be_bytes(),
        batch_header.signer.as_slice(),
    ]);

    let new_pessimistic_root = keccak256_combine([
        batch_header.target.balance_root,
        batch_header.target.nullifier_root,
    ]);

    let mut network_state = initial_network_state;
    let computed_target = network_state.apply_batch_header(batch_header)?;

    if computed_target.exit_root != batch_header.target.exit_root {
        return Err(ProofError::InvalidNewLocalExitRoot {
            declared: batch_header.target.exit_root.into(),
            computed: computed_target.exit_root.into(),
        });
    }
    if computed_target.balance_root != batch_header.target.balance_root {
        return Err(ProofError::InvalidNewBalanceRoot {
            declared: batch_header.target.balance_root.into(),
            computed: computed_target.balance_root.into(),
        });
    }
    if computed_target.nullifier_root != batch_header.target.nullifier_root {
        return Err(ProofError::InvalidNewNullifierRoot {
            declared: batch_header.target.nullifier_root.into(),
            computed: computed_target.nullifier_root.into(),
        });
    }

    Ok(PessimisticProofOutput {
        prev_local_exit_root: prev_ler,
        prev_pessimistic_root,
        l1_info_root: batch_header.l1_info_root,
        origin_network: batch_header.origin_network,
        consensus_hash,
        new_local_exit_root: batch_header.target.exit_root,
        new_pessimistic_root,
    })
}
