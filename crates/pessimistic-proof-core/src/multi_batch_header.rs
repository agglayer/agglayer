#![allow(clippy::too_many_arguments)]
use std::{collections::BTreeMap, hash::Hash};

use agglayer_primitives::keccak::Hasher;
use agglayer_primitives::U256;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;
use unified_bridge::bridge_exit::{BridgeExit, NetworkId};
use unified_bridge::imported_bridge_exit::ImportedBridgeExit;
use unified_bridge::token_info::TokenInfo;

use crate::{
    aggchain_proof::AggchainData, local_balance_tree::LocalBalancePath,
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
    /// Current certificate height of the L2 chain.
    pub height: u64,
    /// Previous pessimistic root.
    #[serde_as(as = "_")]
    pub prev_pessimistic_root: H::Digest,
    /// List of bridge exits created in this batch.
    pub bridge_exits: Vec<BridgeExit>,
    /// List of imported bridge exits claimed in this batch.
    pub imported_bridge_exits: Vec<(ImportedBridgeExit, NullifierPath<H>)>,
    /// L1 info root used to import bridge exits.
    #[serde_as(as = "_")]
    pub l1_info_root: H::Digest,
    /// Token balances of the origin network before processing bridge events,
    /// with Merkle proofs of these balances in the local balance tree.
    pub balances_proofs: BTreeMap<TokenInfo, (U256, LocalBalancePath<H>)>,
    /// Aggchain proof.
    pub aggchain_proof: AggchainData,
}
