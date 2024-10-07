#![allow(clippy::too_many_arguments)]
use std::{borrow::Borrow, collections::BTreeMap, hash::Hash};

use reth_primitives::{Address, Signature, U256};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;

use crate::{
    bridge_exit::{BridgeExit, NetworkId, TokenInfo},
    imported_bridge_exit::{commit_imported_bridge_exits, ImportedBridgeExit},
    keccak::{keccak256_combine, Digest},
    local_balance_tree::LocalBalancePath,
    local_exit_tree::hasher::Hasher,
    local_state::StateCommitment,
    nullifier_tree::NullifierPath,
};

/// Represents the chain state transition for the pessimistic proof.
#[serde_as]
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
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
    /// Signer committing to the state transition.
    pub signer: Address,
    /// Signature committing to the state transition.
    pub signature: Signature,
    /// State commitment target hashes.
    pub target: StateCommitment,
}

impl<H> MultiBatchHeader<H>
where
    H: Hasher,
    H::Digest: Eq + Hash + Copy + Serialize + DeserializeOwned,
{
    /// Creates a new [`MultiBatchHeader`].
    pub fn new(
        origin_network: NetworkId,
        prev_local_exit_root: H::Digest,
        bridge_exits: Vec<BridgeExit>,
        imported_bridge_exits: Vec<(ImportedBridgeExit, NullifierPath<H>)>,
        imported_exits_root: Option<H::Digest>,
        balances_proofs: BTreeMap<TokenInfo, (U256, LocalBalancePath<H>)>,
        prev_balance_root: H::Digest,
        prev_nullifier_root: H::Digest,
        signer: Address,
        signature: Signature,
        target: StateCommitment,
        l1_info_root: H::Digest,
    ) -> Self {
        Self {
            origin_network,
            prev_local_exit_root,
            bridge_exits,
            imported_bridge_exits,
            imported_exits_root,
            balances_proofs,
            prev_balance_root,
            prev_nullifier_root,
            signer,
            signature,
            target,
            l1_info_root,
        }
    }
}

pub fn signature_commitment(
    new_local_exit_root: Digest,
    imported_bridge_exits: impl IntoIterator<Item: Borrow<ImportedBridgeExit>>,
) -> Digest {
    let imported_hash = commit_imported_bridge_exits(imported_bridge_exits.into_iter());
    keccak256_combine([new_local_exit_root.as_slice(), imported_hash.as_slice()])
}
