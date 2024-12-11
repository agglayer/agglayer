use std::borrow::Borrow;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    bridge_exit::BridgeExit,
    global_index::GlobalIndex,
    keccak::{digest::Digest, keccak256_combine},
    local_exit_tree::{data::LETMerkleProof, hasher::Keccak256Hasher},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L1InfoTreeLeafInner {
    pub global_exit_root: Digest,
    pub block_hash: Digest,
    pub timestamp: u64,
}

impl L1InfoTreeLeafInner {
    fn hash(&self) -> Digest {
        keccak256_combine([
            self.global_exit_root.as_slice(),
            self.block_hash.as_slice(),
            &self.timestamp.to_be_bytes(),
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L1InfoTreeLeaf {
    pub l1_info_tree_index: u32,
    pub rer: Digest,
    pub mer: Digest,
    pub inner: L1InfoTreeLeafInner,
}

impl L1InfoTreeLeaf {
    pub fn hash(&self) -> Digest {
        self.inner.hash()
    }
}

#[derive(Clone, Debug, Error, Serialize, Deserialize, PartialEq, Eq)]
pub enum Error {
    /// The global index and the inclusion proof do not both correspond to the
    /// same network type: mainnet or rollup.
    #[error("Mismatch between the global index and the inclusion proof.")]
    MismatchGlobalIndexInclusionProof,
    /// The provided L1 info root does not match the one provided in the
    /// inclusion proof.
    #[error("Mismatch between the provided L1 root and the inclusion proof.")]
    MismatchL1Root,
    /// The provided L1 info leaf does not refer to the same MER as the
    /// inclusion proof.
    #[error("Mismatch on the MER between the L1 leaf and the inclusion proof.")]
    MismatchMER,
    /// The provided L1 info leaf does not refer to the same RER as the
    /// inclusion proof.
    #[error("Mismatch on the RER between the L1 leaf and the inclusion proof.")]
    MismatchRER,
    /// The inclusion proof from the leaf to the LER is invalid.
    #[error("Invalid merkle path from the leaf to the LER.")]
    InvalidMerklePathLeafToLER,
    /// The inclusion proof from the LER to the RER is invalid.
    #[error("Invalid merkle path from the LER to the RER.")]
    InvalidMerklePathLERToRER,
    /// The inclusion proof from the GER to the L1 info Root is invalid.
    #[error("Invalid merkle path from the GER to the L1 Info Root.")]
    InvalidMerklePathGERToL1Root,
    /// The provided imported bridge exit does not target the right destination
    /// network.
    #[error("Invalid imported bridge exit destination network.")]
    InvalidExitNetwork,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub proof: LETMerkleProof<Keccak256Hasher>,
    pub root: Digest,
}

impl MerkleProof {
    pub fn hash(&self) -> Digest {
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

    pub fn verify(&self, leaf: Digest, leaf_index: u32) -> bool {
        self.proof.verify(leaf, leaf_index, self.root)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Claim {
    Mainnet(Box<ClaimFromMainnet>),
    Rollup(Box<ClaimFromRollup>),
}

impl Claim {
    pub fn hash(&self) -> Digest {
        match self {
            Claim::Mainnet(claim_from_mainnet) => claim_from_mainnet.hash(),
            Claim::Rollup(claim_from_rollup) => claim_from_rollup.hash(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimFromMainnet {
    /// Proof from bridge exit leaf to MER
    pub proof_leaf_mer: MerkleProof,
    /// Proof from GER to L1Root
    pub proof_ger_l1root: MerkleProof,
    /// L1InfoTree leaf
    pub l1_leaf: L1InfoTreeLeaf,
}

impl ClaimFromMainnet {
    pub fn hash(&self) -> Digest {
        keccak256_combine([
            self.proof_leaf_mer.hash(),
            self.proof_ger_l1root.hash(),
            self.l1_leaf.hash(),
        ])
    }

    pub fn verify(
        &self,
        leaf: Digest,
        global_index: GlobalIndex,
        l1root: Digest,
    ) -> Result<(), Error> {
        // Check the consistency on the l1 root
        if l1root != self.proof_ger_l1root.root {
            return Err(Error::MismatchL1Root);
        }

        // Check the consistency on the declared MER
        if self.proof_leaf_mer.root != self.l1_leaf.mer {
            return Err(Error::MismatchMER);
        }

        // Check the inclusion proof of the leaf to the LER (here LER is the MER)
        if !self.proof_leaf_mer.verify(leaf, global_index.leaf_index) {
            return Err(Error::InvalidMerklePathLeafToLER);
        }

        // Check the inclusion proof of the L1 leaf to L1Root
        if !self
            .proof_ger_l1root
            .verify(self.l1_leaf.hash(), self.l1_leaf.l1_info_tree_index)
        {
            return Err(Error::InvalidMerklePathGERToL1Root);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimFromRollup {
    /// Proof from bridge exit leaf to LER
    proof_leaf_ler: MerkleProof,
    /// Proof from LER to RER
    proof_ler_rer: MerkleProof,
    /// Proof from GER to L1Root
    proof_ger_l1root: MerkleProof,
    /// L1InfoTree leaf
    l1_leaf: L1InfoTreeLeaf,
}

impl ClaimFromRollup {
    pub fn hash(&self) -> Digest {
        keccak256_combine([
            self.proof_leaf_ler.hash(),
            self.proof_ler_rer.hash(),
            self.proof_ger_l1root.hash(),
            self.l1_leaf.hash(),
        ])
    }

    pub fn verify(
        &self,
        leaf: Digest,
        global_index: GlobalIndex,
        l1root: Digest,
    ) -> Result<(), Error> {
        // Check the consistency on the l1 root
        if l1root != self.proof_ger_l1root.root {
            return Err(Error::MismatchL1Root);
        }

        // Check the consistency on the declared RER
        if self.proof_ler_rer.root != self.l1_leaf.rer {
            return Err(Error::MismatchRER);
        }

        // Check the inclusion proof of the leaf to the LER
        if !self.proof_leaf_ler.verify(leaf, global_index.leaf_index) {
            return Err(Error::InvalidMerklePathLeafToLER);
        }

        // Check the inclusion proof of the LER to the RER
        if !self
            .proof_ler_rer
            .verify(self.proof_leaf_ler.root, global_index.rollup_index)
        {
            return Err(Error::InvalidMerklePathLERToRER);
        }

        // Check the inclusion proof of the L1 leaf to L1Root
        if !self
            .proof_ger_l1root
            .verify(self.l1_leaf.hash(), self.l1_leaf.l1_info_tree_index)
        {
            return Err(Error::InvalidMerklePathGERToL1Root);
        }

        Ok(())
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

impl ImportedBridgeExit {
    /// Creates a new [`ImportedBridgeExit`].
    pub fn new(bridge_exit: BridgeExit, claim_data: Claim, global_index: GlobalIndex) -> Self {
        Self {
            bridge_exit,
            global_index,
            claim_data,
        }
    }

    /// Verifies that the provided inclusion path is valid and consistent with
    /// the provided LER
    pub fn verify_path(&self, l1root: Digest) -> Result<(), Error> {
        // Check that the inclusion proof and the global index both refer to mainnet or
        // rollup
        if self.global_index.mainnet_flag != matches!(self.claim_data, Claim::Mainnet(_)) {
            return Err(Error::MismatchGlobalIndexInclusionProof);
        }

        match &self.claim_data {
            Claim::Mainnet(claim) => {
                claim.verify(self.bridge_exit.hash(), self.global_index, l1root)
            }
            Claim::Rollup(claim) => {
                claim.verify(self.bridge_exit.hash(), self.global_index, l1root)
            }
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

pub fn commit_imported_bridge_exits<E: Borrow<ImportedBridgeExit>>(
    iter: impl Iterator<Item = E>,
) -> Digest {
    keccak256_combine(iter.map(|exit| exit.borrow().global_index.hash()))
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

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
