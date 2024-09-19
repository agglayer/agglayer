#![allow(clippy::too_many_arguments)]
use std::{collections::BTreeMap, hash::Hash};

use reth_primitives::{Address, Signature, U256};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;

use crate::{
    bridge_exit::{BridgeExit, NetworkId, TokenInfo},
    imported_bridge_exit::ImportedBridgeExit,
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
    /// The origin network which emitted this BatchHeader.
    /// TODO: we should clarify naming. We use origin to refer to the issuing
    /// network of a token. Could consider using "sending" or "local" to refer
    /// to the network that created a batch.
    pub origin_network: NetworkId,

    /// The initial local exit root.
    #[serde_as(as = "_")]
    pub prev_local_exit_root: H::Digest,

    /// The set of bridge exits created in this batch
    /// TODO: move out of the header and into a separate struct
    pub bridge_exits: Vec<BridgeExit>,

    /// The set of imported bridge exits claimed in this batch
    /// TODO: move out of the header and into a separate struct
    pub imported_bridge_exits: Vec<(ImportedBridgeExit, NullifierPath<H>)>,

    /// A commitment to the set of imported bridge exits for which the origin
    /// network is the target.
    #[serde_as(as = "Option<_>")]
    pub imported_exits_root: Option<H::Digest>,

    /// The l1 info root against which we import the bridge exits
    #[serde_as(as = "_")]
    pub l1_info_root: H::Digest,

    /// A map from token info to the token balance of the origin network before
    /// any bridge event is processed, along with the Merkle proof of this
    /// balance in the local balance tree.
    // TODO: benchmark if BTreeMap is the best choice in terms of SP1 cycles
    pub balances_proofs: BTreeMap<TokenInfo, (U256, LocalBalancePath<H>)>,

    /// The previous Local Balance Root
    #[serde_as(as = "_")]
    pub prev_balance_root: H::Digest,

    /// The previous NullifierTree Root
    #[serde_as(as = "_")]
    pub prev_nullifier_root: H::Digest,

    /// The signer that commits to the bridge exits
    pub signer: Address,
    /// The signature that commits to the bridge exits
    pub signature: Signature,

    /// Target hashes
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
