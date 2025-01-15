use std::{fmt::Display, ops::Deref};

use agglayer_primitives::{Address, U256};
pub use pessimistic_proof_core::bridge_exit::{LeafType, TokenInfo};
use pessimistic_proof_core::{
    bridge_exit::L1_ETH,
    keccak::{digest::Digest, keccak256, keccak256_combine},
};
use serde::{Deserialize, Serialize};

use crate::utils::Hashable;

impl Hashable for TokenInfo {
    fn hash(&self) -> Digest {
        keccak256_combine([
            &self.origin_network.to_be_bytes(),
            self.origin_token_address.as_slice(),
        ])
    }
}

/// Represents a token bridge exit from the network.
// TODO: Change it to an enum depending on `leaf_type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeExit {
    pub leaf_type: LeafType,

    /// Unique ID for the token being transferred.
    pub token_info: TokenInfo,

    /// Network which the token is transferred to
    pub dest_network: NetworkId,
    /// Address which will own the received token
    pub dest_address: Address,

    /// Token amount sent
    pub amount: U256,

    pub metadata: Option<Digest>,
}

impl From<BridgeExit> for pessimistic_proof_core::bridge_exit::BridgeExit {
    fn from(value: BridgeExit) -> Self {
        Self {
            leaf_type: value.leaf_type,
            token_info: value.token_info,
            dest_network: value.dest_network.into(),
            dest_address: value.dest_address,
            amount: value.amount,
            metadata: value.metadata,
        }
    }
}

impl BridgeExit {
    /// Creates a new [`BridgeExit`].
    pub fn new(
        leaf_type: LeafType,
        origin_network: NetworkId,
        origin_token_address: Address,
        dest_network: NetworkId,
        dest_address: Address,
        amount: U256,
        metadata: Vec<u8>,
    ) -> Self {
        Self {
            leaf_type,
            token_info: TokenInfo {
                origin_network: *origin_network,
                origin_token_address,
            },
            dest_network,
            dest_address,
            amount,
            metadata: Some(keccak256(metadata.as_slice())),
        }
    }

    pub fn is_transfer(&self) -> bool {
        self.leaf_type == LeafType::Transfer
    }
}

impl Hashable for BridgeExit {
    fn hash(&self) -> Digest {
        pessimistic_proof_core::bridge_exit::bridge_exit_hasher(
            self.leaf_type as u8,
            self.token_info.origin_network,
            self.token_info.origin_token_address,
            *self.dest_network,
            self.dest_address,
            self.amount,
            self.metadata,
        )
    }
}

impl BridgeExit {
    pub fn is_message(&self) -> bool {
        self.leaf_type == LeafType::Message
    }

    /// Returns the [`TokenInfo`] considered for the the given amount.
    /// The amount corresponds to L1 ETH if the bridge exit is a message.
    pub fn amount_token_info(&self) -> TokenInfo {
        match self.leaf_type {
            LeafType::Message => L1_ETH,
            LeafType::Transfer => self.token_info,
        }
    }
}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash,
)]
pub struct NetworkId(u32);

impl Display for NetworkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl NetworkId {
    pub const BITS: usize = u32::BITS as usize;
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

impl From<u32> for NetworkId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<NetworkId> for u32 {
    fn from(value: NetworkId) -> Self {
        value.0
    }
}

impl Deref for NetworkId {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use pessimistic_proof_core::local_exit_tree::hasher::Keccak256Hasher;

    use super::*;
    use crate::local_exit_tree::LocalExitTree;

    #[test]
    fn test_deposit_hash() {
        let mut deposit = BridgeExit::new(
            LeafType::Transfer,
            0.into(),
            Address::default(),
            1.into(),
            Address::default(),
            U256::default(),
            vec![],
        );

        let amount_bytes = hex::decode("8ac7230489e80000").unwrap_or_default();
        deposit.amount = U256::try_from_be_slice(amount_bytes.as_slice()).unwrap();

        let dest_addr = hex::decode("c949254d682d8c9ad5682521675b8f43b102aec4").unwrap_or_default();
        deposit.dest_address.copy_from_slice(&dest_addr);

        let leaf_hash = deposit.hash();
        assert_eq!(
            "22ed288677b4c2afd83a6d7d55f7df7f4eaaf60f7310210c030fd27adacbc5e0",
            hex::encode(leaf_hash)
        );

        let mut dm = LocalExitTree::<Keccak256Hasher>::new();
        dm.add_leaf(leaf_hash).unwrap();
        let dm_root = dm.get_root();
        assert_eq!(
            "5ba002329b53c11a2f1dfe90b11e031771842056cf2125b43da8103c199dcd7f",
            hex::encode(dm_root)
        );
    }
}
