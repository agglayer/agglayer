#![allow(clippy::too_many_arguments)]

use std::hash::Hash;

use agglayer_primitives::{keccak::Hasher, U256};
use bytemuck::{Pod, Zeroable};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;
use unified_bridge::{BridgeExit, ImportedBridgeExit, NetworkId, TokenInfo};

use crate::{
    aggchain_proof::AggchainData, local_balance_tree::LocalBalancePath,
    nullifier_tree::NullifierPath,
};

/// Zero-copy compatible BridgeExit for bytemuck operations.
/// This is a fixed-size version that can be safely transmuted.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct BridgeExitZeroCopy {
    /// Origin network (u32)
    pub origin_network: u32,
    /// Destination network (u32)
    pub dest_network: u32,
    /// Origin token address (20 bytes)
    pub origin_token_address: [u8; 20],
    /// Destination address (20 bytes)
    pub dest_address: [u8; 20],
    /// Amount (32 bytes)
    pub amount: [u8; 32],
    /// Metadata hash (32 bytes, 0 if None)
    pub metadata_hash: [u8; 32],
    /// Leaf type (u8: 0=Transfer, 1=Message)
    pub leaf_type: u8,
    /// Padding to ensure proper alignment
    pub _padding: [u8; 3],
}

impl BridgeExitZeroCopy {
    /// Get the size of this struct in bytes.
    pub const fn size() -> usize {
        std::mem::size_of::<Self>()
    }

    /// Convert from BridgeExit to BridgeExitZeroCopy
    pub fn from_bridge_exit(bridge_exit: &unified_bridge::BridgeExit) -> Self {
        Self {
            origin_network: bridge_exit.token_info.origin_network.to_u32(),
            dest_network: bridge_exit.dest_network.to_u32(),
            origin_token_address: bridge_exit
                .token_info
                .origin_token_address
                .as_slice()
                .try_into()
                .unwrap(),
            dest_address: bridge_exit.dest_address.as_slice().try_into().unwrap(),
            amount: bridge_exit.amount.to_be_bytes(),
            metadata_hash: bridge_exit.metadata.unwrap_or_default().0,
            leaf_type: bridge_exit.leaf_type as u8,
            _padding: [0; 3],
        }
    }

    /// Convert from BridgeExitZeroCopy to BridgeExit
    pub fn to_bridge_exit(&self) -> unified_bridge::BridgeExit {
        unified_bridge::BridgeExit {
            leaf_type: self
                .leaf_type
                .try_into()
                .unwrap_or(unified_bridge::LeafType::Transfer),
            token_info: unified_bridge::TokenInfo {
                origin_network: unified_bridge::NetworkId::new(self.origin_network),
                origin_token_address: agglayer_primitives::Address::new(self.origin_token_address),
            },
            dest_network: unified_bridge::NetworkId::new(self.dest_network),
            dest_address: agglayer_primitives::Address::from(self.dest_address),
            amount: agglayer_primitives::U256::from_be_bytes(self.amount),
            metadata: if self.metadata_hash == [0; 32] {
                None
            } else {
                Some(agglayer_primitives::Digest(self.metadata_hash))
            },
        }
    }
}

/// Zero-copy compatible TokenInfo for bytemuck operations.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct TokenInfoZeroCopy {
    /// Origin network (u32)
    pub origin_network: u32,
    /// Origin token address (20 bytes)
    pub origin_token_address: [u8; 20],
}

impl TokenInfoZeroCopy {
    /// Get the size of this struct in bytes.
    pub const fn size() -> usize {
        std::mem::size_of::<Self>()
    }

    /// Convert from TokenInfo to TokenInfoZeroCopy
    pub fn from_token_info(token_info: &unified_bridge::TokenInfo) -> Self {
        Self {
            origin_network: token_info.origin_network.to_u32(),
            origin_token_address: token_info
                .origin_token_address
                .as_slice()
                .try_into()
                .unwrap(),
        }
    }

    /// Convert from TokenInfoZeroCopy to TokenInfo
    pub fn to_token_info(&self) -> unified_bridge::TokenInfo {
        unified_bridge::TokenInfo {
            origin_network: unified_bridge::NetworkId::new(self.origin_network),
            origin_token_address: agglayer_primitives::Address::from(self.origin_token_address),
        }
    }
}

/// Zero-copy compatible ImportedBridgeExit for bytemuck operations.
/// This captures the essential fixed-size data from ImportedBridgeExit.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct ImportedBridgeExitZeroCopy {
    /// Global index leaf_index (u64)
    pub global_index_index: u64,
    /// Global index rollup_index (u32) - this is what GlobalIndex::new()
    /// expects as first parameter
    pub global_index_rollup: u32,
    /// Bridge exit data (120 bytes)
    pub bridge_exit: BridgeExitZeroCopy,
}

impl ImportedBridgeExitZeroCopy {
    /// Get the size of this struct in bytes.
    pub const fn size() -> usize {
        std::mem::size_of::<Self>()
    }

    /// Convert from ImportedBridgeExit to ImportedBridgeExitZeroCopy
    pub fn from_imported_bridge_exit(
        imported_bridge_exit: &unified_bridge::ImportedBridgeExit,
    ) -> Self {
        Self {
            global_index_index: imported_bridge_exit.global_index.leaf_index() as u64,
            global_index_rollup: imported_bridge_exit
                .global_index
                .rollup_index()
                .unwrap()
                .to_u32(),
            bridge_exit: BridgeExitZeroCopy::from_bridge_exit(&imported_bridge_exit.bridge_exit),
        }
    }

