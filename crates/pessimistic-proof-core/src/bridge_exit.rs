#![allow(clippy::too_many_arguments)]

use agglayer_primitives::{address, Address, U256};
use hex_literal::hex;
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
    pub origin_network: NetworkId,
    /// The address of the token on the origin network
    pub origin_token_address: Address,
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

const EMPTY_METADATA_HASH: Digest = Digest(hex!(
    "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
));

pub fn bridge_exit_hasher(
    leaf_type: u8,
    origin_network: u32,
    origin_token_address: Address,
    dest_network: u32,
    dest_address: Address,
    amount: U256,
    metadata: Option<Digest>,
) -> Digest {
    keccak256_combine([
        [leaf_type].as_slice(),
        &u32::to_be_bytes(origin_network),
        origin_token_address.as_slice(),
        &u32::to_be_bytes(dest_network),
        dest_address.as_slice(),
        &amount.to_be_bytes::<32>(),
        &metadata.unwrap_or(EMPTY_METADATA_HASH).0,
    ])
}

impl BridgeExit {
    /// Hashes the [`BridgeExit`] to be inserted in a
    /// [`crate::local_exit_tree::LocalExitTree`].
    pub fn hash(&self) -> Digest {
        bridge_exit_hasher(
            self.leaf_type as u8,
            self.token_info.origin_network,
            self.token_info.origin_token_address,
            self.dest_network,
            self.dest_address,
            self.amount,
            self.metadata,
        )
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
