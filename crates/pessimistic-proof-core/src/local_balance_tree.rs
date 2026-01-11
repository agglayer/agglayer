use agglayer_primitives::{Digest, FromU256, U256};
use agglayer_tries::proof::SmtMerkleProof;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use unified_bridge::TokenInfo;

use crate::ProofError;

/// The key is [`TokenInfo`] which can be packed into 192 bits (32 for network
/// id and 160 for token address).
pub const LOCAL_BALANCE_TREE_DEPTH: usize = 192;

/// A commitment to the set of per-network local balance trees maintained by the
/// local network
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalBalanceTree {
    /// The Merkle Root of the local balance tree
    #[serde_as(as = "_")]
    pub root: Digest,
}

pub type LocalBalancePath = SmtMerkleProof<LOCAL_BALANCE_TREE_DEPTH>;

impl LocalBalanceTree {
    pub fn verify_and_update(
        &mut self,
        key: TokenInfo,
        path_to_update: &LocalBalancePath,
        old_balance: U256,
        new_balance: U256,
    ) -> Result<(), ProofError> {
        self.root = path_to_update
            .verify_and_update(
                key,
                Digest::from_u256(old_balance),
                Digest::from_u256(new_balance),
                self.root,
            )
            .ok_or(ProofError::InvalidBalancePath)?;

        Ok(())
    }
}