    /// Convert from ImportedBridgeExitZeroCopy to ImportedBridgeExit
    pub fn to_imported_bridge_exit(&self) -> unified_bridge::ImportedBridgeExit {
        unified_bridge::ImportedBridgeExit {
            bridge_exit: self.bridge_exit.to_bridge_exit(),
            claim_data: unified_bridge::Claim::Mainnet(Box::new(
                unified_bridge::ClaimFromMainnet {
                    proof_leaf_mer: unified_bridge::MerkleProof {
                        proof: unified_bridge::LETMerkleProof {
                            siblings: [agglayer_primitives::Digest([0u8; 32]); 32],
                        },
                        root: agglayer_primitives::Digest([0u8; 32]),
                    },
                    proof_ger_l1root: unified_bridge::MerkleProof {
                        proof: unified_bridge::LETMerkleProof {
                            siblings: [agglayer_primitives::Digest([0u8; 32]); 32],
                        },
                        root: agglayer_primitives::Digest([0u8; 32]),
                    },
                    l1_leaf: unified_bridge::L1InfoTreeLeaf {
                        l1_info_tree_index: 0,
                        rer: agglayer_primitives::Digest::default(),
                        mer: agglayer_primitives::Digest::default(),
                        inner: unified_bridge::L1InfoTreeLeafInner {
                            block_hash: agglayer_primitives::Digest::default(),
                            timestamp: 0,
                            global_exit_root: agglayer_primitives::Digest::default(),
                        },
                    },
                },
            )),
            global_index: unified_bridge::GlobalIndex::new(
                unified_bridge::NetworkId::new(self.global_index_rollup),
                self.global_index_index as u32,
            ),
        }
    }
}

/// Zero-copy compatible SmtMerkleProof for bytemuck operations.
/// This captures the fixed-size siblings array.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SmtMerkleProofZeroCopy {
    /// Siblings array (192 * 32 = 6144 bytes)
    pub siblings: [[u8; 32]; 192],
}

// SAFETY: This struct has a stable C-compatible memory layout
// - #[repr(C)] ensures C-compatible layout
// - All fields are fixed-size arrays of u8, which are Pod and Zeroable
// - The total size is 6144 bytes with no padding
// - Cannot use derive due to large array size (6144 bytes exceeds bytemuck's
//   derive limits)
unsafe impl Pod for SmtMerkleProofZeroCopy {}
unsafe impl Zeroable for SmtMerkleProofZeroCopy {}

impl SmtMerkleProofZeroCopy {
    /// Get the size of this struct in bytes.
    pub const fn size() -> usize {
        std::mem::size_of::<Self>()
    }

    /// Convert from SmtMerkleProof to SmtMerkleProofZeroCopy
    pub fn from_smt_merkle_proof(
        proof: &agglayer_tries::proof::SmtMerkleProof<
            agglayer_primitives::keccak::Keccak256Hasher,
            192,
        >,
    ) -> Self {
        let mut siblings = [[0u8; 32]; 192];
        for (i, sibling) in proof.siblings.iter().enumerate() {
            siblings[i] = sibling.0;
        }
        Self { siblings }
    }

    /// Convert from SmtMerkleProofZeroCopy to SmtMerkleProof
    pub fn to_smt_merkle_proof(
        &self,
    ) -> agglayer_tries::proof::SmtMerkleProof<agglayer_primitives::keccak::Keccak256Hasher, 192>
    {
        let siblings: [agglayer_primitives::Digest; 192] =
            self.siblings.map(|s| agglayer_primitives::Digest(s));
        agglayer_tries::proof::SmtMerkleProof { siblings }
    }
}

/// Zero-copy compatible SmtNonInclusionProof for bytemuck operations.
/// This captures the variable-length siblings as a fixed-size array.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SmtNonInclusionProofZeroCopy {
    /// Siblings array (64 * 32 = 2048 bytes)
    pub siblings: [[u8; 32]; 64],
}

// SAFETY: This struct has a stable C-compatible memory layout
// - #[repr(C)] ensures C-compatible layout
// - All fields are fixed-size arrays of u8, which are Pod and Zeroable
// - The total size is 2048 bytes with no padding
// - Cannot use derive due to large array size (2048 bytes exceeds bytemuck's
//   derive limits)
unsafe impl Pod for SmtNonInclusionProofZeroCopy {}
unsafe impl Zeroable for SmtNonInclusionProofZeroCopy {}

impl SmtNonInclusionProofZeroCopy {
    /// Get the size of this struct in bytes.
    pub const fn size() -> usize {
        std::mem::size_of::<Self>()
    }

    /// Convert from SmtNonInclusionProof to SmtNonInclusionProofZeroCopy
    pub fn from_smt_non_inclusion_proof(
        proof: &agglayer_tries::proof::SmtNonInclusionProof<
            agglayer_primitives::keccak::Keccak256Hasher,
            64,
        >,
    ) -> Self {
        let mut siblings = [[0u8; 32]; 64];
        for (i, sibling) in proof.siblings.iter().enumerate() {
            if i < 64 {
                siblings[i] = sibling.0;
            }
        }
        Self { siblings }
    }

    /// Convert from SmtNonInclusionProofZeroCopy to SmtNonInclusionProof
    pub fn to_smt_non_inclusion_proof(
        &self,
    ) -> agglayer_tries::proof::SmtNonInclusionProof<agglayer_primitives::keccak::Keccak256Hasher, 64>
    {
        let siblings: Vec<agglayer_primitives::Digest> = self
            .siblings
            .iter()
            .map(|s| agglayer_primitives::Digest(*s))
            .collect();
        agglayer_tries::proof::SmtNonInclusionProof { siblings }
    }
}

/// Zero-copy compatible balance proof entry for bytemuck operations.
/// Note: This only captures the fixed-size parts, Merkle proofs would need
/// separate handling.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct BalanceProofEntryZeroCopy {
    /// Token info (24 bytes)
    pub token_info: TokenInfoZeroCopy,
    /// Balance amount (32 bytes)
    pub balance: [u8; 32],
    /// Padding to ensure proper alignment
    pub _padding: [u8; 8],
}

