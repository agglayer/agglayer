use std::borrow::Borrow;

use reth_primitives::U256;
use serde::{Deserialize, Serialize};

use crate::{
    bridge_exit::BridgeExit,
    global_index::GlobalIndex,
    keccak::{keccak256, keccak256_combine, Digest},
    local_exit_tree::{data::LETMerkleProof, hasher::Keccak256Hasher},
    ProofError,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct L1InfoTreeLeaf {
    l1_info_tree_index: u32,
    rer: Digest,
    mer: Digest,
    last_block_hash: Digest,
    timestamp: u64,
}

impl L1InfoTreeLeaf {
    fn ger(&self) -> Digest {
        keccak256_combine([self.mer, self.rer])
    }

    fn hash(&self) -> Digest {
        let ger = self.ger();
        keccak256_combine([
            ger.as_slice(),
            self.last_block_hash.as_slice(),
            &self.timestamp.to_be_bytes(),
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    proof: LETMerkleProof<Keccak256Hasher>,
    root: Digest,
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
    proof_leaf_mer: MerkleProof,
    /// Proof from GER to L1Root
    proof_ger_l1root: MerkleProof,
    /// L1InfoTree leaf
    l1_leaf: L1InfoTreeLeaf,
}

impl ClaimFromMainnet {
    pub fn verify(
        &self,
        leaf: Digest,
        global_index: GlobalIndex,
        l1root: Digest,
    ) -> Result<(), ProofError> {
        // Check the consistency on the l1 root
        if l1root != self.l1_leaf.hash() {
            return Err(ProofError::InvalidImportedBridgeExitMerklePath);
        }

        // Check the consistency on the declared MER
        if self.proof_leaf_mer.root != self.l1_leaf.mer {
            return Err(ProofError::InvalidImportedBridgeExitMerklePath);
        }

        // Check the inclusion proof of the leaf to the MER
        if !self.proof_leaf_mer.verify(leaf, global_index.leaf_index) {
            return Err(ProofError::InvalidImportedBridgeExitMerklePath);
        }

        // Check the inclusion proof of the L1 leaf to L1Root
        if !self
            .proof_ger_l1root
            .verify(self.l1_leaf.hash(), self.l1_leaf.l1_info_tree_index)
        {
            return Err(ProofError::InvalidImportedBridgeExitMerklePath);
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
    pub fn verify(
        &self,
        leaf: Digest,
        global_index: GlobalIndex,
        l1root: Digest,
    ) -> Result<(), ProofError> {
        // Check the consistency on the l1 root
        if l1root != self.l1_leaf.hash() {
            return Err(ProofError::InvalidImportedBridgeExitMerklePath);
        }

        // Check the consistency on the declared RER
        if self.proof_ler_rer.root != self.l1_leaf.rer {
            return Err(ProofError::InvalidImportedBridgeExitMerklePath);
        }

        // Check the inclusion proof of the leaf to the LER
        if !self.proof_leaf_ler.verify(leaf, global_index.leaf_index) {
            return Err(ProofError::InvalidImportedBridgeExitMerklePath);
        }

        // Check the inclusion proof of the LER to the RER
        if !self
            .proof_ler_rer
            .verify(self.proof_leaf_ler.root, global_index.rollup_index)
        {
            return Err(ProofError::InvalidImportedBridgeExitMerklePath);
        }

        // Check the inclusion proof of the L1 leaf to L1Root
        if !self
            .proof_ger_l1root
            .verify(self.l1_leaf.hash(), self.l1_leaf.l1_info_tree_index)
        {
            return Err(ProofError::InvalidImportedBridgeExitMerklePath);
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
    pub fn verify_path(&self, l1root: Digest) -> Result<(), ProofError> {
        // Check that the inclusion proof and the global index both refer to mainnet or
        // rollup
        if self.global_index.mainnet_flag != matches!(self.claim_data, Claim::Mainnet(_)) {
            return Err(ProofError::MismatchGlobalIndexInclusionProof);
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

    pub fn hash(&self) -> Digest {
        let global_index: U256 = self.global_index.into();
        keccak256(global_index.as_le_slice())
    }
}

pub fn commit_imported_bridge_exits<E: Borrow<ImportedBridgeExit>>(
    iter: impl Iterator<Item = E>,
) -> Digest {
    keccak256_combine(iter.map(|exit| exit.borrow().hash()))
}
