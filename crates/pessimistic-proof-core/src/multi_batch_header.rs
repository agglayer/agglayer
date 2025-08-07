#![allow(clippy::too_many_arguments)]
use std::collections::BTreeMap;

use agglayer_primitives::{Digest, U256};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use unified_bridge::{BridgeExit, ImportedBridgeExit, NetworkId, TokenInfo};

use crate::{
    aggchain_proof::AggchainData, local_balance_tree::LocalBalancePath,
    nullifier_tree::NullifierPath,
};

/// Represents the chain state transition for the pessimistic proof.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultiBatchHeader {
    /// Network that emitted this [`MultiBatchHeader`].
    pub origin_network: NetworkId,
    /// Current certificate height of the L2 chain.
    pub height: u64,
    /// Previous pessimistic root.
    #[serde_as(as = "_")]
    pub prev_pessimistic_root: Digest,
    /// List of bridge exits created in this batch.
    pub bridge_exits: Vec<BridgeExit>,
    /// List of imported bridge exits claimed in this batch.
    pub imported_bridge_exits: Vec<(ImportedBridgeExit, NullifierPath)>,
    /// L1 info root used to import bridge exits.
    #[serde_as(as = "_")]
    pub l1_info_root: Digest,
    /// Token balances of the origin network before processing bridge events,
    /// with Merkle proofs of these balances in the local balance tree.
    pub balances_proofs: BTreeMap<TokenInfo, (U256, LocalBalancePath)>,
    /// Aggchain proof.
    pub aggchain_proof: AggchainData,
}