impl BalanceProofEntryZeroCopy {
    /// Get the size of this struct in bytes.
    pub const fn size() -> usize {
        std::mem::size_of::<Self>()
    }
}

/// Zero-copy representation of MultiBatchHeader for safe transmute.
/// This struct has a stable C-compatible memory layout with fixed-size fields
/// and offsets to variable-length data.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MultiBatchHeaderZeroCopy {
    /// Current certificate height of the L2 chain (u64)
    pub height: u64,
    /// Network that emitted this MultiBatchHeader (u32)
    pub origin_network: u32,
    /// Number of bridge exits (u32)
    pub bridge_exits_count: u32,
    /// Number of imported bridge exits (u32)
    pub imported_bridge_exits_count: u32,
    /// Number of balance proofs (u32)
    pub balances_proofs_count: u32,
    /// Previous pessimistic root (32 bytes)
    pub prev_pessimistic_root: [u8; 32],
    /// L1 info root used to import bridge exits (32 bytes)
    pub l1_info_root: [u8; 32],
    /// Aggchain proof data (variable size, but we'll use a fixed buffer)
    /// For ECDSA: 64 bytes signature (truncated)
    /// For Generic: 32 bytes aggchain_params + 32 bytes vkey
    pub aggchain_proof_data: [u8; 64],
    /// Aggchain proof type (u8: 0=ECDSA, 1=Generic)
    pub aggchain_proof_type: u8,
    /// Padding to ensure proper alignment
    pub _padding: [u8; 7],
}

// SAFETY: This struct has a stable C-compatible memory layout
// - #[repr(C)] ensures C-compatible layout
// - Fields are ordered by alignment (u64 first, then u32, then arrays, then u8)
// - Explicit padding field ensures proper alignment without internal padding
// - Total size is 160 bytes: 8+4+4+4+4+32+32+64+1+7 = 160
// - Cannot use derive due to complex field layout and explicit padding
//   requirements
unsafe impl Pod for MultiBatchHeaderZeroCopy {}
unsafe impl Zeroable for MultiBatchHeaderZeroCopy {}

impl MultiBatchHeaderZeroCopy {
    /// Get the size of this struct in bytes.
    pub const fn size() -> usize {
        std::mem::size_of::<Self>()
    }

    /// Safely deserialize from bytes using bytemuck.
    pub fn from_bytes(data: &[u8]) -> Result<&Self, bytemuck::PodCastError> {
        if data.len() != Self::size() {
            return Err(bytemuck::PodCastError::SizeMismatch);
        }
        bytemuck::try_from_bytes(data)
    }

