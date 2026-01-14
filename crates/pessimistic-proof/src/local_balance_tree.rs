use agglayer_primitives::{Digest, U256};
use agglayer_tries::{
    error::SmtError,
    smt::{Smt, SmtPath},
    utils::empty_hash_at_height,
};
pub use pessimistic_proof_core::local_balance_tree::{LocalBalancePath, LOCAL_BALANCE_TREE_DEPTH};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use unified_bridge::TokenInfo;

/// A commitment to the set of per-network local balance trees maintained by the
/// local network
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalBalanceTree {
    /// The Merkle Root of the local balance tree
    #[serde_as(as = "_")]
    pub root: Digest,
}

impl Default for LocalBalanceTree {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalBalanceTree {
    pub fn new() -> Self {
        // We add 1 to the depth here because the empty hash at height 0 is
        // already set to Digest::ZERO.
        let root = empty_hash_at_height::<{ LOCAL_BALANCE_TREE_DEPTH + 1 }>();
        LocalBalanceTree { root }
    }

    pub fn new_with_root(root: Digest) -> Self {
        LocalBalanceTree { root }
    }
}

impl From<LocalBalanceTree> for pessimistic_proof_core::local_balance_tree::LocalBalanceTree {
    fn from(tree: LocalBalanceTree) -> Self {
        Self { root: tree.root }
    }
}

pub struct BalanceTree(pub Smt<LOCAL_BALANCE_TREE_DEPTH>);

impl BalanceTree {
    /// Returns all the non-zero token balance contained in this balance tree.
    pub fn get_all_balances(
        &self,
    ) -> Result<impl Iterator<Item = (SmtPath<LOCAL_BALANCE_TREE_DEPTH>, Digest)>, SmtError> {
        Ok(self.0.entries()?.into_iter())
    }

    /// Returns the balance for the given [`TokenInfo`].
    pub fn get_balance(&self, token_info: TokenInfo) -> U256 {
        self.0
            .get(token_info)
            .map(|v| U256::from_be_bytes(*v.as_bytes()))
            .unwrap_or(U256::ZERO)
    }
}
