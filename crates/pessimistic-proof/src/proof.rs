use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    bridge_exit::{NetworkId, TokenInfo},
    keccak::{keccak256_combine, Digest},
    local_exit_tree::hasher::Keccak256Hasher,
    local_state::LocalNetworkState,
    multi_batch_header::MultiBatchHeader,
};

/// Represents all errors that can occur while generating the proof.
#[derive(Error, Debug)]
pub enum ProofError {
    #[error("Invalid initial local exit root. Got: {got:?}, Expected: {expected:?}")]
    InvalidInitialLocalExitRoot { got: Digest, expected: Digest },
    #[error("Invalid final local exit root.")]
    InvalidFinalLocalExitRoot,
    #[error("Invalid initial balance root.")]
    InvalidInitialBalanceRoot,
    #[error("Invalid final balance root.")]
    InvalidFinalBalanceRoot,
    #[error("Invalid initial nullifier root.")]
    InvalidInitialNullifierRoot,
    #[error("Invalid final nullifier root.")]
    InvalidFinalNullifierRoot,
    #[error("Invalid imported bridge exit merkle path.")]
    InvalidImportedBridgeExitMerklePath,
    #[error("Invalid imported bridge exit root.")]
    InvalidImportedBridgeExitRoot,
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
    #[error("Invalid message origin network.")]
    InvalidMessageOriginNetwork,
    #[error("Invalid ETH network.")]
    InvalidEthNetwork,
    #[error("Invalid imported bridge exit network.")]
    InvalidImportedBridgeExitNetwork,
    #[error("Mismatch between the global index and the inclusion proof.")]
    MismatchGlobalIndexInclusionProof,
    #[error("Duplicate token {0:?} in balance proofs")]
    DuplicateTokenBalanceProof(TokenInfo),
}

pub type ExitRoot = Digest;
pub type BalanceRoot = Digest;
pub type NullifierRoot = Digest;

/// Outputs of the pessimistic proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PessimisticProofOutput {
    /// The previous local exit root.
    pub prev_local_exit_root: Digest,
    /// The previous pessimistic root.
    pub prev_pessimistic_root: Digest,
    /// The global exit root against which we prove the inclusion of the
    /// imported bridge exits.
    pub selected_ger: Digest,
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

const PESSIMISTIC_CONSENSUS_TYPE: u32 = 0;

/// Proves that the given [`MultiBatchHeader`] can be applied on the given
/// [`LocalNetworkState`].
pub fn generate_pessimistic_proof(
    initial_network_state: LocalNetworkState,
    batch_header: &MultiBatchHeader<Keccak256Hasher>,
) -> Result<PessimisticProofOutput, ProofError> {
    let (prev_ler, prev_lbr, prev_nr) = initial_network_state.roots();
    let prev_pessimistic_root = keccak256_combine([prev_lbr, prev_nr]);

    let consensus_hash = keccak256_combine([
        &PESSIMISTIC_CONSENSUS_TYPE.to_be_bytes(),
        batch_header.signer.as_slice(),
    ]);

    let selected_ger = keccak256_combine([
        batch_header.imported_mainnet_exit_root,
        batch_header.imported_rollup_exit_root,
    ]);

    let new_pessimistic_root = keccak256_combine([
        batch_header.new_balance_root,
        batch_header.new_nullifier_root,
    ]);

    let mut network_state = initial_network_state;
    network_state.apply_batch_header(batch_header)?;

    Ok(PessimisticProofOutput {
        prev_local_exit_root: prev_ler,
        prev_pessimistic_root,
        selected_ger,
        origin_network: batch_header.origin_network,
        consensus_hash,
        new_local_exit_root: batch_header.new_local_exit_root,
        new_pessimistic_root,
    })
}