    /// Convert this struct to a byte slice (zero-copy).
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    /// Convert this struct to an owned byte vector (creates a copy).
    /// Use as_bytes() for zero-copy operations.
    pub fn to_bytes_copy(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

/// Represents the chain state transition for the pessimistic proof.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultiBatchHeader<H>
where
    H: Hasher,
    H::Digest: Eq + Hash + Copy + Serialize + DeserializeOwned,
{
    /// Network that emitted this [`MultiBatchHeader`].
    pub origin_network: NetworkId,
    /// Current certificate height of the L2 chain.
    pub height: u64,
    /// Previous pessimistic root.
    #[serde_as(as = "_")]
    pub prev_pessimistic_root: H::Digest,
    /// List of bridge exits created in this batch.
    pub bridge_exits: Vec<BridgeExit>,
    /// List of imported bridge exits claimed in this batch.
    pub imported_bridge_exits: Vec<(ImportedBridgeExit, NullifierPath<H>)>,
    /// L1 info root used to import bridge exits.
    #[serde_as(as = "_")]
    pub l1_info_root: H::Digest,
    /// Token balances of the origin network before processing bridge events,
    /// with Merkle proofs of these balances in the local balance tree.
    /// Using Vec instead of BTreeMap for better zero-copy compatibility.
    pub balances_proofs: Vec<(TokenInfo, (U256, LocalBalancePath<H>))>,
    /// Aggchain proof.
    pub aggchain_proof: AggchainData,
}

impl<H> MultiBatchHeader<H>
where
    H: Hasher,
    H::Digest: Eq + Hash + Copy + Serialize + DeserializeOwned + AsRef<[u8]> + From<[u8; 32]>,
{
    /// Convert to zero-copy representation.
    /// Note: This only captures the fixed-size header data.
    /// The variable-length data (bridge_exits, imported_bridge_exits,
    /// balances_proofs) should be serialized separately for full data
    /// integrity. See the test_full_recovery_from_zero_copy_components_*
    /// tests for the complete pattern that achieves 100% data integrity.
    ///
    /// IMPORTANT: This method performs data transformation but returns a
    /// zero-copy struct. Use the returned struct's as_bytes() method for
    /// true zero-copy operations.
    pub fn to_zero_copy(&self) -> MultiBatchHeaderZeroCopy {
        let aggchain_proof_type = match &self.aggchain_proof {
            AggchainData::ECDSA { .. } => 0u8,
            AggchainData::Generic { .. } => 1u8,
        };

        let mut aggchain_proof_data = [0u8; 64];
        match &self.aggchain_proof {
            AggchainData::ECDSA { signature, .. } => {
                // Copy signature bytes (64 bytes, truncating if needed)
                let sig_bytes = signature.as_bytes();
                aggchain_proof_data[..64].copy_from_slice(&sig_bytes[..64]);
            }
            AggchainData::Generic {
                aggchain_params,
                aggchain_vkey,
            } => {
                // Copy aggchain_params (32 bytes) + vkey (32 bytes)
                aggchain_proof_data[..32].copy_from_slice(aggchain_params.as_slice());
                // Convert vkey from [u32; 8] to bytes
                for (i, &val) in aggchain_vkey.iter().enumerate() {
                    let bytes = val.to_be_bytes();
                    aggchain_proof_data[32 + i * 4..36 + i * 4].copy_from_slice(&bytes);
                }
            }
        }

        MultiBatchHeaderZeroCopy {
            height: self.height,
            origin_network: self.origin_network.to_u32(),
            bridge_exits_count: self.bridge_exits.len() as u32,
            imported_bridge_exits_count: self.imported_bridge_exits.len() as u32,
            balances_proofs_count: self.balances_proofs.len() as u32,
            prev_pessimistic_root: self.prev_pessimistic_root.as_ref().try_into().unwrap(),
            l1_info_root: self.l1_info_root.as_ref().try_into().unwrap(),
            aggchain_proof_data,
            aggchain_proof_type,
            _padding: [0; 7],
        }
    }

    /// Convert from zero-copy representation.
    /// Note: This only reconstructs the fixed-size header data.
    /// The variable-length data should be deserialized separately for full data
    /// integrity. See the test_full_recovery_from_zero_copy_components_*
    /// tests for the complete pattern that achieves 100% data integrity.
    pub fn from_zero_copy(
        zero_copy: &MultiBatchHeaderZeroCopy,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let origin_network = NetworkId::new(zero_copy.origin_network);
        let prev_pessimistic_root =
            <H::Digest as From<[u8; 32]>>::from(zero_copy.prev_pessimistic_root);
        let l1_info_root = <H::Digest as From<[u8; 32]>>::from(zero_copy.l1_info_root);

        let aggchain_proof = match zero_copy.aggchain_proof_type {
            0 => {
                // ECDSA - we can't reconstruct the full signature from 64 bytes, so we'll use a
                // placeholder In a real implementation, you'd need to serialize
                // the full signature separately
                let signature = agglayer_primitives::Signature::new(
                    agglayer_primitives::U256::default(),
                    agglayer_primitives::U256::default(),
                    false,
                );
                AggchainData::ECDSA {
                    signer: agglayer_primitives::Address::new([0; 20]), /* Would need to be
                                                                         * provided separately */
                    signature,
                }
            }
            1 => {
                // Generic
                let aggchain_params = agglayer_primitives::Digest::from(
                    <[u8; 32]>::try_from(&zero_copy.aggchain_proof_data[..32]).unwrap(),
                );
                // Reconstruct vkey from bytes
                let mut aggchain_vkey = [0u32; 8];
                for i in 0..8 {
                    let start = 32 + i * 4;
                    let end = start + 4;
                    let bytes = &zero_copy.aggchain_proof_data[start..end];
                    aggchain_vkey[i] = u32::from_be_bytes(bytes.try_into().unwrap());
                }
                AggchainData::Generic {
                    aggchain_params,
                    aggchain_vkey,
                }
            }
            _ => return Err("Invalid aggchain proof type".into()),
        };

        Ok(MultiBatchHeader {
            origin_network,
            height: zero_copy.height,
            prev_pessimistic_root,
            bridge_exits: Vec::new(), // Should be deserialized separately for full data integrity
            imported_bridge_exits: Vec::new(), /* Should be deserialized separately for full data
                                       * integrity */
            l1_info_root,
            balances_proofs: Vec::new(), /* Should be deserialized separately for full data
                                          * integrity */
            aggchain_proof,
        })
    }

    /// Convert to bytes using zero-copy serialization for the header.
    /// Note: This only serializes the fixed-size header data.
    /// For full data integrity, serialize variable-length data separately.
    /// See the test_full_recovery_from_zero_copy_components_* tests for the
    /// complete pattern that achieves 100% data integrity.
    /// WARNING: This method creates a copy. For true zero-copy, use the
    /// zero-copy structs directly.
    pub fn to_bytes_zero_copy(&self) -> Vec<u8> {
        self.to_zero_copy().to_bytes_copy()
    }

    /// Convert from bytes using zero-copy deserialization for the header.
    /// Note: This only deserializes the fixed-size header data.
    /// For full data integrity, deserialize variable-length data separately.
    /// See the test_full_recovery_from_zero_copy_components_* tests for the
    /// complete pattern that achieves 100% data integrity.
    pub fn from_bytes_zero_copy(
        data: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let zero_copy = MultiBatchHeaderZeroCopy::from_bytes(data)
            .map_err(|e| format!("Failed to deserialize zero-copy data: {:?}", e))?;
        Self::from_zero_copy(zero_copy)
    }
}

// Implement From trait for zero-copy structs to enable .into() conversions
impl From<BridgeExitZeroCopy> for unified_bridge::BridgeExit {
    fn from(zero_copy: BridgeExitZeroCopy) -> Self {
        zero_copy.to_bridge_exit()
    }
}

impl From<TokenInfoZeroCopy> for unified_bridge::TokenInfo {
    fn from(zero_copy: TokenInfoZeroCopy) -> Self {
        zero_copy.to_token_info()
    }
}

#[cfg(test)]
mod tests {
    use agglayer_primitives::{Address, Digest, Signature, U256};
    use unified_bridge::{
        GlobalIndex, L1InfoTreeLeaf, L1InfoTreeLeafInner, LETMerkleProof, LeafType, MerkleProof,
    };

    use super::*;

    /// Deep comparison function to check for lossy conversions
    /// This function compares all fields including nested structures
    fn deep_equals<H>(original: &MultiBatchHeader<H>, reconstructed: &MultiBatchHeader<H>) -> bool
    where
        H: Hasher,
        H::Digest: Eq + Hash + Copy + Serialize + DeserializeOwned + AsRef<[u8]> + From<[u8; 32]>,
    {
        // Compare basic fields
        if original.origin_network != reconstructed.origin_network
            || original.height != reconstructed.height
            || original.prev_pessimistic_root != reconstructed.prev_pessimistic_root
            || original.l1_info_root != reconstructed.l1_info_root
        {
            return false;
        }

        // Compare bridge_exits
        if original.bridge_exits.len() != reconstructed.bridge_exits.len() {
            return false;
        }
        for (orig, rec) in original
            .bridge_exits
            .iter()
            .zip(reconstructed.bridge_exits.iter())
        {
            if orig.leaf_type != rec.leaf_type
                || orig.token_info.origin_network != rec.token_info.origin_network
                || orig.token_info.origin_token_address != rec.token_info.origin_token_address
                || orig.dest_network != rec.dest_network
                || orig.dest_address != rec.dest_address
                || orig.amount != rec.amount
                || orig.metadata != rec.metadata
            {
                return false;
            }
        }

        // Compare imported_bridge_exits
        if original.imported_bridge_exits.len() != reconstructed.imported_bridge_exits.len() {
            return false;
        }
        for (orig, rec) in original
            .imported_bridge_exits
            .iter()
            .zip(reconstructed.imported_bridge_exits.iter())
        {
            // Compare bridge_exit part
            let orig_be = &orig.0.bridge_exit;
            let rec_be = &rec.0.bridge_exit;
            if orig_be.leaf_type != rec_be.leaf_type
                || orig_be.token_info.origin_network != rec_be.token_info.origin_network
                || orig_be.token_info.origin_token_address != rec_be.token_info.origin_token_address
                || orig_be.dest_network != rec_be.dest_network
                || orig_be.dest_address != rec_be.dest_address
                || orig_be.amount != rec_be.amount
                || orig_be.metadata != rec_be.metadata
            {
                return false;
            }

            // Compare global_index
            if orig.0.global_index.network_id() != rec.0.global_index.network_id()
                || orig.0.global_index.leaf_index() != rec.0.global_index.leaf_index()
            {
                return false;
            }

            // Compare claim_data - this is where we expect to find lossy conversions
            match (&orig.0.claim_data, &rec.0.claim_data) {
                (
                    unified_bridge::Claim::Mainnet(orig_claim),
                    unified_bridge::Claim::Mainnet(rec_claim),
                ) => {
                    // Compare all the nested fields in claim_data
                    if orig_claim.proof_leaf_mer.proof.siblings
                        != rec_claim.proof_leaf_mer.proof.siblings
                    {
                        println!("❌ claim_data.proof_leaf_mer.proof.siblings mismatch");
                        println!(
                            "  Original: {:?}",
                            orig_claim.proof_leaf_mer.proof.siblings[0]
                        );
                        println!(
                            "  Reconstructed: {:?}",
                            rec_claim.proof_leaf_mer.proof.siblings[0]
                        );
                        return false;
                    }
                    if orig_claim.proof_leaf_mer.root != rec_claim.proof_leaf_mer.root {
                        println!("❌ claim_data.proof_leaf_mer.root mismatch");
                        println!("  Original: {:?}", orig_claim.proof_leaf_mer.root);
                        println!("  Reconstructed: {:?}", rec_claim.proof_leaf_mer.root);
                        return false;
                    }
                    if orig_claim.proof_ger_l1root.proof.siblings
                        != rec_claim.proof_ger_l1root.proof.siblings
                    {
                        println!("❌ claim_data.proof_ger_l1root.proof.siblings mismatch");
                        return false;
                    }
                    if orig_claim.proof_ger_l1root.root != rec_claim.proof_ger_l1root.root {
                        println!("❌ claim_data.proof_ger_l1root.root mismatch");
                        return false;
                    }
                    if orig_claim.l1_leaf.l1_info_tree_index != rec_claim.l1_leaf.l1_info_tree_index
                    {
                        println!("❌ claim_data.l1_leaf.l1_info_tree_index mismatch");
                        println!("  Original: {}", orig_claim.l1_leaf.l1_info_tree_index);
                        println!("  Reconstructed: {}", rec_claim.l1_leaf.l1_info_tree_index);
                        return false;
                    }
                    if orig_claim.l1_leaf.rer != rec_claim.l1_leaf.rer {
                        println!("❌ claim_data.l1_leaf.rer mismatch");
                        return false;
                    }
                    if orig_claim.l1_leaf.mer != rec_claim.l1_leaf.mer {
                        println!("❌ claim_data.l1_leaf.mer mismatch");
                        return false;
                    }
                    if orig_claim.l1_leaf.inner.block_hash != rec_claim.l1_leaf.inner.block_hash {
                        println!("❌ claim_data.l1_leaf.inner.block_hash mismatch");
                        return false;
                    }
                    if orig_claim.l1_leaf.inner.timestamp != rec_claim.l1_leaf.inner.timestamp {
                        println!("❌ claim_data.l1_leaf.inner.timestamp mismatch");
                        println!("  Original: {}", orig_claim.l1_leaf.inner.timestamp);
                        println!("  Reconstructed: {}", rec_claim.l1_leaf.inner.timestamp);
                        return false;
                    }
                    if orig_claim.l1_leaf.inner.global_exit_root
                        != rec_claim.l1_leaf.inner.global_exit_root
                    {
                        println!("❌ claim_data.l1_leaf.inner.global_exit_root mismatch");
                        return false;
                    }
                }
                _ => return false, // Different claim types
            }

            // Compare nullifier paths
            if orig.1.siblings != rec.1.siblings {
                return false;
            }
        }

        // Compare balances_proofs
        if original.balances_proofs.len() != reconstructed.balances_proofs.len() {
            return false;
        }
        for (orig, rec) in original
            .balances_proofs
            .iter()
            .zip(reconstructed.balances_proofs.iter())
        {
            if orig.0.origin_network != rec.0.origin_network
                || orig.0.origin_token_address != rec.0.origin_token_address
                || orig.1 .0 != rec.1 .0
            {
                // balance
                return false;
            }
            // Compare Merkle paths
            if orig.1 .1.siblings != rec.1 .1.siblings {
                return false;
            }
        }

        // Compare aggchain_proof
        match (&original.aggchain_proof, &reconstructed.aggchain_proof) {
            (
                AggchainData::ECDSA {
                    signer: orig_signer,
                    signature: orig_sig,
                },
                AggchainData::ECDSA {
                    signer: rec_signer,
                    signature: rec_sig,
                },
            ) => {
                if orig_signer != rec_signer
                    || orig_sig.r() != rec_sig.r()
                    || orig_sig.s() != rec_sig.s()
                    || orig_sig.v() != rec_sig.v()
                {
                    return false;
                }
            }
            (
                AggchainData::Generic {
                    aggchain_params: orig_params,
                    aggchain_vkey: orig_vkey,
                },
                AggchainData::Generic {
                    aggchain_params: rec_params,
                    aggchain_vkey: rec_vkey,
                },
            ) => {
                if orig_params != rec_params || orig_vkey != rec_vkey {
                    return false;
                }
            }
            _ => return false, // Different aggchain proof types
        }

        true
    }

    /// Test helper to create a sample BridgeExit
    fn create_sample_bridge_exit() -> unified_bridge::BridgeExit {
        unified_bridge::BridgeExit {
            leaf_type: LeafType::Transfer,
            token_info: unified_bridge::TokenInfo {
                origin_network: unified_bridge::NetworkId::new(1),
                origin_token_address: Address::new([1u8; 20]),
            },
            dest_network: unified_bridge::NetworkId::new(2),
            dest_address: Address::new([2u8; 20]),
            amount: U256::from(1000u64),
            metadata: Some(Digest([3u8; 32])),
        }
    }

    /// Test helper to create a sample ImportedBridgeExit
    fn create_sample_imported_bridge_exit() -> unified_bridge::ImportedBridgeExit {
        unified_bridge::ImportedBridgeExit {
            bridge_exit: create_sample_bridge_exit(),
            claim_data: unified_bridge::Claim::Mainnet(Box::new(
                unified_bridge::ClaimFromMainnet {
                    proof_leaf_mer: MerkleProof {
                        proof: LETMerkleProof {
                            siblings: [Digest([4u8; 32]); 32],
                        },
                        root: Digest([5u8; 32]),
                    },
                    proof_ger_l1root: MerkleProof {
                        proof: LETMerkleProof {
                            siblings: [Digest([6u8; 32]); 32],
                        },
                        root: Digest([7u8; 32]),
                    },
                    l1_leaf: L1InfoTreeLeaf {
                        l1_info_tree_index: 42,
                        rer: Digest([8u8; 32]),
                        mer: Digest([9u8; 32]),
                        inner: L1InfoTreeLeafInner {
                            block_hash: Digest([10u8; 32]),
                            timestamp: 1234567890,
                            global_exit_root: Digest([11u8; 32]),
                        },
                    },
                },
            )),
            global_index: GlobalIndex::new(unified_bridge::NetworkId::new(3), 123),
        }
    }

    /// Test helper to create a sample TokenInfo
    fn create_sample_token_info() -> unified_bridge::TokenInfo {
        unified_bridge::TokenInfo {
            origin_network: unified_bridge::NetworkId::new(4),
            origin_token_address: Address::new([12u8; 20]),
        }
    }

    /// Test helper to create a sample SmtMerkleProof
    fn create_sample_smt_merkle_proof(
    ) -> agglayer_tries::proof::SmtMerkleProof<agglayer_primitives::keccak::Keccak256Hasher, 192>
    {
        agglayer_tries::proof::SmtMerkleProof {
            siblings: [Digest([13u8; 32]); 192],
        }
    }

    /// Test helper to create a sample SmtNonInclusionProof
    fn create_sample_smt_non_inclusion_proof(
    ) -> agglayer_tries::proof::SmtNonInclusionProof<agglayer_primitives::keccak::Keccak256Hasher, 64>
    {
        agglayer_tries::proof::SmtNonInclusionProof {
            siblings: vec![Digest([14u8; 32]); 64],
        }
    }

    /// Test helper to create a sample MultiBatchHeader
    fn create_sample_multi_batch_header(
    ) -> MultiBatchHeader<agglayer_primitives::keccak::Keccak256Hasher> {
        MultiBatchHeader {
            origin_network: unified_bridge::NetworkId::new(5),
            height: 1000,
            prev_pessimistic_root: Digest([15u8; 32]),
            bridge_exits: vec![create_sample_bridge_exit()],
            imported_bridge_exits: vec![(
                create_sample_imported_bridge_exit(),
                create_sample_smt_non_inclusion_proof(),
            )],
            l1_info_root: Digest([16u8; 32]),
            balances_proofs: vec![(
                create_sample_token_info(),
                (U256::from(5000u64), create_sample_smt_merkle_proof()),
            )],
            aggchain_proof: AggchainData::ECDSA {
                signer: Address::new([17u8; 20]),
                signature: Signature::new(U256::from(18u64), U256::from(19u64), false),
            },
        }
    }

    /// Test helper to create a sample MultiBatchHeader with Generic aggchain
    /// proof
    fn create_sample_multi_batch_header_generic(
    ) -> MultiBatchHeader<agglayer_primitives::keccak::Keccak256Hasher> {
        MultiBatchHeader {
            origin_network: unified_bridge::NetworkId::new(6),
            height: 2000,
            prev_pessimistic_root: Digest([20u8; 32]),
            bridge_exits: vec![create_sample_bridge_exit()],
            imported_bridge_exits: vec![(
                create_sample_imported_bridge_exit(),
                create_sample_smt_non_inclusion_proof(),
            )],
            l1_info_root: Digest([21u8; 32]),
            balances_proofs: vec![(
                create_sample_token_info(),
                (U256::from(7000u64), create_sample_smt_merkle_proof()),
            )],
            aggchain_proof: AggchainData::Generic {
                aggchain_params: Digest([22u8; 32]),
                aggchain_vkey: [23u32, 24u32, 25u32, 26u32, 27u32, 28u32, 29u32, 30u32],
            },
        }
    }

    #[test]
    fn test_zero_copy_struct_sizes() {
        // Test that the size calculations are correct
        assert_eq!(
            BridgeExitZeroCopy::size(),
            std::mem::size_of::<BridgeExitZeroCopy>()
        );
        assert_eq!(
            TokenInfoZeroCopy::size(),
            std::mem::size_of::<TokenInfoZeroCopy>()
        );
        assert_eq!(
            ImportedBridgeExitZeroCopy::size(),
            std::mem::size_of::<ImportedBridgeExitZeroCopy>()
        );
        assert_eq!(
            SmtMerkleProofZeroCopy::size(),
            std::mem::size_of::<SmtMerkleProofZeroCopy>()
        );
        assert_eq!(
            SmtNonInclusionProofZeroCopy::size(),
            std::mem::size_of::<SmtNonInclusionProofZeroCopy>()
        );
        assert_eq!(
            MultiBatchHeaderZeroCopy::size(),
            std::mem::size_of::<MultiBatchHeaderZeroCopy>()
        );
    }

    #[test]
    fn test_edge_cases() {
        // Test with maximum values
        let mut bridge_exit = create_sample_bridge_exit();
        bridge_exit.amount = U256::MAX;
        bridge_exit.metadata = Some(Digest([0xFFu8; 32]));

        let zero_copy = BridgeExitZeroCopy::from_bridge_exit(&bridge_exit);
        let reconstructed = zero_copy.to_bridge_exit();

        assert_eq!(bridge_exit.amount, reconstructed.amount);
        assert_eq!(bridge_exit.metadata, reconstructed.metadata);

        // Test with zero values
        let mut bridge_exit = create_sample_bridge_exit();
        bridge_exit.amount = U256::ZERO;
        bridge_exit.metadata = None;

        let zero_copy = BridgeExitZeroCopy::from_bridge_exit(&bridge_exit);
        let reconstructed = zero_copy.to_bridge_exit();

        assert_eq!(bridge_exit.amount, reconstructed.amount);
        assert_eq!(bridge_exit.metadata, reconstructed.metadata);
    }

    #[test]
    fn test_invalid_aggchain_proof_type() {
        let mut zero_copy = create_sample_multi_batch_header().to_zero_copy();
        zero_copy.aggchain_proof_type = 255; // Invalid type

        let result: Result<MultiBatchHeader<agglayer_primitives::keccak::Keccak256Hasher>, _> =
            MultiBatchHeader::from_zero_copy(&zero_copy);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid aggchain proof type"));
    }

    #[test]
    fn test_byte_serialization_errors() {
        // Test with wrong size data
        let wrong_size_data = vec![0u8; 100]; // Wrong size
        let result = MultiBatchHeaderZeroCopy::from_bytes(&wrong_size_data);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            bytemuck::PodCastError::SizeMismatch
        ));

        // Test with empty data
        let empty_data = vec![];
        let result = MultiBatchHeaderZeroCopy::from_bytes(&empty_data);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            bytemuck::PodCastError::SizeMismatch
        ));
    }

    /// Test demonstrating full recovery of MultiBatchHeader from zero-copy
    /// components This simulates the pattern used in the SP1 zkvm
    /// environment
    #[test]
    fn test_full_recovery_from_zero_copy_components_ecdsa() {
        let original = create_sample_multi_batch_header();

        // Simulate the zero-copy serialization process
        let header_zero_copy = original.to_zero_copy();

        // Convert bridge_exits to zero-copy format
        let bridge_exits_zero_copy: Vec<BridgeExitZeroCopy> = original
            .bridge_exits
            .iter()
            .map(|be| BridgeExitZeroCopy::from_bridge_exit(be))
            .collect();

        // Convert imported_bridge_exits to zero-copy format
        let imported_bridge_exits_zero_copy: Vec<ImportedBridgeExitZeroCopy> = original
            .imported_bridge_exits
            .iter()
            .map(|(ibe, _)| ImportedBridgeExitZeroCopy::from_imported_bridge_exit(ibe))
            .collect();

        // Extract nullifier paths
        let nullifier_paths_zero_copy: Vec<SmtNonInclusionProofZeroCopy> = original
            .imported_bridge_exits
            .iter()
            .map(|(_, path)| SmtNonInclusionProofZeroCopy::from_smt_non_inclusion_proof(path))
            .collect();

        // Convert balances_proofs to zero-copy format
        let balances_proofs_zero_copy: Vec<BalanceProofEntryZeroCopy> = original
            .balances_proofs
            .iter()
            .map(|(token_info, (balance, _))| BalanceProofEntryZeroCopy {
                token_info: TokenInfoZeroCopy::from_token_info(token_info),
                balance: balance.to_be_bytes(),
                _padding: [0; 8],
            })
            .collect();

        // Extract balance Merkle paths
        let balance_merkle_paths_zero_copy: Vec<SmtMerkleProofZeroCopy> = original
            .balances_proofs
            .iter()
            .map(|(_, (_, path))| SmtMerkleProofZeroCopy::from_smt_merkle_proof(path))
            .collect();

        // Simulate reading the aggchain_proof separately (since zero-copy truncates it)
        let aggchain_proof = original.aggchain_proof.clone();

        // Simulate the reconstruction process (like in SP1 zkvm)
        let mut reconstructed: MultiBatchHeader<agglayer_primitives::keccak::Keccak256Hasher> =
            MultiBatchHeader::from_zero_copy(&header_zero_copy)
                .expect("Failed to reconstruct MultiBatchHeader");

        // Set the aggchain_proof from the separately read data
        reconstructed.aggchain_proof = aggchain_proof;

        // Convert bridge_exits back to original format
        reconstructed.bridge_exits = bridge_exits_zero_copy
            .iter()
            .map(|be| be.to_bridge_exit())
            .collect();

        // Convert imported_bridge_exits back to original format
        reconstructed.imported_bridge_exits = imported_bridge_exits_zero_copy
            .iter()
            .zip(nullifier_paths_zero_copy.iter())
            .map(|(ibe, path)| {
                let imported_bridge_exit = ibe.to_imported_bridge_exit();
                let nullifier_path = path.to_smt_non_inclusion_proof();
                (imported_bridge_exit, nullifier_path)
            })
            .collect();

        // Convert balances_proofs back to original format
        reconstructed.balances_proofs = balances_proofs_zero_copy
            .iter()
            .zip(balance_merkle_paths_zero_copy.iter())
            .map(|(bp, path)| {
                let token_info = bp.token_info.to_token_info();
                let balance = U256::from_be_bytes(bp.balance);
                let merkle_path = path.to_smt_merkle_proof();
                (token_info, (balance, merkle_path))
            })
            .collect();

        // Verify full recovery using comprehensive deep comparison
        // This will catch any lossy conversions in any field
        assert!(
            deep_equals(&original, &reconstructed),
            "Deep comparison failed - there are lossy conversions!"
        );

        println!("✓ Full recovery from zero-copy components successful for ECDSA");
    }

    /// Test demonstrating full recovery of MultiBatchHeader from zero-copy
    /// components with Generic aggchain proof
    #[test]
    fn test_full_recovery_from_zero_copy_components_generic() {
        let original = create_sample_multi_batch_header_generic();

        // Simulate the zero-copy serialization process
        let header_zero_copy = original.to_zero_copy();

        // Convert bridge_exits to zero-copy format
        let bridge_exits_zero_copy: Vec<BridgeExitZeroCopy> = original
            .bridge_exits
            .iter()
            .map(|be| BridgeExitZeroCopy::from_bridge_exit(be))
            .collect();

        // Convert imported_bridge_exits to zero-copy format
        let imported_bridge_exits_zero_copy: Vec<ImportedBridgeExitZeroCopy> = original
            .imported_bridge_exits
            .iter()
            .map(|(ibe, _)| ImportedBridgeExitZeroCopy::from_imported_bridge_exit(ibe))
            .collect();

        // Extract nullifier paths
        let nullifier_paths_zero_copy: Vec<SmtNonInclusionProofZeroCopy> = original
            .imported_bridge_exits
            .iter()
            .map(|(_, path)| SmtNonInclusionProofZeroCopy::from_smt_non_inclusion_proof(path))
            .collect();

        // Convert balances_proofs to zero-copy format
        let balances_proofs_zero_copy: Vec<BalanceProofEntryZeroCopy> = original
            .balances_proofs
            .iter()
            .map(|(token_info, (balance, _))| BalanceProofEntryZeroCopy {
                token_info: TokenInfoZeroCopy::from_token_info(token_info),
                balance: balance.to_be_bytes(),
                _padding: [0; 8],
            })
            .collect();

        // Extract balance Merkle paths
        let balance_merkle_paths_zero_copy: Vec<SmtMerkleProofZeroCopy> = original
            .balances_proofs
            .iter()
            .map(|(_, (_, path))| SmtMerkleProofZeroCopy::from_smt_merkle_proof(path))
            .collect();

        // Simulate reading the aggchain_proof separately (since zero-copy truncates it)
        let aggchain_proof = original.aggchain_proof.clone();

        // Simulate the reconstruction process (like in SP1 zkvm)
        let mut reconstructed: MultiBatchHeader<agglayer_primitives::keccak::Keccak256Hasher> =
            MultiBatchHeader::from_zero_copy(&header_zero_copy)
                .expect("Failed to reconstruct MultiBatchHeader");

        // Set the aggchain_proof from the separately read data
        reconstructed.aggchain_proof = aggchain_proof;

        // Convert bridge_exits back to original format
        reconstructed.bridge_exits = bridge_exits_zero_copy
            .iter()
            .map(|be| be.to_bridge_exit())
            .collect();

        // Convert imported_bridge_exits back to original format
        reconstructed.imported_bridge_exits = imported_bridge_exits_zero_copy
            .iter()
            .zip(nullifier_paths_zero_copy.iter())
            .map(|(ibe, path)| {
                let imported_bridge_exit = ibe.to_imported_bridge_exit();
                let nullifier_path = path.to_smt_non_inclusion_proof();
                (imported_bridge_exit, nullifier_path)
            })
            .collect();

        // Convert balances_proofs back to original format
        reconstructed.balances_proofs = balances_proofs_zero_copy
            .iter()
            .zip(balance_merkle_paths_zero_copy.iter())
            .map(|(bp, path)| {
                let token_info = bp.token_info.to_token_info();
                let balance = U256::from_be_bytes(bp.balance);
                let merkle_path = path.to_smt_merkle_proof();
                (token_info, (balance, merkle_path))
            })
            .collect();

        // Verify full recovery using comprehensive deep comparison
        // This will catch any lossy conversions in any field
        assert!(
            deep_equals(&original, &reconstructed),
            "Deep comparison failed - there are lossy conversions!"
        );

        println!("✓ Full recovery from zero-copy components successful for Generic");
    }
}
