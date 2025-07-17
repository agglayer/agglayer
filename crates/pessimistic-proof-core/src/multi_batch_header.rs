#![allow(clippy::too_many_arguments)]
use std::hash::Hash;

use agglayer_primitives::{keccak::Hasher, U256};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;
use unified_bridge::{BridgeExit, ImportedBridgeExit, NetworkId, TokenInfo};

use crate::{
    aggchain_proof::AggchainData, local_balance_tree::LocalBalancePath,
    nullifier_tree::NullifierPath,
};

// TODO: Repeated code (crates/agglayer-primitives/src/signature.rs), consider
// refactoring
#[derive(
    Clone, Debug, Serialize, Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub struct U256Def {
    pub limbs: [u64; 4],
}

impl From<U256Def> for U256 {
    #[inline]
    fn from(value: U256Def) -> Self {
        U256::from_limbs(value.limbs)
    }
}

impl From<U256> for U256Def {
    #[inline]
    fn from(value: U256) -> Self {
        Self {
            limbs: *value.as_limbs(),
        }
    }
}

/// Represents the chain state transition for the pessimistic proof.
#[serde_as]
#[derive(
    Clone, Debug, Serialize, Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub struct MultiBatchHeader<H>
where
    H: Hasher,
    H::Digest: Eq + Hash + Copy + Serialize + DeserializeOwned + rkyv::Archive,
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
    /// Optimized for SP1: Using Vec instead of BTreeMap for better cycle
    /// performance.
    pub balances_proofs: Vec<(TokenInfo, (U256Def, LocalBalancePath<H>))>,
    /// Aggchain proof.
    pub aggchain_proof: AggchainData,
}

// #[test]
// fn test_multi_batch_header_serialization() {
//     type Digest = <Keccak256Hasher as Hasher>::Digest;
//     let l: MultiBatchHeader<Keccak256Hasher> = MultiBatchHeader {
//         origin_network: NetworkId::new(0),
//         height: 1,
//         prev_pessimistic_root: Digest { 0: [0; 32] },
//         bridge_exits: vec![],
//         imported_bridge_exits: vec![],
//         l1_info_root: Digest { 0: [0; 32] },
//         balances_proofs: vec![],
//         aggchain_proof: AggchainData::Generic {
//             aggchain_params: Digest { 0: [0; 32] },
//             aggchain_vkey: [0; 8],
//         },
//     };
//     let serialized = rkyv::to_bytes::<rkyv::rancor::Error>(&l).unwrap();
//     let deserialized =
//         rkyv::from_bytes::<MultiBatchHeader<Keccak256Hasher>,
// rkyv::rancor::Error>(&serialized)             .unwrap();
//     assert_eq!(l.origin_network, deserialized.origin_network);
//     assert_eq!(l.height, deserialized.height);
//     assert_eq!(l.prev_pessimistic_root, deserialized.prev_pessimistic_root);
//     assert_eq!(l.bridge_exits, deserialized.bridge_exits);
// }
