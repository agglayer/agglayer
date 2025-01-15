use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    bridge_exit::BridgeExit,
    global_index::GlobalIndex,
    keccak::{digest::Digest, keccak256_combine},
    local_exit_tree::{hasher::Keccak256Hasher, proof::LETMerkleProof},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L1InfoTreeLeafInner {
    pub global_exit_root: Digest,
    pub block_hash: Digest,
    pub timestamp: u64,
}

impl L1InfoTreeLeafInner {
    pub fn hash(&self) -> Digest {
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
    pub fn verify(&self, leaf: Digest, leaf_index: u32) -> bool {
        self.proof.verify(leaf, leaf_index, self.root)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Claim {
    Mainnet(Box<ClaimFromMainnet>),
    Rollup(Box<ClaimFromRollup>),
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
    pub proof_leaf_ler: MerkleProof,
    /// Proof from LER to RER
    pub proof_ler_rer: MerkleProof,
    /// Proof from GER to L1Root
    pub proof_ger_l1root: MerkleProof,
    /// L1InfoTree leaf
    pub l1_leaf: L1InfoTreeLeaf,
}

impl ClaimFromRollup {
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
}

pub fn commit_imported_bridge_exits(iter: impl Iterator<Item = GlobalIndex>) -> Digest {
    keccak256_combine(iter.map(|global_index| global_index.hash()))
}
