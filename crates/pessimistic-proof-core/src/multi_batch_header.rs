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
    /// Global index index (u64)
    pub global_index_index: u64,
    /// Global index network (u32)
    pub global_index_network: u32,
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
            global_index_network: imported_bridge_exit.global_index.network_id().to_u32(),
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
                unified_bridge::NetworkId::new(self.global_index_network),
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
    /// balances_proofs) would need to be serialized separately or handled
    /// differently.
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
    /// The variable-length data would need to be deserialized separately.
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
            bridge_exits: Vec::new(), // Would need to be deserialized separately
            imported_bridge_exits: Vec::new(), // Would need to be deserialized separately
            l1_info_root,
            balances_proofs: Vec::new(), // Would need to be deserialized separately
            aggchain_proof,
        })
    }

    /// Convert to bytes using zero-copy serialization for the header.
    /// Note: This only serializes the fixed-size header data.
    /// WARNING: This method creates a copy. For true zero-copy, use the
    /// zero-copy structs directly.
    pub fn to_bytes_zero_copy(&self) -> Vec<u8> {
        self.to_zero_copy().to_bytes_copy()
    }

    /// Convert from bytes using zero-copy deserialization for the header.
    /// Note: This only deserializes the fixed-size header data.
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
    use agglayer_primitives::keccak::Keccak256Hasher;

    use super::*;

    #[test]
    fn test_zero_copy_roundtrip() {
        let header = MultiBatchHeader::<Keccak256Hasher> {
            origin_network: NetworkId::new(1),
            height: 123,
            prev_pessimistic_root: agglayer_primitives::Digest::default(),
            bridge_exits: vec![],
            imported_bridge_exits: vec![],
            l1_info_root: agglayer_primitives::Digest::default(),
            balances_proofs: Vec::new(),
            aggchain_proof: AggchainData::ECDSA {
                signer: agglayer_primitives::Address::new([0; 20]),
                signature: agglayer_primitives::Signature::new(
                    agglayer_primitives::U256::default(),
                    agglayer_primitives::U256::default(),
                    false,
                ),
            },
        };

        let zero_copy = header.to_zero_copy();
        let reconstructed =
            MultiBatchHeader::<Keccak256Hasher>::from_zero_copy(&zero_copy).unwrap();

        assert_eq!(reconstructed.origin_network, header.origin_network);
        assert_eq!(reconstructed.height, header.height);
        assert_eq!(
            reconstructed.prev_pessimistic_root,
            header.prev_pessimistic_root
        );
        assert_eq!(reconstructed.l1_info_root, header.l1_info_root);
        // Note: Variable-length fields are not reconstructed in zero-copy
    }

    #[test]
    fn test_zero_copy_bytes_roundtrip() {
        let header = MultiBatchHeader::<Keccak256Hasher> {
            origin_network: NetworkId::new(2),
            height: 456,
            prev_pessimistic_root: agglayer_primitives::Digest::default(),
            bridge_exits: vec![],
            imported_bridge_exits: vec![],
            l1_info_root: agglayer_primitives::Digest::default(),
            balances_proofs: Vec::new(),
            aggchain_proof: AggchainData::ECDSA {
                signer: agglayer_primitives::Address::new([0; 20]),
                signature: agglayer_primitives::Signature::new(
                    agglayer_primitives::U256::default(),
                    agglayer_primitives::U256::default(),
                    false,
                ),
            },
        };

        let bytes = header.to_bytes_zero_copy();
        let reconstructed =
            MultiBatchHeader::<Keccak256Hasher>::from_bytes_zero_copy(&bytes).unwrap();

        assert_eq!(reconstructed.origin_network, header.origin_network);
        assert_eq!(reconstructed.height, header.height);
        assert_eq!(
            reconstructed.prev_pessimistic_root,
            header.prev_pessimistic_root
        );
        assert_eq!(reconstructed.l1_info_root, header.l1_info_root);
    }

    #[test]
    fn test_zero_copy_with_non_empty_vectors() {
        // Create a header with non-empty vectors
        let bridge_exit = BridgeExit {
            leaf_type: unified_bridge::LeafType::Transfer,
            token_info: TokenInfo {
                origin_network: NetworkId::new(1),
                origin_token_address: agglayer_primitives::Address::new([1; 20]),
            },
            dest_network: NetworkId::new(2),
            dest_address: agglayer_primitives::Address::new([2; 20]),
            amount: agglayer_primitives::U256::from(1000),
            metadata: None,
        };

        let imported_bridge_exit = ImportedBridgeExit {
            bridge_exit: bridge_exit.clone(),
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
            global_index: unified_bridge::GlobalIndex::new(NetworkId::new(1), 0),
        };

        let nullifier_path = crate::nullifier_tree::NullifierPath::<Keccak256Hasher> {
            siblings: vec![agglayer_primitives::Digest([0u8; 32]); 64],
        };

        let balance_path = crate::local_balance_tree::LocalBalancePath::<Keccak256Hasher> {
            siblings: [agglayer_primitives::Digest([0u8; 32]);
                crate::local_balance_tree::LOCAL_BALANCE_TREE_DEPTH],
        };

        let header = MultiBatchHeader::<Keccak256Hasher> {
            origin_network: NetworkId::new(1),
            height: 123,
            prev_pessimistic_root: agglayer_primitives::Digest::default(),
            bridge_exits: vec![bridge_exit],
            imported_bridge_exits: vec![(imported_bridge_exit, nullifier_path)],
            l1_info_root: agglayer_primitives::Digest::default(),
            balances_proofs: vec![(
                TokenInfo {
                    origin_network: NetworkId::new(1),
                    origin_token_address: agglayer_primitives::Address::new([3; 20]),
                },
                (agglayer_primitives::U256::from(500), balance_path),
            )],
            aggchain_proof: AggchainData::ECDSA {
                signer: agglayer_primitives::Address::new([0; 20]),
                signature: agglayer_primitives::Signature::new(
                    agglayer_primitives::U256::default(),
                    agglayer_primitives::U256::default(),
                    false,
                ),
            },
        };

        // Test zero-copy conversion
        let zero_copy = header.to_zero_copy();

        // Verify the counts are correct
        assert_eq!(zero_copy.bridge_exits_count, 1);
        assert_eq!(zero_copy.imported_bridge_exits_count, 1);
        assert_eq!(zero_copy.balances_proofs_count, 1);
        assert_eq!(zero_copy.origin_network, 1);
        assert_eq!(zero_copy.height, 123);
        assert_eq!(zero_copy.aggchain_proof_type, 0); // ECDSA

        // Test reconstruction (note: vectors will be empty in reconstruction)
        let reconstructed =
            MultiBatchHeader::<Keccak256Hasher>::from_zero_copy(&zero_copy).unwrap();

        assert_eq!(reconstructed.origin_network, header.origin_network);
        assert_eq!(reconstructed.height, header.height);
        assert_eq!(
            reconstructed.prev_pessimistic_root,
            header.prev_pessimistic_root
        );
        assert_eq!(reconstructed.l1_info_root, header.l1_info_root);
        assert_eq!(reconstructed.bridge_exits.len(), 0); // Not reconstructed in zero-copy
        assert_eq!(reconstructed.imported_bridge_exits.len(), 0); // Not reconstructed in zero-copy
        assert_eq!(reconstructed.balances_proofs.len(), 0); // Not reconstructed
                                                            // in zero-copy
    }

    #[test]
    fn test_bridge_exit_zero_copy_roundtrip() {
        let bridge_exit = BridgeExit {
            leaf_type: unified_bridge::LeafType::Transfer,
            token_info: TokenInfo {
                origin_network: NetworkId::new(1),
                origin_token_address: agglayer_primitives::Address::new([1; 20]),
            },
            dest_network: NetworkId::new(2),
            dest_address: agglayer_primitives::Address::new([2; 20]),
            amount: agglayer_primitives::U256::from(1000),
            metadata: Some(agglayer_primitives::Digest([3; 32])),
        };

        let zero_copy = BridgeExitZeroCopy::from_bridge_exit(&bridge_exit);
        let reconstructed = zero_copy.to_bridge_exit();

        assert_eq!(reconstructed.leaf_type, bridge_exit.leaf_type);
        assert_eq!(
            reconstructed.token_info.origin_network,
            bridge_exit.token_info.origin_network
        );
        assert_eq!(
            reconstructed.token_info.origin_token_address,
            bridge_exit.token_info.origin_token_address
        );
        assert_eq!(reconstructed.dest_network, bridge_exit.dest_network);
        assert_eq!(reconstructed.dest_address, bridge_exit.dest_address);
        assert_eq!(reconstructed.amount, bridge_exit.amount);
        assert_eq!(reconstructed.metadata, bridge_exit.metadata);
    }

    #[test]
    fn test_token_info_zero_copy_roundtrip() {
        let token_info = TokenInfo {
            origin_network: NetworkId::new(1),
            origin_token_address: agglayer_primitives::Address::new([1; 20]),
        };

        let zero_copy = TokenInfoZeroCopy::from_token_info(&token_info);
        let reconstructed = zero_copy.to_token_info();

        assert_eq!(reconstructed.origin_network, token_info.origin_network);
        assert_eq!(
            reconstructed.origin_token_address,
            token_info.origin_token_address
        );
    }

    #[test]
    fn test_zero_copy_alignment() {
        // Test that the zero-copy structs have proper alignment
        assert_eq!(std::mem::align_of::<MultiBatchHeaderZeroCopy>(), 8);
        assert_eq!(std::mem::align_of::<BridgeExitZeroCopy>(), 4);
        assert_eq!(std::mem::align_of::<TokenInfoZeroCopy>(), 4);

        // Test sizes
        assert_eq!(
            MultiBatchHeaderZeroCopy::size(),
            std::mem::size_of::<MultiBatchHeaderZeroCopy>()
        );
        assert_eq!(
            BridgeExitZeroCopy::size(),
            std::mem::size_of::<BridgeExitZeroCopy>()
        );
        assert_eq!(
            TokenInfoZeroCopy::size(),
            std::mem::size_of::<TokenInfoZeroCopy>()
        );

        // Verify sizes are reasonable
        assert!(MultiBatchHeaderZeroCopy::size() > 0);
        assert!(BridgeExitZeroCopy::size() > 0);
        assert!(TokenInfoZeroCopy::size() > 0);
    }

    #[test]
    fn test_bytemuck_operations_simulating_main() {
        // Test the exact operations that main.rs would perform
        let header = MultiBatchHeader::<Keccak256Hasher> {
            origin_network: NetworkId::new(1),
            height: 123,
            prev_pessimistic_root: agglayer_primitives::Digest::default(),
            bridge_exits: vec![],
            imported_bridge_exits: vec![],
            l1_info_root: agglayer_primitives::Digest::default(),
            balances_proofs: Vec::new(),
            aggchain_proof: AggchainData::ECDSA {
                signer: agglayer_primitives::Address::new([0; 20]),
                signature: agglayer_primitives::Signature::new(
                    agglayer_primitives::U256::default(),
                    agglayer_primitives::U256::default(),
                    false,
                ),
            },
        };

        // Simulate the test suite writing data
        let header_zero_copy = header.to_zero_copy();
        let header_bytes = header_zero_copy.to_bytes_copy();

        // Simulate main.rs reading data (this is where the alignment error occurred)
        // Test both approaches that were tried in main.rs:

        // Approach 1: Direct bytemuck::from_bytes (this failed)
        let result1 = bytemuck::from_bytes::<MultiBatchHeaderZeroCopy>(&header_bytes);
        assert!(result1.origin_network == 1);
        assert!(result1.height == 123);

        // Approach 2: bytemuck::try_from_bytes (this also failed)
        let result2 = bytemuck::try_from_bytes::<MultiBatchHeaderZeroCopy>(&header_bytes);
        assert!(result2.is_ok());
        let header_copy2 = result2.unwrap();
        assert!(header_copy2.origin_network == 1);
        assert!(header_copy2.height == 123);

        // Approach 3: Copy to aligned buffer (this was the attempted fix)
        let mut aligned_buffer = [0u8; std::mem::size_of::<MultiBatchHeaderZeroCopy>()];
        aligned_buffer.copy_from_slice(&header_bytes);
        let result3 = bytemuck::from_bytes::<MultiBatchHeaderZeroCopy>(&aligned_buffer);
        assert!(result3.origin_network == 1);
        assert!(result3.height == 123);
    }

    #[test]
    fn test_vector_operations_simulating_main() {
        // Test the vector operations that main.rs would perform
        let bridge_exit = BridgeExit {
            leaf_type: unified_bridge::LeafType::Transfer,
            token_info: TokenInfo {
                origin_network: NetworkId::new(1),
                origin_token_address: agglayer_primitives::Address::new([1; 20]),
            },
            dest_network: NetworkId::new(2),
            dest_address: agglayer_primitives::Address::new([2; 20]),
            amount: agglayer_primitives::U256::from(1000),
            metadata: None,
        };

        // Simulate test suite creating zero-copy vectors
        let bridge_exits_zero_copy: Vec<BridgeExitZeroCopy> =
            vec![BridgeExitZeroCopy::from_bridge_exit(&bridge_exit)];
        let bridge_exits_bytes = bytemuck::cast_slice(&bridge_exits_zero_copy);

        println!(
            "BridgeExitZeroCopy size: {}",
            std::mem::size_of::<BridgeExitZeroCopy>()
        );
        println!(
            "BridgeExitZeroCopy alignment: {}",
            std::mem::align_of::<BridgeExitZeroCopy>()
        );
        println!("bridge_exits_bytes len: {}", bridge_exits_bytes.len());
        println!("bridge_exits_bytes ptr: {:p}", bridge_exits_bytes.as_ptr());
        println!(
            "bridge_exits_bytes ptr alignment: {}",
            (bridge_exits_bytes.as_ptr() as usize) % std::mem::align_of::<BridgeExitZeroCopy>()
        );

        // Simulate main.rs reading the vector data
        // Try direct cast_slice instead of try_cast_vec
        let reconstructed_bridge_exits =
            bytemuck::cast_slice::<u8, BridgeExitZeroCopy>(&bridge_exits_bytes);
        assert_eq!(reconstructed_bridge_exits.len(), 1);

        // Test conversion back to original type
        let reconstructed_bridge_exit = reconstructed_bridge_exits[0].to_bridge_exit();
        assert_eq!(reconstructed_bridge_exit.leaf_type, bridge_exit.leaf_type);
        assert_eq!(
            reconstructed_bridge_exit.token_info.origin_network,
            bridge_exit.token_info.origin_network
        );
        assert_eq!(
            reconstructed_bridge_exit.dest_network,
            bridge_exit.dest_network
        );
        assert_eq!(reconstructed_bridge_exit.amount, bridge_exit.amount);
    }

    #[test]
    fn test_full_roundtrip_simulating_main() {
        // Test the complete roundtrip that main.rs would perform
        let header = MultiBatchHeader::<Keccak256Hasher> {
            origin_network: NetworkId::new(1),
            height: 123,
            prev_pessimistic_root: agglayer_primitives::Digest::default(),
            bridge_exits: vec![BridgeExit {
                leaf_type: unified_bridge::LeafType::Transfer,
                token_info: TokenInfo {
                    origin_network: NetworkId::new(1),
                    origin_token_address: agglayer_primitives::Address::new([1; 20]),
                },
                dest_network: NetworkId::new(2),
                dest_address: agglayer_primitives::Address::new([2; 20]),
                amount: agglayer_primitives::U256::from(1000),
                metadata: None,
            }],
            imported_bridge_exits: vec![],
            l1_info_root: agglayer_primitives::Digest::default(),
            balances_proofs: vec![(
                TokenInfo {
                    origin_network: NetworkId::new(1),
                    origin_token_address: agglayer_primitives::Address::new([3; 20]),
                },
                (
                    agglayer_primitives::U256::from(500),
                    crate::local_balance_tree::LocalBalancePath::<Keccak256Hasher> {
                        siblings: [agglayer_primitives::Digest([0u8; 32]);
                            crate::local_balance_tree::LOCAL_BALANCE_TREE_DEPTH],
                    },
                ),
            )],
            aggchain_proof: AggchainData::ECDSA {
                signer: agglayer_primitives::Address::new([0; 20]),
                signature: agglayer_primitives::Signature::new(
                    agglayer_primitives::U256::default(),
                    agglayer_primitives::U256::default(),
                    false,
                ),
            },
        };

        // Simulate test suite preparing data
        let header_zero_copy = header.to_zero_copy();
        let header_bytes = header_zero_copy.to_bytes_copy();

        let bridge_exits_zero_copy: Vec<BridgeExitZeroCopy> = header
            .bridge_exits
            .iter()
            .map(|be| BridgeExitZeroCopy::from_bridge_exit(be))
            .collect();
        let bridge_exits_bytes = bytemuck::cast_slice(&bridge_exits_zero_copy);

        let balances_proofs_zero_copy: Vec<TokenInfoZeroCopy> = header
            .balances_proofs
            .iter()
            .map(|(ti, _)| TokenInfoZeroCopy::from_token_info(ti))
            .collect();
        let balances_proofs_bytes = bytemuck::cast_slice(&balances_proofs_zero_copy);

        // Simulate main.rs reading and reconstructing data
        let mut aligned_buffer = [0u8; std::mem::size_of::<MultiBatchHeaderZeroCopy>()];
        aligned_buffer.copy_from_slice(&header_bytes);
        let header_zero_copy_read =
            bytemuck::from_bytes::<MultiBatchHeaderZeroCopy>(&aligned_buffer);

        let bridge_exits_read = bytemuck::cast_slice::<u8, BridgeExitZeroCopy>(&bridge_exits_bytes);
        let balances_proofs_read =
            bytemuck::cast_slice::<u8, TokenInfoZeroCopy>(&balances_proofs_bytes);

        // Verify the data is correct
        assert_eq!(header_zero_copy_read.origin_network, 1);
        assert_eq!(header_zero_copy_read.height, 123);
        assert_eq!(header_zero_copy_read.bridge_exits_count, 1);
        assert_eq!(header_zero_copy_read.balances_proofs_count, 1);

        assert_eq!(bridge_exits_read.len(), 1);
        assert_eq!(balances_proofs_read.len(), 1);

        // Test reconstruction (this is what main.rs would do)
        let mut reconstructed_header =
            MultiBatchHeader::<Keccak256Hasher>::from_zero_copy(header_zero_copy_read).unwrap();

        // Convert zero-copy vectors back to original types
        reconstructed_header.bridge_exits = bridge_exits_read
            .into_iter()
            .map(|be| be.to_bridge_exit())
            .collect();
        reconstructed_header.balances_proofs = balances_proofs_read
            .into_iter()
            .map(|t| {
                let balance_path = crate::local_balance_tree::LocalBalancePath::<Keccak256Hasher> {
                    siblings: [agglayer_primitives::Digest([0u8; 32]);
                        crate::local_balance_tree::LOCAL_BALANCE_TREE_DEPTH],
                };
                (
                    t.to_token_info(),
                    (agglayer_primitives::U256::default(), balance_path),
                )
            })
            .collect();

        // Verify reconstruction
        assert_eq!(reconstructed_header.origin_network, header.origin_network);
        assert_eq!(reconstructed_header.height, header.height);
        assert_eq!(
            reconstructed_header.bridge_exits.len(),
            header.bridge_exits.len()
        );
        assert_eq!(
            reconstructed_header.balances_proofs.len(),
            header.balances_proofs.len()
        );
    }

    #[test]
    fn test_alignment_issue_reproduction() {
        // This test reproduces the exact alignment issue we see in main.rs
        // The problem is that sp1_zkvm::io::read_vec() returns data that might not be
        // properly aligned

        let header = MultiBatchHeader::<Keccak256Hasher> {
            origin_network: NetworkId::new(1),
            height: 123,
            prev_pessimistic_root: agglayer_primitives::Digest::default(),
            bridge_exits: vec![],
            imported_bridge_exits: vec![],
            l1_info_root: agglayer_primitives::Digest::default(),
            balances_proofs: Vec::new(),
            aggchain_proof: AggchainData::ECDSA {
                signer: agglayer_primitives::Address::new([0; 20]),
                signature: agglayer_primitives::Signature::new(
                    agglayer_primitives::U256::default(),
                    agglayer_primitives::U256::default(),
                    false,
                ),
            },
        };

        // Simulate test suite writing data
        let header_zero_copy = header.to_zero_copy();
        let header_bytes = header_zero_copy.to_bytes_copy();

        // Simulate the alignment issue by creating unaligned data
        // This is what happens when sp1_zkvm::io::read_vec() returns data
        let mut unaligned_data = vec![0u8; header_bytes.len() + 1]; // Add one byte to force unalignment
        unaligned_data[1..].copy_from_slice(&header_bytes); // Start at offset 1 to create unalignment

        // Try to read the unaligned data - this should fail with the same error we see
        // in main.rs
        let result = bytemuck::try_from_bytes::<MultiBatchHeaderZeroCopy>(&unaligned_data[1..]);
        assert!(result.is_err());

        // The error should be TargetAlignmentGreaterAndInputNotAligned
        if let Err(bytemuck::PodCastError::TargetAlignmentGreaterAndInputNotAligned { .. }) = result
        {
            // This is the exact error we see in main.rs
            println!("Successfully reproduced the alignment error!");
        } else {
            panic!("Expected TargetAlignmentGreaterAndInputNotAligned error");
        }

        // Test that copying to an aligned buffer fixes the issue (like we tried in
        // main.rs)
        let mut aligned_buffer = [0u8; std::mem::size_of::<MultiBatchHeaderZeroCopy>()];
        aligned_buffer.copy_from_slice(&header_bytes);
        let result = bytemuck::try_from_bytes::<MultiBatchHeaderZeroCopy>(&aligned_buffer);
        assert!(result.is_ok());
    }

    #[test]
    fn test_zero_copy_roundtrip_performance() {
        // This test demonstrates the performance benefits of zero-copy serialization
        // by measuring the roundtrip time for zero-copy operations

        let header = MultiBatchHeader::<Keccak256Hasher> {
            origin_network: NetworkId::new(1),
            height: 123,
            prev_pessimistic_root: agglayer_primitives::Digest::default(),
            bridge_exits: vec![BridgeExit {
                leaf_type: unified_bridge::LeafType::Transfer,
                token_info: TokenInfo {
                    origin_network: NetworkId::new(1),
                    origin_token_address: agglayer_primitives::Address::new([1; 20]),
                },
                dest_network: NetworkId::new(2),
                dest_address: agglayer_primitives::Address::new([2; 20]),
                amount: agglayer_primitives::U256::from(1000),
                metadata: None,
            }],
            imported_bridge_exits: vec![],
            l1_info_root: agglayer_primitives::Digest::default(),
            balances_proofs: vec![(
                TokenInfo {
                    origin_network: NetworkId::new(1),
                    origin_token_address: agglayer_primitives::Address::new([3; 20]),
                },
                (
                    agglayer_primitives::U256::from(500),
                    crate::local_balance_tree::LocalBalancePath::<Keccak256Hasher> {
                        siblings: [agglayer_primitives::Digest([0u8; 32]);
                            crate::local_balance_tree::LOCAL_BALANCE_TREE_DEPTH],
                    },
                ),
            )],
            aggchain_proof: AggchainData::ECDSA {
                signer: agglayer_primitives::Address::new([0; 20]),
                signature: agglayer_primitives::Signature::new(
                    agglayer_primitives::U256::default(),
                    agglayer_primitives::U256::default(),
                    false,
                ),
            },
        };

        // Zero-copy serialization
        let zero_copy_bytes = header.to_bytes_zero_copy();

        println!("Zero-copy size: {} bytes", zero_copy_bytes.len());
        println!(
            "MultiBatchHeaderZeroCopy struct size: {} bytes",
            std::mem::size_of::<MultiBatchHeaderZeroCopy>()
        );

        // Test that zero-copy roundtrip works correctly
        let start = std::time::Instant::now();
        let reconstructed =
            MultiBatchHeader::<Keccak256Hasher>::from_bytes_zero_copy(&zero_copy_bytes).unwrap();
        let roundtrip_time = start.elapsed();

        println!("Zero-copy roundtrip time: {:?}", roundtrip_time);

        // Verify the roundtrip preserved the data
        assert_eq!(reconstructed.origin_network, header.origin_network);
        assert_eq!(reconstructed.height, header.height);
        assert_eq!(
            reconstructed.prev_pessimistic_root,
            header.prev_pessimistic_root
        );
        assert_eq!(reconstructed.l1_info_root, header.l1_info_root);

        // Note: Variable-length fields are not reconstructed in zero-copy
        // This demonstrates the limitation of the current zero-copy approach
        assert_eq!(reconstructed.bridge_exits.len(), 0);
        assert_eq!(reconstructed.imported_bridge_exits.len(), 0);
        assert_eq!(reconstructed.balances_proofs.len(), 0);
    }

    #[test]
    fn test_struct_sizes_are_padding_free() {
        // Test that all zero-copy structs have padding-free layouts
        // This verifies that our field reordering eliminated all padding

        // BridgeExitZeroCopy: u32(4) + u32(4) + [u8;20](20) + [u8;20](20) + [u8;32](32)
        // + [u8;32](32) + u8(1) + [u8;3](3) = 116 bytes
        assert_eq!(std::mem::size_of::<BridgeExitZeroCopy>(), 116);

        // TokenInfoZeroCopy: u32(4) + [u8;20](20) = 24 bytes
        assert_eq!(std::mem::size_of::<TokenInfoZeroCopy>(), 24);

        // ImportedBridgeExitZeroCopy: u64(8) + u32(4) + BridgeExitZeroCopy(116) = 128
        // bytes
        assert_eq!(std::mem::size_of::<ImportedBridgeExitZeroCopy>(), 128);

        // SmtMerkleProofZeroCopy: [[u8;32];192](6144) = 6144 bytes
        assert_eq!(std::mem::size_of::<SmtMerkleProofZeroCopy>(), 6144);

        // SmtNonInclusionProofZeroCopy: [[u8;32];64](2048) = 2048 bytes
        assert_eq!(std::mem::size_of::<SmtNonInclusionProofZeroCopy>(), 2048);

        // BalanceProofEntryZeroCopy: TokenInfoZeroCopy(24) + [u8;32](32) + [u8;8](8) =
        // 64 bytes
        assert_eq!(std::mem::size_of::<BalanceProofEntryZeroCopy>(), 64);

        // MultiBatchHeaderZeroCopy: u64(8) + u32(4) + u32(4) + u32(4) + u32(4) +
        // [u8;32](32) + [u8;32](32) + [u8;64](64) + u8(1) + [u8;7](7) = 160 bytes
        assert_eq!(std::mem::size_of::<MultiBatchHeaderZeroCopy>(), 160);

        // Verify that the size() methods return the same values
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
            BalanceProofEntryZeroCopy::size(),
            std::mem::size_of::<BalanceProofEntryZeroCopy>()
        );
        assert_eq!(
            MultiBatchHeaderZeroCopy::size(),
            std::mem::size_of::<MultiBatchHeaderZeroCopy>()
        );

        println!("All zero-copy structs have padding-free layouts!");
        println!(
            "BridgeExitZeroCopy: {} bytes",
            std::mem::size_of::<BridgeExitZeroCopy>()
        );
        println!(
            "TokenInfoZeroCopy: {} bytes",
            std::mem::size_of::<TokenInfoZeroCopy>()
        );
        println!(
            "ImportedBridgeExitZeroCopy: {} bytes",
            std::mem::size_of::<ImportedBridgeExitZeroCopy>()
        );
        println!(
            "SmtMerkleProofZeroCopy: {} bytes",
            std::mem::size_of::<SmtMerkleProofZeroCopy>()
        );
        println!(
            "SmtNonInclusionProofZeroCopy: {} bytes",
            std::mem::size_of::<SmtNonInclusionProofZeroCopy>()
        );
        println!(
            "BalanceProofEntryZeroCopy: {} bytes",
            std::mem::size_of::<BalanceProofEntryZeroCopy>()
        );
        println!(
            "MultiBatchHeaderZeroCopy: {} bytes",
            std::mem::size_of::<MultiBatchHeaderZeroCopy>()
        );
    }

    #[test]
    fn test_true_zero_copy_vs_copy_operations() {
        // This test demonstrates the difference between true zero-copy and copy
        // operations

        let header = MultiBatchHeader::<Keccak256Hasher> {
            origin_network: NetworkId::new(1),
            height: 123,
            prev_pessimistic_root: agglayer_primitives::Digest::default(),
            bridge_exits: vec![],
            imported_bridge_exits: vec![],
            l1_info_root: agglayer_primitives::Digest::default(),
            balances_proofs: Vec::new(),
            aggchain_proof: AggchainData::ECDSA {
                signer: agglayer_primitives::Address::new([0; 20]),
                signature: agglayer_primitives::Signature::new(
                    agglayer_primitives::U256::default(),
                    agglayer_primitives::U256::default(),
                    false,
                ),
            },
        };

        // Test 1: True zero-copy operations
        let zero_copy = header.to_zero_copy();

        // as_bytes() returns a reference - no allocation, no copying
        let zero_copy_bytes = zero_copy.as_bytes();
        println!("True zero-copy bytes length: {}", zero_copy_bytes.len());

        // Verify the bytes are correct
        assert_eq!(zero_copy_bytes.len(), MultiBatchHeaderZeroCopy::size());
        assert_eq!(zero_copy_bytes[0..8], zero_copy.height.to_le_bytes());

        // Test 2: Copy operations (for comparison)
        let copied_bytes = zero_copy.to_bytes_copy();
        println!("Copy bytes length: {}", copied_bytes.len());

        // Both should have the same content
        assert_eq!(zero_copy_bytes, copied_bytes.as_slice());

        // Test 3: Demonstrate that as_bytes() is truly zero-copy
        // The pointer should be the same as the original struct
        let zero_copy_ptr = zero_copy_bytes.as_ptr();
        let struct_ptr = &zero_copy as *const _ as *const u8;

        println!("Zero-copy bytes pointer: {:p}", zero_copy_ptr);
        println!("Struct pointer: {:p}", struct_ptr);

        // The pointers should be the same (or very close) for true zero-copy
        // Note: There might be a small offset due to struct layout, but they should be
        // close
        let ptr_diff = (zero_copy_ptr as usize).abs_diff(struct_ptr as usize);
        println!("Pointer difference: {} bytes", ptr_diff);

        // The difference should be small (likely 0 or a small offset)
        assert!(
            ptr_diff < 100,
            "Pointers should be close for true zero-copy"
        );

        // Test 4: Performance comparison
        let iterations = 1000;

        // Zero-copy timing
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _bytes = zero_copy.as_bytes();
        }
        let zero_copy_time = start.elapsed();

        // Copy timing
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _bytes = zero_copy.to_bytes_copy();
        }
        let copy_time = start.elapsed();

        println!(
            "Zero-copy time for {} iterations: {:?}",
            iterations, zero_copy_time
        );
        println!("Copy time for {} iterations: {:?}", iterations, copy_time);

        // Zero-copy should be significantly faster
        assert!(
            zero_copy_time < copy_time,
            "Zero-copy should be faster than copy"
        );

        println!("✅ True zero-copy operations are working correctly!");
    }
}
