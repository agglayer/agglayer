#![allow(clippy::too_many_arguments)]
use std::{collections::BTreeMap, hash::Hash};

use agglayer_primitives::U256;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;

use crate::{
    aggchain_proof::AggchainData,
    bridge_exit::{BridgeExit, NetworkId, TokenInfo},
    global_index::GlobalIndex,
    imported_bridge_exit::{commit_imported_bridge_exits, ImportedBridgeExit},
    keccak::{digest::Digest, keccak256_combine},
    local_balance_tree::LocalBalancePath,
    local_exit_tree::hasher::Hasher,
    local_state::StateCommitment,
    nullifier_tree::NullifierPath,
};

/// Represents the chain state transition for the pessimistic proof.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultiBatchHeader<H>
where
    H: Hasher,
    H::Digest: Eq + Hash + Copy + Serialize + DeserializeOwned,
{
    /// Network that emitted this [`MultiBatchHeader`].
    pub origin_network: NetworkId,
    /// Previous local exit root.
    #[serde_as(as = "_")]
    pub prev_local_exit_root: H::Digest,
    /// Previous local balance root.
    #[serde_as(as = "_")]
    pub prev_balance_root: H::Digest,
    /// Previous nullifier tree root.
    #[serde_as(as = "_")]
    pub prev_nullifier_root: H::Digest,
    /// List of bridge exits created in this batch.
    pub bridge_exits: Vec<BridgeExit>,
    /// List of imported bridge exits claimed in this batch.
    pub imported_bridge_exits: Vec<(ImportedBridgeExit, NullifierPath<H>)>,
    /// Commitment to the imported bridge exits. None if zero imported bridge
    /// exit.
    #[serde_as(as = "Option<_>")]
    pub imported_exits_root: Option<H::Digest>,
    /// L1 info root used to import bridge exits.
    #[serde_as(as = "_")]
    pub l1_info_root: H::Digest,
    /// Token balances of the origin network before processing bridge events,
    /// with Merkle proofs of these balances in the local balance tree.
    pub balances_proofs: BTreeMap<TokenInfo, (U256, LocalBalancePath<H>)>,
    /// State commitment target hashes.
    pub target: StateCommitment,
    /// Aggchain proof.
    pub aggchain_proof: AggchainData,
}

pub fn signature_commitment(
    new_local_exit_root: Digest,
    imported_bridge_exits: impl Iterator<Item = GlobalIndex>,
) -> Digest {
    let imported_hash = commit_imported_bridge_exits(imported_bridge_exits);
    keccak256_combine([new_local_exit_root.as_slice(), imported_hash.as_slice()])
}
