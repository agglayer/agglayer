#![allow(clippy::too_many_arguments)]

use std::{fmt::Display, ops::Deref};

use agglayer_primitives::{address, Address, U256};
use hex_literal::hex;
use pessimistic_proof_core::bridge_exit::{LeafType, TokenInfo};
use pessimistic_proof_core::keccak::digest::Digest;
use pessimistic_proof_core::keccak::keccak256;
use pessimistic_proof_core::keccak::keccak256_combine;
use serde::{Deserialize, Serialize};

//pub(crate) const L1_NETWORK_ID: NetworkId = NetworkId(0);
pub(crate) const L1_ETH: TokenInfo = TokenInfo {
    origin_network: 0,
    origin_token_address: address!("0000000000000000000000000000000000000000"),
};

/// Represents a token bridge exit from the network.
// TODO: Change it to an enum depending on `leaf_type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeExit {
    /// Leaf type whether message of transfer.
    pub leaf_type: LeafType,
    /// Unique ID for the token being transferred.
    pub token_info: TokenInfo,
    /// Network which the token is transferred to.
    pub dest_network: NetworkId,
    /// Address which will own the received token.
    pub dest_address: Address,
    /// Token amount sent.
    pub amount: U256,
    /// Optional hash of the metadata.
    pub metadata: Option<Digest>,
}

impl From<BridgeExit> for pessimistic_proof_core::bridge_exit::BridgeExit {
    fn from(value: BridgeExit) -> Self {
        Self {
            leaf_type: value.leaf_type,
            token_info: value.token_info,
            dest_network: *value.dest_network,
            dest_address: value.dest_address,
            amount: value.amount,
            metadata: value.metadata.unwrap_or(EMPTY_METADATA_HASH).0.into(),
        }
    }
}

const EMPTY_METADATA_HASH: Digest = Digest(hex!(
    "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
));

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

    /// Hashes the [`BridgeExit`] to be inserted in a
    /// [`crate::local_exit_tree::LocalExitTree`].
    pub fn hash(&self) -> Digest {
        keccak256_combine([
            [self.leaf_type as u8].as_slice(),
            &u32::to_be_bytes(self.token_info.origin_network.into()),
            self.token_info.origin_token_address.as_slice(),
            &u32::to_be_bytes(self.dest_network.into()),
            self.dest_address.as_slice(),
            &self.amount.to_be_bytes::<32>(),
            &self.metadata.unwrap_or(EMPTY_METADATA_HASH).0,
        ])
    }

    pub fn is_transfer(&self) -> bool {
        self.leaf_type == LeafType::Transfer
    }
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
    pub fn new(value: u32) -> Self {
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
    use pessimistic_proof_core::local_state::local_exit_tree::{
        hasher::Keccak256Hasher, LocalExitTree,
    };

    use super::*;

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
