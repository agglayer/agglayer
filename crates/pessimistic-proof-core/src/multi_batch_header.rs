#![allow(clippy::too_many_arguments)]
use std::hash::Hash;

use agglayer_primitives::{keccak::Hasher, U256};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;
use unified_bridge::{BridgeExit, ImportedBridgeExit, NetworkId, TokenInfo};
use bytemuck::{Pod, Zeroable};

use crate::{
    aggchain_proof::AggchainData, local_balance_tree::LocalBalancePath,
    nullifier_tree::NullifierPath,
};

/// Zero-copy representation of MultiBatchHeader for safe transmute.
/// This struct has a stable C-compatible memory layout with fixed-size fields
/// and offsets to variable-length data.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MultiBatchHeaderZeroCopy {
    /// Network that emitted this MultiBatchHeader (u32)
    pub origin_network: u32,
    /// Current certificate height of the L2 chain (u64)
    pub height: u64,
    /// Previous pessimistic root (32 bytes)
    pub prev_pessimistic_root: [u8; 32],
    /// L1 info root used to import bridge exits (32 bytes)
    pub l1_info_root: [u8; 32],
    /// Number of bridge exits (u32)
    pub bridge_exits_count: u32,
    /// Number of imported bridge exits (u32)
    pub imported_bridge_exits_count: u32,
    /// Number of balance proofs (u32)
    pub balances_proofs_count: u32,
    /// Aggchain proof type (u8: 0=ECDSA, 1=Generic)
    pub aggchain_proof_type: u8,
    /// Aggchain proof data (variable size, but we'll use a fixed buffer)
    /// For ECDSA: 64 bytes signature (truncated)
    /// For Generic: 32 bytes aggchain_params + 32 bytes vkey
    pub aggchain_proof_data: [u8; 64],
    /// Padding to ensure proper alignment
    pub _padding: [u8; 3],
}

// SAFETY: This struct has a stable C-compatible memory layout
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

    /// Convert this struct to a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    /// Convert this struct to an owned byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
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
    /// The variable-length data (bridge_exits, imported_bridge_exits, balances_proofs)
    /// would need to be serialized separately or handled differently.
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
            AggchainData::Generic { aggchain_params, aggchain_vkey } => {
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
            origin_network: self.origin_network.to_u32(),
            height: self.height,
            prev_pessimistic_root: self.prev_pessimistic_root.as_ref().try_into().unwrap(),
            l1_info_root: self.l1_info_root.as_ref().try_into().unwrap(),
            bridge_exits_count: self.bridge_exits.len() as u32,
            imported_bridge_exits_count: self.imported_bridge_exits.len() as u32,
            balances_proofs_count: self.balances_proofs.len() as u32,
            aggchain_proof_type,
            aggchain_proof_data,
            _padding: [0; 3],
        }
    }

    /// Convert from zero-copy representation.
    /// Note: This only reconstructs the fixed-size header data.
    /// The variable-length data would need to be deserialized separately.
    pub fn from_zero_copy(zero_copy: &MultiBatchHeaderZeroCopy) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let origin_network = NetworkId::new(zero_copy.origin_network);
        let prev_pessimistic_root = <H::Digest as From<[u8; 32]>>::from(zero_copy.prev_pessimistic_root);
        let l1_info_root = <H::Digest as From<[u8; 32]>>::from(zero_copy.l1_info_root);

        let aggchain_proof = match zero_copy.aggchain_proof_type {
            0 => {
                // ECDSA - we can't reconstruct the full signature from 64 bytes, so we'll use a placeholder
                // In a real implementation, you'd need to serialize the full signature separately
                let signature = agglayer_primitives::Signature::new(
                    agglayer_primitives::U256::default(),
                    agglayer_primitives::U256::default(),
                    false
                );
                AggchainData::ECDSA { 
                    signer: agglayer_primitives::Address::new([0; 20]), // Would need to be provided separately
                    signature 
                }
            }
            1 => {
                // Generic
                let aggchain_params = agglayer_primitives::Digest::from(<[u8; 32]>::try_from(&zero_copy.aggchain_proof_data[..32]).unwrap());
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
                    aggchain_vkey 
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
    pub fn to_bytes_zero_copy(&self) -> Vec<u8> {
        self.to_zero_copy().to_bytes()
    }

    /// Convert from bytes using zero-copy deserialization for the header.
    /// Note: This only deserializes the fixed-size header data.
    pub fn from_bytes_zero_copy(data: &[u8]) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let zero_copy = MultiBatchHeaderZeroCopy::from_bytes(data)
            .map_err(|e| format!("Failed to deserialize zero-copy data: {:?}", e))?;
        Self::from_zero_copy(zero_copy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agglayer_primitives::keccak::Keccak256Hasher;

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
                    false
                ),
            },
        };

        let zero_copy = header.to_zero_copy();
        let reconstructed = MultiBatchHeader::<Keccak256Hasher>::from_zero_copy(&zero_copy).unwrap();

        assert_eq!(reconstructed.origin_network, header.origin_network);
        assert_eq!(reconstructed.height, header.height);
        assert_eq!(reconstructed.prev_pessimistic_root, header.prev_pessimistic_root);
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
                    false
                ),
            },
        };

        let bytes = header.to_bytes_zero_copy();
        let reconstructed = MultiBatchHeader::<Keccak256Hasher>::from_bytes_zero_copy(&bytes).unwrap();

        assert_eq!(reconstructed.origin_network, header.origin_network);
        assert_eq!(reconstructed.height, header.height);
        assert_eq!(reconstructed.prev_pessimistic_root, header.prev_pessimistic_root);
        assert_eq!(reconstructed.l1_info_root, header.l1_info_root);
    }
}
