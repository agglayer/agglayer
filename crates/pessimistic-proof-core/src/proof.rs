use std::collections::{BTreeMap, BTreeSet};

use agglayer_bincode as bincode;
use agglayer_primitives::{Address, Digest};
use agglayer_tries::roots::LocalExitRoot;
use hex_literal::hex;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use unified_bridge::{
    Error, GlobalIndex, ImportedBridgeExitCommitmentValues, ImportedBridgeExitCommitmentVersion,
    LocalExitTreeError, NetworkId, TokenInfo,
};

use crate::{
    aggchain_data::MultisigError,
    local_state::{
        commitment::{PessimisticRootCommitmentValues, StateCommitment},
        NetworkState,
    },
    multi_batch_header::MultiBatchHeader,
};

/// Refers to the commitment on the imported bridge exits involved in the
/// aggchain proof public values (`commit_imported_bridge_exits` field).
/// This constant defines which commitment version is expected to verify the
/// aggchain proof.
pub const IMPORTED_BRIDGE_EXIT_COMMITMENT_VERSION: ImportedBridgeExitCommitmentVersion =
    ImportedBridgeExitCommitmentVersion::V3;

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

    /// Invalid multisig
    #[error("Invalid multisig")]
    InvalidMultisig(#[source] MultisigError),
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
    /// The list of pre-confirmed LERs per origin network.
    pub preconfirmed_lers: BTreeMap<NetworkId, BTreeSet<LocalExitRoot>>, // todo: hash
}

impl PessimisticProofOutput {
    pub fn bincode_codec() -> bincode::Codec<impl bincode::Options> {
        bincode::contracts()
    }
}

pub const EMPTY_LER: LocalExitRoot = LocalExitRoot::new(Digest(hex!(
    "27ae5ba08d7291c96c8cbddcc148bf48a6d68c7974b94356f53754ef6171d757"
)));

pub const EMPTY_PP_ROOT_V2: Digest = Digest(hex!(
    "c89c9c0f2ebd19afa9e5910097c43e56fb4aff3a06ddee8d7c9bae09bc769184"
));

/// Represents all the enforced values for the stark and for the signed
/// commitments.
#[derive(Clone)]
pub struct ConstrainedValues {
    pub initial_state_commitment: StateCommitment,
    pub final_state_commitment: StateCommitment,
    pub prev_pessimistic_root: Digest,
    pub height: u64,
    pub origin_network: NetworkId,
    pub l1_info_root: Digest,
    pub commit_imported_bridge_exits: ImportedBridgeExitCommitmentValues,
    pub certificate_id: Digest,
}

impl ConstrainedValues {
    fn new(
        batch_header: &MultiBatchHeader,
        initial_state_commitment: &StateCommitment,
        final_state_commitment: &StateCommitment,
    ) -> Self {
        Self {
            initial_state_commitment: initial_state_commitment.clone(),
            final_state_commitment: final_state_commitment.clone(),
            height: batch_header.height,
            origin_network: batch_header.origin_network,
            l1_info_root: batch_header.l1_info_root,
            commit_imported_bridge_exits: batch_header.commit_imported_bridge_exits(),
            certificate_id: batch_header.certificate_id,
            prev_pessimistic_root: batch_header.prev_pessimistic_root,
        }
    }
}

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

    let constrained_values = ConstrainedValues::new(
        batch_header,
        &initial_state_commitment,
        &final_state_commitment,
    );

    // Verify multisig, aggchain proof, or both.
    let target_pp_root_version = batch_header.aggchain_data.verify(constrained_values)?;

    let height = batch_header
        .height
        .checked_add(1)
        .ok_or(ProofError::HeightOverflow)?;

    let new_pessimistic_root = PessimisticRootCommitmentValues {
        balance_root: final_state_commitment.balance_root,
        nullifier_root: final_state_commitment.nullifier_root,
        ler_leaf_count: final_state_commitment.ler_leaf_count,
        height,
        origin_network: batch_header.origin_network,
    }
    .compute_pp_root(target_pp_root_version);

    let preconfirmed_lers = {
        let ilers: Vec<_> = batch_header
            .imported_bridge_exits
            .iter()
            .filter_map(|(ib, _)| ib.preconfirmed_ler())
            .collect();

        let mut imported_ler_per_origin: BTreeMap<NetworkId, BTreeSet<LocalExitRoot>> =
            BTreeMap::new();
        for (origin_network, imported_ler) in ilers {
            imported_ler_per_origin
                .entry(origin_network)
                .or_default()
                .insert(imported_ler.into());
        }

        imported_ler_per_origin
    };

    Ok((
        PessimisticProofOutput {
            prev_local_exit_root: zero_if_empty_local_exit_root(initial_state_commitment.exit_root),
            prev_pessimistic_root: batch_header.prev_pessimistic_root,
            l1_info_root: batch_header.l1_info_root,
            origin_network: batch_header.origin_network,
            aggchain_hash: batch_header.aggchain_data.aggchain_hash(),
            new_local_exit_root: zero_if_empty_local_exit_root(final_state_commitment.exit_root),
            new_pessimistic_root,
            preconfirmed_lers,
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
