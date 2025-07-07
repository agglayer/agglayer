use std::hash::Hash;

use agglayer_primitives::{keccak::Hasher, FromU256, U256};
use agglayer_tries::proof::SmtMerkleProof;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;
use unified_bridge::TokenInfo;

use crate::ProofError;

/// The key is [`TokenInfo`] which can be packed into 192 bits (32 for network
/// id and 160 for token address).
pub const LOCAL_BALANCE_TREE_DEPTH: usize = 192;

// TODO: This is basically the same as the nullifier tree, consider refactoring
/// A commitment to the set of per-network nullifier trees maintained by the
/// local network
#[serde_as]
#[derive(
    Clone, Debug, Serialize, Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub struct LocalBalanceTree<H>
where
    H: Hasher,
    H::Digest: Serialize + DeserializeOwned + rkyv::Archive,
{
    /// The Merkle Root of the nullifier tree
    #[serde_as(as = "_")]
    pub root: H::Digest,
}

pub type LocalBalancePath<H> = SmtMerkleProof<H, LOCAL_BALANCE_TREE_DEPTH>;

impl<H> LocalBalanceTree<H>
where
    H: Hasher,
    H::Digest:
        Copy + Eq + Hash + Default + Serialize + for<'a> Deserialize<'a> + FromU256 + rkyv::Archive,
{
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
