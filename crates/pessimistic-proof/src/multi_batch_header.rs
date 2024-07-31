#![allow(clippy::too_many_arguments)]
use std::{collections::BTreeMap, hash::Hash};

use reth_primitives::U256;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;

use crate::{
    bridge_exit::{BridgeExit, NetworkId, TokenInfo},
    imported_bridge_exit::ImportedBridgeExit,
    local_balance_tree::LocalBalancePath,
    local_exit_tree::hasher::Hasher,
    nullifier_tree::NullifierPath,
};

/// Represents the data submitted by the CDKs to the AggLayer.
///
/// The bridge exits plus the imported bridge exits define
/// the state transition, resp. the amount that goes out and the amount that comes in.
///
/// The bridge exits refer to the [`BridgeExit`]  emitted by
/// the origin network of the [`MultiBatchHeader`].
///
/// The imported bridge exits refer to the [`BridgeExit`] received and imported
/// by the origin network of the [`MultiBatchHeader].
#[serde_as]
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct MultiBatchHeader<H>
where
    H: Hasher,
    H::Digest: Eq + Hash + Copy + Serialize + DeserializeOwned,
{
    /// The origin network which emitted this BatchHeader.
    /// TODO: we should clarify naming. We use origin to refer to the issuing network of a token. Could consider using "sending" or "local" to refer to the network that created a batch.
    pub origin_network: NetworkId,

    /// The initial local exit root.
    #[serde_as(as = "_")]
    pub prev_local_exit_root: H::Digest,

    /// The new local exit root
    #[serde_as(as = "_")]
    pub new_local_exit_root: H::Digest,

    /// The set of bridge exits created in this batch
    /// TODO: move out of the header and into a separate struct
    pub bridge_exits: Vec<BridgeExit>,

    /// The set of imported bridge exits claimed in this batch
    /// TODO: move out of the header and into a separate struct
    pub imported_bridge_exits: Vec<(ImportedBridgeExit, NullifierPath<H>)>,

    /// A commitment to the set of imported bridge exits for which the origin network is the target.
    #[serde_as(as = "Option<_>")]
    pub imported_exits_root: Option<H::Digest>,

    /// The set of imported local exit roots
    // TODO: benchmark if BTreeMap is the best choice in terms of SP1 cycles
    pub imported_local_exit_roots: BTreeMap<NetworkId, H::Digest>,

    /// A map from token info to the token balance of the origin network before any bridge event is processed,
    /// along with the Merkle proof of this balance in the local balance tree.
    // TODO: benchmark if BTreeMap is the best choice in terms of SP1 cycles
    pub balances_proofs: BTreeMap<TokenInfo, (U256, LocalBalancePath<H>)>,

    /// The previous Local Balance Root
    #[serde_as(as = "_")]
    pub prev_balance_root: H::Digest,

    /// The new Local Balance Root
    #[serde_as(as = "_")]
    pub new_balance_root: H::Digest,

    /// The previous NullifierTree Root
    #[serde_as(as = "_")]
    pub prev_nullifier_root: H::Digest,

    /// The new NullifierTree Root
    #[serde_as(as = "_")]
    pub new_nullifier_root: H::Digest,
    // /// A validity proof verifying transaction execution
    //pub validity_proof: Option<ValidityProof>,

    // /// A consensus proof for the latest block
    //pub consensus_proof: Option<ConsensusProof>,

    // /// The signature that commits to the state transition.
    //pub signature: (),
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
        new_local_exit_root: H::Digest,
        bridge_exits: Vec<BridgeExit>,
        imported_bridge_exits: Vec<(ImportedBridgeExit, NullifierPath<H>)>,
        imported_exits_root: Option<H::Digest>,
        imported_local_exit_roots: BTreeMap<NetworkId, H::Digest>,
        balances_proofs: BTreeMap<TokenInfo, (U256, LocalBalancePath<H>)>,
        prev_balance_root: H::Digest,
        new_balance_root: H::Digest,
        prev_nullifier_root: H::Digest,
        new_nullifier_root: H::Digest,
    ) -> Self {
        Self {
            origin_network,
            prev_local_exit_root,
            new_local_exit_root,
            bridge_exits,
            imported_bridge_exits,
            imported_exits_root,
            imported_local_exit_roots,
            balances_proofs,
            prev_balance_root,
            new_balance_root,
            prev_nullifier_root,
            new_nullifier_root,
        }
    }
}
