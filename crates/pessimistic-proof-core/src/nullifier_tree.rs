use agglayer_primitives::{Digest, FromBool};
use agglayer_tries::proof::{SmtNonInclusionProof, ToBits};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use unified_bridge::{GlobalIndex, NetworkId};

use crate::ProofError;

// 32 bits for the network id and 32 bits for the LET index
pub const NULLIFIER_TREE_DEPTH: usize = 64;

/// A commitment to the set of per-network nullifier trees maintained by the
/// local network
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NullifierTree {
    /// The Merkle Root of the nullifier tree
    #[serde_as(as = "_")]
    pub root: Digest,
}

pub type NullifierPath = SmtNonInclusionProof<NULLIFIER_TREE_DEPTH>;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct NullifierKey {
    pub network_id: NetworkId,
    pub let_index: u32,
}

impl From<GlobalIndex> for NullifierKey {
    fn from(value: GlobalIndex) -> Self {
        Self {
            network_id: value.network_id(),
            let_index: value.leaf_index(),
        }
    }
}

impl ToBits<64> for NullifierKey {
    fn to_bits(&self) -> [bool; 64] {
        std::array::from_fn(|i| {
            if i < 32 {
                (self.network_id.to_u32() >> i) & 1 == 1
            } else {
                (self.let_index >> (i - 32)) & 1 == 1
            }
        })
    }
}

impl NullifierTree {
    pub fn verify_and_update(
        &mut self,
        key: NullifierKey,
        path_to_update: &NullifierPath,
    ) -> Result<(), ProofError> {
        self.root = path_to_update
            .verify_and_update(key, Digest::from_bool(true), self.root)
            .ok_or(ProofError::InvalidNullifierPath)?;

        Ok(())
    }
}
