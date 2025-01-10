pub use pessimistic_proof_core::imported_bridge_exit::{
    commit_imported_bridge_exits, Claim, ClaimFromMainnet, ClaimFromRollup, L1InfoTreeLeaf,
    L1InfoTreeLeafInner, MerkleProof,
};
use pessimistic_proof_core::{
    global_index::GlobalIndex,
    keccak::{digest::Digest, keccak256_combine},
};
use serde::{Deserialize, Serialize};

use crate::{bridge_exit::BridgeExit, utils::Hashable};

impl Hashable for MerkleProof {
    fn hash(&self) -> Digest {
        keccak256_combine([
            self.root.as_slice(),
            self.proof
                .siblings
                .iter()
                .flat_map(|v| v.0)
                .collect::<Vec<_>>()
                .as_slice(),
        ])
    }
}
impl Hashable for Claim {
    fn hash(&self) -> Digest {
        match self {
            Claim::Mainnet(claim_from_mainnet) => claim_from_mainnet.hash(),
            Claim::Rollup(claim_from_rollup) => claim_from_rollup.hash(),
        }
    }
}

impl Hashable for ClaimFromMainnet {
    fn hash(&self) -> Digest {
        keccak256_combine([
            self.proof_leaf_mer.hash(),
            self.proof_ger_l1root.hash(),
            self.l1_leaf.hash(),
        ])
    }
}

impl Hashable for ClaimFromRollup {
    fn hash(&self) -> Digest {
        keccak256_combine([
            self.proof_leaf_ler.hash(),
            self.proof_ler_rer.hash(),
            self.proof_ger_l1root.hash(),
            self.l1_leaf.hash(),
        ])
    }
}

/// Represents a token bridge exit originating on another network but claimed on
/// the current network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedBridgeExit {
    /// The bridge exit initiated on another network, called the "sending"
    /// network. Need to verify that the destination network matches the
    /// current network, and that the bridge exit is included in an imported
    /// LER
    pub bridge_exit: BridgeExit,
    /// The claim data
    pub claim_data: Claim,
    /// The global index of the imported bridge exit.
    pub global_index: GlobalIndex,
}

impl From<ImportedBridgeExit> for pessimistic_proof_core::imported_bridge_exit::ImportedBridgeExit {
    fn from(value: ImportedBridgeExit) -> Self {
        Self {
            bridge_exit: value.bridge_exit.into(),
            claim_data: value.claim_data,
            global_index: value.global_index,
        }
    }
}

impl ImportedBridgeExit {
    /// Creates a new [`ImportedBridgeExit`].
    pub fn new(bridge_exit: BridgeExit, claim_data: Claim, global_index: GlobalIndex) -> Self {
        Self {
            bridge_exit,
            global_index,
            claim_data,
        }
    }
    /// Returns the considered L1 Info Root against which the claim is done.
    pub fn l1_info_root(&self) -> Digest {
        match &self.claim_data {
            Claim::Mainnet(claim) => claim.proof_ger_l1root.root,
            Claim::Rollup(claim) => claim.proof_ger_l1root.root,
        }
    }

    /// Returns the considered L1 Info Tree leaf index against which the claim
    /// is done.
    pub fn l1_leaf_index(&self) -> u32 {
        match &self.claim_data {
            Claim::Mainnet(claim) => claim.l1_leaf.l1_info_tree_index,
            Claim::Rollup(claim) => claim.l1_leaf.l1_info_tree_index,
        }
    }

    /// Hash the entire data structure.
    pub fn hash(&self) -> Digest {
        keccak256_combine([
            self.bridge_exit.hash(),
            self.claim_data.hash(),
            self.global_index.hash(),
        ])
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use pessimistic_proof_core::{
        imported_bridge_exit::L1InfoTreeLeafInner, local_exit_tree::hasher::Keccak256Hasher,
    };

    use super::*;
    use crate::local_exit_tree::LocalExitTree;

    #[test]
    fn can_parse_empty_l1infotree() {
        let empty_l1_info_tree: Digest =
            hex!("27ae5ba08d7291c96c8cbddcc148bf48a6d68c7974b94356f53754ef6171d757").into();

        let l1_tree = LocalExitTree::<Keccak256Hasher, 32>::default();

        assert_eq!(empty_l1_info_tree, l1_tree.get_root());
    }

    #[test]
    fn can_parse_l1infotree_leaf() {
        assert_eq!(
            hex!("f62f487534b899b1c362242616725878188ca891fab60854b792ca0628286de7"),
            L1InfoTreeLeafInner {
                global_exit_root: hex!(
                    "16994edfddddb9480667b64174fc00d3b6da7290d37b8db3a16571b4ddf0789f"
                )
                .into(),
                block_hash: hex!(
                    "24a5871d68723340d9eadc674aa8ad75f3e33b61d5a9db7db92af856a19270bb"
                )
                .into(),
                timestamp: 1697231573,
            }
            .hash()
            .0,
        );

        assert_eq!(
            hex!("ba9c9985e6c9cee54f57991049af0c42439fa2b2915a0597f4d63f63d31c1d4f"),
            L1InfoTreeLeafInner {
                global_exit_root: hex!(
                    "356682567c5d485bbabe89590d3d72b08671a0a07899dcbaddccbe0599491669"
                )
                .into(),
                block_hash: hex!(
                    "8f9cfb43c0f6bc7ce9f9e43e8761776a2ef9657ccf87318e2487c313d119b8cf"
                )
                .into(),
                timestamp: 658736476,
            }
            .hash()
            .0,
        );
    }
}
