#![allow(clippy::too_many_arguments)]

use agglayer_primitives::{address, Address, U256};
use serde::{Deserialize, Serialize};

use crate::keccak::{digest::Digest, keccak256_combine};

pub const L1_NETWORK_ID: NetworkId = 0;
pub const L1_ETH: TokenInfo = TokenInfo {
    origin_network: L1_NETWORK_ID,
    origin_token_address: address!("0000000000000000000000000000000000000000"),
};

/// Encapsulates the information to uniquely identify a token on the origin
/// network.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Copy)]
pub struct TokenInfo {
    /// Network which the token originates from
    pub origin_network: u32,
    /// The address of the token on the origin network
    pub origin_token_address: Address,
}

impl TokenInfo {
    /// Computes the Keccak digest of [`TokenInfo`].
    pub fn hash(&self) -> Digest {
        keccak256_combine([
            &self.origin_network.to_be_bytes(),
            self.origin_token_address.as_slice(),
        ])
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeafType {
    Transfer = 0,
    Message = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("Invalid leaf type number")]
pub struct LeafTypeFromU8Error;

impl TryFrom<u8> for LeafType {
    type Error = LeafTypeFromU8Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Transfer),
            1 => Ok(Self::Message),
            _ => Err(LeafTypeFromU8Error),
        }
    }
}

/// Represents a token bridge exit from the network.
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
    /// Hash of the metadata.
    pub metadata: Digest,
}

impl BridgeExit {
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
            &self.metadata.0,
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

pub type NetworkId = u32;
