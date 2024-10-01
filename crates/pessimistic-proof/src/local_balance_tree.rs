use std::hash::Hash;

use reth_primitives::U256;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;

use crate::{
    bridge_exit::TokenInfo,
    local_exit_tree::hasher::Hasher,
    utils::smt::{SmtMerkleProof, ToBits},
    ProofError,
};

/// The key is [`TokenInfo`] which can be packed into 192 bits (32 for network
/// id and 160 for token address).
pub const LOCAL_BALANCE_TREE_DEPTH: usize = 192;

// TODO: This is basically the same as the nullifier tree, consider refactoring
/// A commitment to the set of per-network nullifier trees maintained by the
/// local network
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalBalanceTree<H>
where
    H: Hasher,
    H::Digest: Serialize + DeserializeOwned,
{
    /// The Merkle Root of the nullifier tree
    #[serde_as(as = "_")]
    pub root: H::Digest,
    /// `empty_hash_at_height[i]` is the root of an empty Merkle tree of depth
    /// `i`.
    #[serde_as(as = "[_; LOCAL_BALANCE_TREE_DEPTH]")]
    empty_hash_at_height: [H::Digest; LOCAL_BALANCE_TREE_DEPTH],
}

pub type LocalBalancePath<H> = SmtMerkleProof<H, LOCAL_BALANCE_TREE_DEPTH>;

impl ToBits<192> for TokenInfo {
    fn to_bits(&self) -> [bool; 192] {
        let address_bytes = self.origin_token_address.0;
        std::array::from_fn(|i| {
            if i < 32 {
                (*self.origin_network >> i) & 1 == 1
            } else {
                ((address_bytes[(i - 32) / 8]) >> (i % 8)) & 1 == 1
            }
        })
    }
}

pub trait FromU256 {
    fn from_u256(u: U256) -> Self;
}

impl<H> Default for LocalBalanceTree<H>
where
    H: Hasher,
    H::Digest: Copy + Eq + Hash + Default + Serialize + for<'a> Deserialize<'a> + FromU256,
{
    fn default() -> Self {
        Self::new()
    }
}
impl<H> LocalBalanceTree<H>
where
    H: Hasher,
    H::Digest: Copy + Eq + Hash + Default + Serialize + for<'a> Deserialize<'a> + FromU256,
{
    pub fn new() -> Self {
        let mut empty_hash_at_height = [H::Digest::default(); LOCAL_BALANCE_TREE_DEPTH];
        for height in 1..LOCAL_BALANCE_TREE_DEPTH {
            empty_hash_at_height[height] = H::merge(
                &empty_hash_at_height[height - 1],
                &empty_hash_at_height[height - 1],
            );
        }
        let root = H::merge(
            &empty_hash_at_height[LOCAL_BALANCE_TREE_DEPTH - 1],
            &empty_hash_at_height[LOCAL_BALANCE_TREE_DEPTH - 1],
        );
        LocalBalanceTree {
            root,
            empty_hash_at_height,
        }
    }

    pub fn new_with_root(root: H::Digest) -> Self {
        let mut res = Self::new();
        res.root = root;
        res
    }

    // TODO: Consider batching the updates per network for efficiency
    pub fn verify_and_update(
        &mut self,
        key: TokenInfo,
        path_to_update: &LocalBalancePath<H>,
        old_balance: U256,
        new_balance: U256,
    ) -> Result<(), ProofError> {
        self.root = path_to_update
            .verify_and_update(
                key,
                H::Digest::from_u256(old_balance),
                H::Digest::from_u256(new_balance),
                self.root,
            )
            .ok_or(ProofError::InvalidBalancePath)?;

        Ok(())
    }
}
