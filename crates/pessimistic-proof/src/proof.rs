use thiserror::Error;

use crate::{
    bridge_exit::NetworkId, keccak::Digest, local_exit_tree::hasher::Keccak256Hasher,
    local_state::LocalNetworkState, multi_batch_header::MultiBatchHeader,
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
    #[error("detected debt for the network {network:?}")]
    HasDebt { network: NetworkId },
    #[error("Invalid imported exits root")]
    InvalidImportedExitsRoot,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Invalid message origin network")]
    InvalidMessageOriginNetwork,
    #[error("Invalid ETH network")]
    InvalidEthNetwork,
    #[error("Invalid imported bridge exit network")]
    InvalidImportedBridgeExitNetwork,
}

pub type ExitRoot = Digest;
pub type BalanceRoot = Digest;
pub type NullifierRoot = Digest;
pub type LeafProofOutput = (ExitRoot, BalanceRoot, NullifierRoot);

/// Proves that the given [`MultiBatchHeader`] can be applied on the given [`LocalNetworkState`].
pub fn generate_leaf_proof(
    initial_network_state: LocalNetworkState,
    batch_header: &MultiBatchHeader<Keccak256Hasher>,
) -> Result<LeafProofOutput, ProofError> {
    let mut network_state = initial_network_state;

    network_state.apply_batch_header(batch_header)?;

    Ok((
        batch_header.new_local_exit_root,
        batch_header.new_balance_root,
        batch_header.new_nullifier_root,
    ))
}
