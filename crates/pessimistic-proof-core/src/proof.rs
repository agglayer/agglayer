use agglayer_primitives::keccak::Keccak256Hasher;
use agglayer_primitives::{digest::Digest, Address};
pub use bincode::Options;
use hex_literal::hex;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use unified_bridge::global_index::GlobalIndex;
use unified_bridge::imported_bridge_exit;
use unified_bridge::{
    bridge_exit::NetworkId, local_exit_tree::LocalExitTreeError, token_info::TokenInfo,
};

use crate::{
    local_state::{commitment::PessimisticRoot, NetworkState},
    multi_batch_header::MultiBatchHeader,
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
    /// The previous local exit root declared by the chain does not match the
    /// one computed by the prover.
    #[error("Invalid previous local exit root. declared: {declared}, computed: {computed}")]
    InvalidPreviousLocalExitRoot { declared: Digest, computed: Digest },
    /// The previous balance root declared by the agglayer does not match the
    /// one computed by the prover.
    #[error("Invalid previous balance root. declared: {declared}, computed: {computed}")]
    InvalidPreviousBalanceRoot { declared: Digest, computed: Digest },
    /// The previous nullifier root declared by the agglayer does not match the
    /// one computed by the prover.
    #[error("Invalid previous nullifier root. declared: {declared}, computed: {computed}")]
    InvalidPreviousNullifierRoot { declared: Digest, computed: Digest },
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
    /// The new local exit root declared by the chain does not match the
    /// one computed by the prover.
    #[error("Invalid new local exit root. declared: {declared}, computed: {computed}")]
    InvalidNewLocalExitRoot { declared: Digest, computed: Digest },
    /// The new balance root declared by the agglayer does not match the
    /// one computed by the prover.
    #[error("Invalid new balance root. declared: {declared}, computed: {computed}")]
    InvalidNewBalanceRoot { declared: Digest, computed: Digest },
    /// The new nullifier root declared by the agglayer does not match the
    /// one computed by the prover.
    #[error("Invalid new nullifier root. declared: {declared}, computed: {computed}")]
    InvalidNewNullifierRoot { declared: Digest, computed: Digest },
    /// The provided imported bridge exit is invalid.
    #[error("Invalid imported bridge exit. global index: {global_index:?}, error: {source}")]
    InvalidImportedBridgeExit {
        source: imported_bridge_exit::Error,
        global_index: GlobalIndex,
    },
    /// The commitment to the list of imported bridge exits is invalid.
    #[error(
        "Invalid commitment on the imported bridge exits. declared: {declared}, computed: \
         {computed}"
    )]
    InvalidImportedExitsRoot { declared: Digest, computed: Digest },
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
    /// The signature is on a payload that is with an inconsistent version.
    #[error("Inconsistent signed payload version.")]
    InconsistentSignedPayload,
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
    /// Height overflow.
    #[error("Height overflow")]
    HeightOverflow,
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
    /// The aggchain hash.
    pub aggchain_hash: Digest,
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

pub const EMPTY_LER: Digest = Digest(hex!(
    "27ae5ba08d7291c96c8cbddcc148bf48a6d68c7974b94356f53754ef6171d757"
));

pub const EMPTY_PP_ROOT_V2: Digest = Digest(hex!(
    "c89c9c0f2ebd19afa9e5910097c43e56fb4aff3a06ddee8d7c9bae09bc769184"
));

/// Proves that the given [`MultiBatchHeader`] can be applied on the given
/// [`NetworkState`].
pub fn generate_pessimistic_proof(
    initial_network_state: NetworkState,
    batch_header: &MultiBatchHeader<Keccak256Hasher>,
) -> Result<PessimisticProofOutput, ProofError> {
    let target_pp_root_version = initial_network_state.verify_consensus(batch_header)?;

    let new_pessimistic_root = {
        let mut network_state = initial_network_state;
        let computed_target = network_state.apply_batch_header(batch_header)?;

        if computed_target.exit_root != batch_header.target.exit_root {
            return Err(ProofError::InvalidNewLocalExitRoot {
                declared: batch_header.target.exit_root,
                computed: computed_target.exit_root,
            });
        }

        if computed_target.balance_root != batch_header.target.balance_root {
            return Err(ProofError::InvalidNewBalanceRoot {
                declared: batch_header.target.balance_root,
                computed: computed_target.balance_root,
            });
        }

        if computed_target.nullifier_root != batch_header.target.nullifier_root {
            return Err(ProofError::InvalidNewNullifierRoot {
                declared: batch_header.target.nullifier_root,
                computed: computed_target.nullifier_root,
            });
        }

        let Some(height) = batch_header.height.checked_add(1) else {
            return Err(ProofError::HeightOverflow);
        };

        PessimisticRoot {
            balance_root: network_state.balance_tree.root,
            nullifier_root: network_state.nullifier_tree.root,
            ler_leaf_count: network_state.exit_tree.leaf_count,
            height,
            origin_network: batch_header.origin_network,
        }
        .compute_pp_root(target_pp_root_version)
    };

    // NOTE: Hack to comply with the L1 contracts which assume `0x00..00` for the
    // empty roots of the different trees involved. Therefore, we do
    // one mapping of empty tree hash <> 0x00..0 on the public inputs.
    let prev_local_exit_root = if batch_header.prev_local_exit_root == EMPTY_LER {
        Digest::default()
    } else {
        batch_header.prev_local_exit_root
    };

    Ok(PessimisticProofOutput {
        prev_local_exit_root,
        prev_pessimistic_root: batch_header.prev_pessimistic_root,
        l1_info_root: batch_header.l1_info_root,
        origin_network: batch_header.origin_network,
        aggchain_hash: batch_header.aggchain_proof.aggchain_hash(),
        new_local_exit_root: batch_header.target.exit_root,
        new_pessimistic_root,
    })
}
