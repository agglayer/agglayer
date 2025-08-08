#![allow(clippy::too_many_arguments)]

use agglayer_primitives::{Address, Digest, Signature, U256};
use agglayer_tries::proof::{SmtMerkleProof, SmtNonInclusionProof};
use bytemuck::{Pod, Zeroable};
use serde::Serialize;
use unified_bridge::{
    BridgeExit, Claim, ClaimFromMainnet, ClaimFromRollup, GlobalIndex, ImportedBridgeExit,
    L1InfoTreeLeaf, L1InfoTreeLeafInner, LETMerkleProof, LeafType, MerkleProof, NetworkId,
    TokenInfo,
};

use crate::{
    aggchain_proof::AggchainData, local_balance_tree::LocalBalancePath,
    nullifier_tree::NullifierPath,
};

/// Type aliases for semantic clarity and type safety
/// These make the code more readable by giving meaning to raw byte arrays
///
/// A 32-byte hash/digest value (same as Digest, but for zero-copy
/// compatibility) Used for hashes, roots, siblings, metadata hashes
pub type Hash256 = [u8; 32];

/// A 32-byte U256 value (for token amounts, balances)
/// This is the big-endian byte representation of a U256
pub type U256Bytes = [u8; 32];

/// A 20-byte Ethereum address (for token addresses, destination addresses)
/// This is the standard Ethereum address format
pub type AddressBytes = [u8; 20];

/// Type aliases for agglayer_tries proof types to improve readability
pub type BalanceMerkleProof = SmtMerkleProof<192>;
pub type NullifierNonInclusionProof = SmtNonInclusionProof<64>;

/// Constants for aggchain proof types to eliminate magic numbers
pub const AGGCHAIN_PROOF_TYPE_ECDSA: u8 = 0;
pub const AGGCHAIN_PROOF_TYPE_GENERIC: u8 = 1;

/// Constants for claim types to eliminate magic numbers
pub const CLAIM_TYPE_MAINNET: u8 = 0;
pub const CLAIM_TYPE_ROLLUP: u8 = 1;

/// Helper function to convert array of Digests to array of byte arrays
fn digest_array_to_bytes<const N: usize>(digests: &[Digest; N]) -> [[u8; 32]; N] {
    std::array::from_fn(|i| digests[i].0)
}

/// Helper function to convert array of byte arrays to array of Digests
fn bytes_array_to_digests<const N: usize>(bytes: &[[u8; 32]; N]) -> [Digest; N] {
    bytes.map(Digest)
}

// Static assertions for large structs that cannot use derive
// These ensure compile-time verification of struct sizes
const _BALANCE_MERKLE_PROOF_SIZE: () = {
    assert!(std::mem::size_of::<BalanceMerkleProofZeroCopy>() == 6144);
    assert!(std::mem::align_of::<BalanceMerkleProofZeroCopy>() == 1);
};

const _SMT_NON_INCLUSION_PROOF_SIZE: () = {
    assert!(std::mem::size_of::<SmtNonInclusionProofZeroCopy>() == 2052);
    assert!(std::mem::align_of::<SmtNonInclusionProofZeroCopy>() == 1);
};

const _CLAIM_ZERO_COPY_SIZE: () = {
    assert!(std::mem::size_of::<ClaimZeroCopy>() == 3592);
    assert!(std::mem::align_of::<ClaimZeroCopy>() == 1);
};

const _MULTI_BATCH_HEADER_SIZE: () = {
    assert!(std::mem::size_of::<MultiBatchHeaderZeroCopy>() == 248);
    assert!(std::mem::align_of::<MultiBatchHeaderZeroCopy>() == 8);
};

const _BRIDGE_EXIT_ZERO_COPY_SIZE: () = {
    assert!(std::mem::size_of::<BridgeExitZeroCopy>() == 116);
    assert!(std::mem::align_of::<BridgeExitZeroCopy>() == 4);
};

const _AGGCHAIN_DATA_ZERO_COPY_SIZE: () = {
    assert!(std::mem::size_of::<AggchainDataZeroCopy>() == 160);
    assert!(std::mem::align_of::<AggchainDataZeroCopy>() == 1);
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
    pub origin_token_address: AddressBytes,
    /// Destination address (20 bytes)
    pub dest_address: AddressBytes,
    /// Amount (32 bytes) - big-endian U256 representation
    pub amount: U256Bytes,
    /// Metadata hash (32 bytes, 0 if None)
    pub metadata_hash: Hash256,
    /// Leaf type (u8: 0=Transfer, 1=Message)
    pub leaf_type: u8,
    /// Whether metadata is present (u8: 0=No metadata, 1=Has metadata)
    pub has_metadata: u8,
    /// Padding to ensure proper alignment for 4-byte boundaries.
    /// The leaf_type and has_metadata fields are 1 byte each, so we need 2
    /// bytes of padding to align the struct to 4-byte boundaries for
    /// optimal memory access.
    pub _padding: [u8; 2],
}

impl From<&BridgeExit> for BridgeExitZeroCopy {
    fn from(bridge_exit: &BridgeExit) -> Self {
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
            has_metadata: if bridge_exit.metadata.is_some() { 1 } else { 0 },
            _padding: [0; 2],
        }
    }
}

impl From<&BridgeExitZeroCopy> for BridgeExit {
    fn from(zc: &BridgeExitZeroCopy) -> Self {
        BridgeExit {
            leaf_type: zc.leaf_type.try_into().unwrap_or(LeafType::Transfer),
            token_info: TokenInfo {
                origin_network: NetworkId::new(zc.origin_network),
                origin_token_address: Address::new(zc.origin_token_address),
            },
            dest_network: NetworkId::new(zc.dest_network),
            dest_address: Address::from(zc.dest_address),
            amount: U256::from_be_bytes(zc.amount),
            metadata: if zc.has_metadata != 0 {
                Some(Digest(zc.metadata_hash))
            } else {
                None
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
    pub origin_token_address: AddressBytes,
}

impl From<&TokenInfo> for TokenInfoZeroCopy {
    fn from(token_info: &TokenInfo) -> Self {
        Self {
            origin_network: token_info.origin_network.to_u32(),
            origin_token_address: token_info
                .origin_token_address
                .as_slice()
                .try_into()
                .unwrap(),
        }
    }
}

impl From<&TokenInfoZeroCopy> for TokenInfo {
    fn from(zc: &TokenInfoZeroCopy) -> Self {
        TokenInfo {
            origin_network: NetworkId::new(zc.origin_network),
            origin_token_address: Address::from(zc.origin_token_address),
        }
    }
}

/// Zero-copy compatible ImportedBridgeExit for bytemuck operations.
/// This captures the essential fixed-size data from ImportedBridgeExit.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct ImportedBridgeExitZeroCopy {
    /// Global index leaf_index (u32)
    pub global_index_index: u32,
    /// Global index rollup_index (u32) - this is what GlobalIndex::new()
    /// expects as first parameter
    pub global_index_rollup: u32,
    /// Bridge exit data (116 bytes)
    pub bridge_exit: BridgeExitZeroCopy,
    /// Claim data (3352 bytes)
    pub claim_data: ClaimZeroCopy,
    /// End padding to ensure the struct size is a multiple of 8 bytes.
    /// This ensures proper alignment when the struct is used in arrays or
    /// as part of larger structures. Total size: 3480 bytes.
    pub _end_padding: [u8; 4],
}

impl TryFrom<&ImportedBridgeExit> for ImportedBridgeExitZeroCopy {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(imported_bridge_exit: &ImportedBridgeExit) -> Result<Self, Self::Error> {
        let claim_data = ClaimZeroCopy::from(&imported_bridge_exit.claim_data);

        let rollup_index = imported_bridge_exit.global_index.rollup_index().ok_or(
            "GlobalIndex rollup_index is None - this should not happen in rollup contexts",
        )?;

        Ok(Self {
            global_index_index: imported_bridge_exit.global_index.leaf_index(),
            global_index_rollup: rollup_index.to_u32(),
            bridge_exit: (&imported_bridge_exit.bridge_exit).into(),
            claim_data,
            _end_padding: [0; 4],
        })
    }
}

impl TryFrom<&ImportedBridgeExitZeroCopy> for ImportedBridgeExit {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(zc: &ImportedBridgeExitZeroCopy) -> Result<Self, Self::Error> {
        let claim = Claim::try_from(&zc.claim_data)?;
        Ok(ImportedBridgeExit {
            bridge_exit: (&zc.bridge_exit).into(),
            claim_data: claim,
            global_index: GlobalIndex::new(
                NetworkId::new(zc.global_index_rollup),
                zc.global_index_index,
            ),
        })
    }
}

/// Helper struct for Hash256 chunks to work around bytemuck array size
/// limitations Using 32-element chunks as the largest safe size for bytemuck
/// derives
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct Hash256Chunk32(pub [Hash256; 32]);

/// Zero-copy compatible merkle proof types for specific depths.
/// These are newtype wrappers for the specific depths we actually use.
///
/// Balance merkle proof (192 siblings, 6144 bytes) - broken into 6 chunks of 32
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct BalanceMerkleProofZeroCopy {
    /// Siblings 0-31
    pub chunk1: Hash256Chunk32,
    /// Siblings 32-63
    pub chunk2: Hash256Chunk32,
    /// Siblings 64-95
    pub chunk3: Hash256Chunk32,
    /// Siblings 96-127
    pub chunk4: Hash256Chunk32,
    /// Siblings 128-159
    pub chunk5: Hash256Chunk32,
    /// Siblings 160-191
    pub chunk6: Hash256Chunk32,
}

/// LET merkle proof (32 siblings, 1024 bytes)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct LETMerkleProofZeroCopy(pub [Hash256; 32]);

/// Helper function to convert BalanceMerkleProof to BalanceMerkleProofZeroCopy
fn balance_merkle_proof_to_zero_copy(proof: &BalanceMerkleProof) -> BalanceMerkleProofZeroCopy {
    let bytes = digest_array_to_bytes(&proof.siblings);

    // Split the 192-element array into 6 chunks of 32 elements each
    let chunk1 = std::array::from_fn(|i| bytes[i]);
    let chunk2 = std::array::from_fn(|i| bytes[32 + i]);
    let chunk3 = std::array::from_fn(|i| bytes[64 + i]);
    let chunk4 = std::array::from_fn(|i| bytes[96 + i]);
    let chunk5 = std::array::from_fn(|i| bytes[128 + i]);
    let chunk6 = std::array::from_fn(|i| bytes[160 + i]);

    BalanceMerkleProofZeroCopy {
        chunk1: Hash256Chunk32(chunk1),
        chunk2: Hash256Chunk32(chunk2),
        chunk3: Hash256Chunk32(chunk3),
        chunk4: Hash256Chunk32(chunk4),
        chunk5: Hash256Chunk32(chunk5),
        chunk6: Hash256Chunk32(chunk6),
    }
}

/// Helper function to convert BalanceMerkleProofZeroCopy to BalanceMerkleProof
fn balance_merkle_proof_from_zero_copy(zc: &BalanceMerkleProofZeroCopy) -> BalanceMerkleProof {
    // Reconstruct the 192-element array from 6 chunks
    let mut bytes = [[0u8; 32]; 192];
    bytes[0..32].copy_from_slice(&zc.chunk1.0);
    bytes[32..64].copy_from_slice(&zc.chunk2.0);
    bytes[64..96].copy_from_slice(&zc.chunk3.0);
    bytes[96..128].copy_from_slice(&zc.chunk4.0);
    bytes[128..160].copy_from_slice(&zc.chunk5.0);
    bytes[160..192].copy_from_slice(&zc.chunk6.0);

    BalanceMerkleProof {
        siblings: bytes_array_to_digests(&bytes),
    }
}

/// Helper function to convert LETMerkleProof to LETMerkleProofZeroCopy
fn let_merkle_proof_to_zero_copy(proof: &LETMerkleProof) -> LETMerkleProofZeroCopy {
    LETMerkleProofZeroCopy(digest_array_to_bytes(&proof.siblings))
}

/// Helper function to convert LETMerkleProofZeroCopy to LETMerkleProof
fn let_merkle_proof_from_zero_copy(zc: &LETMerkleProofZeroCopy) -> LETMerkleProof {
    LETMerkleProof {
        siblings: bytes_array_to_digests(&zc.0),
    }
}

/// Zero-copy compatible SmtNonInclusionProof for bytemuck operations.
/// This captures the variable-length siblings as fixed-size chunks with length
/// tracking. Now uses safe Pod/Zeroable derives by breaking large array into
/// chunks.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct SmtNonInclusionProofZeroCopy {
    /// Number of actual siblings (u8, max 64)
    pub num_siblings: u8,
    /// Padding to ensure proper alignment for 4-byte boundaries.
    /// The num_siblings field is 1 byte, so we need 3 bytes of padding
    /// to align the siblings chunks to 4-byte boundaries for optimal memory
    /// access.
    pub _padding: [u8; 3],
    /// Siblings chunk 1 (elements 0-31)
    pub siblings_chunk1: Hash256Chunk32,
    /// Siblings chunk 2 (elements 32-63)
    pub siblings_chunk2: Hash256Chunk32,
}

impl From<&NullifierNonInclusionProof> for SmtNonInclusionProofZeroCopy {
    fn from(proof: &NullifierNonInclusionProof) -> Self {
        // Assert that we're not truncating data - this should never happen in practice
        // since SmtNonInclusionProof is designed to work with max 64 siblings
        assert!(
            proof.siblings.len() <= 64,
            "SmtNonInclusionProof cannot have more than 64 siblings, got {}",
            proof.siblings.len()
        );
        let num_siblings = proof.siblings.len() as u8;

        // Fill first chunk (elements 0-31)
        let chunk1 = std::array::from_fn(|i| {
            if i < num_siblings as usize {
                proof.siblings[i].0
            } else {
                [0u8; 32]
            }
        });

        // Fill second chunk (elements 32-63)
        let chunk2 = std::array::from_fn(|i| {
            let index = 32 + i;
            if index < num_siblings as usize {
                proof.siblings[index].0
            } else {
                [0u8; 32]
            }
        });

        Self {
            num_siblings,
            _padding: [0; 3],
            siblings_chunk1: Hash256Chunk32(chunk1),
            siblings_chunk2: Hash256Chunk32(chunk2),
        }
    }
}

impl From<&SmtNonInclusionProofZeroCopy> for NullifierNonInclusionProof {
    fn from(zc: &SmtNonInclusionProofZeroCopy) -> Self {
        // Assert that num_siblings doesn't exceed the maximum capacity
        // This should never happen if the zero-copy struct was created correctly
        assert!(
            zc.num_siblings <= 64,
            "SmtNonInclusionProofZeroCopy cannot have more than 64 siblings, got {}",
            zc.num_siblings
        );

        let num_siblings = zc.num_siblings as usize;
        let mut siblings = Vec::with_capacity(num_siblings);

        // Collect from first chunk (0-31)
        for i in 0..std::cmp::min(32, num_siblings) {
            siblings.push(Digest(zc.siblings_chunk1.0[i]));
        }

        // Collect from second chunk (32-63) if needed
        if num_siblings > 32 {
            for i in 0..(num_siblings - 32) {
                siblings.push(Digest(zc.siblings_chunk2.0[i]));
            }
        }

        NullifierNonInclusionProof { siblings }
    }
}

/// Zero-copy compatible MerkleProof for bytemuck operations.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct MerkleProofZeroCopy {
    /// Proof data (1024 bytes)
    pub proof: LETMerkleProofZeroCopy,
    /// Root (32 bytes)
    pub root: Hash256,
}

impl From<&MerkleProof> for MerkleProofZeroCopy {
    fn from(proof: &MerkleProof) -> Self {
        Self {
            proof: let_merkle_proof_to_zero_copy(&proof.proof),
            root: proof.root.0,
        }
    }
}

impl From<&MerkleProofZeroCopy> for MerkleProof {
    fn from(zc: &MerkleProofZeroCopy) -> Self {
        MerkleProof {
            proof: let_merkle_proof_from_zero_copy(&zc.proof),
            root: Digest(zc.root),
        }
    }
}

/// Zero-copy compatible L1InfoTreeLeafInner for bytemuck operations.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct L1InfoTreeLeafInnerZeroCopy {
    /// Block hash (32 bytes)
    pub block_hash: Hash256,
    /// Timestamp (u64)
    pub timestamp: u64,
    /// Global exit root (32 bytes)
    pub global_exit_root: Hash256,
}

impl From<&L1InfoTreeLeafInner> for L1InfoTreeLeafInnerZeroCopy {
    fn from(inner: &L1InfoTreeLeafInner) -> Self {
        Self {
            block_hash: inner.block_hash.0,
            timestamp: inner.timestamp,
            global_exit_root: inner.global_exit_root.0,
        }
    }
}

impl From<&L1InfoTreeLeafInnerZeroCopy> for L1InfoTreeLeafInner {
    fn from(zc: &L1InfoTreeLeafInnerZeroCopy) -> Self {
        L1InfoTreeLeafInner {
            block_hash: Digest(zc.block_hash),
            timestamp: zc.timestamp,
            global_exit_root: Digest(zc.global_exit_root),
        }
    }
}

/// Zero-copy compatible L1InfoTreeLeaf for bytemuck operations.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct L1InfoTreeLeafZeroCopy {
    /// L1 info tree index (u32)
    pub l1_info_tree_index: u32,
    /// Padding to ensure proper alignment for the inner struct.
    /// The L1InfoTreeLeafInnerZeroCopy contains a u64 timestamp field which
    /// requires 8-byte alignment. This 4-byte padding ensures the inner struct
    /// starts at an 8-byte boundary for optimal memory access.
    pub _padding: [u8; 4],
    /// RER (32 bytes)
    pub rer: Hash256,
    /// MER (32 bytes)
    pub mer: Hash256,
    /// Inner data (72 bytes)
    pub inner: L1InfoTreeLeafInnerZeroCopy,
}

impl From<&L1InfoTreeLeaf> for L1InfoTreeLeafZeroCopy {
    fn from(leaf: &L1InfoTreeLeaf) -> Self {
        Self {
            l1_info_tree_index: leaf.l1_info_tree_index,
            _padding: [0; 4],
            rer: leaf.rer.0,
            mer: leaf.mer.0,
            inner: (&leaf.inner).into(),
        }
    }
}

impl From<&L1InfoTreeLeafZeroCopy> for L1InfoTreeLeaf {
    fn from(zc: &L1InfoTreeLeafZeroCopy) -> Self {
        L1InfoTreeLeaf {
            l1_info_tree_index: zc.l1_info_tree_index,
            rer: Digest(zc.rer),
            mer: Digest(zc.mer),
            inner: (&zc.inner).into(),
        }
    }
}

/// Zero-copy compatible ClaimFromMainnet for bytemuck operations.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct ClaimFromMainnetZeroCopy {
    /// Proof leaf MER (1056 bytes)
    pub proof_leaf_mer: MerkleProofZeroCopy,
    /// Proof GER L1 root (1056 bytes)
    pub proof_ger_l1root: MerkleProofZeroCopy,
    /// L1 leaf (176 bytes)
    pub l1_leaf: L1InfoTreeLeafZeroCopy,
}

impl From<&ClaimFromMainnet> for ClaimFromMainnetZeroCopy {
    fn from(claim: &ClaimFromMainnet) -> Self {
        Self {
            proof_leaf_mer: (&claim.proof_leaf_mer).into(),
            proof_ger_l1root: (&claim.proof_ger_l1root).into(),
            l1_leaf: (&claim.l1_leaf).into(),
        }
    }
}

impl From<&ClaimFromMainnetZeroCopy> for ClaimFromMainnet {
    fn from(zc: &ClaimFromMainnetZeroCopy) -> Self {
        ClaimFromMainnet {
            proof_leaf_mer: (&zc.proof_leaf_mer).into(),
            proof_ger_l1root: (&zc.proof_ger_l1root).into(),
            l1_leaf: (&zc.l1_leaf).into(),
        }
    }
}

/// Zero-copy compatible ClaimFromRollup for bytemuck operations.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct ClaimFromRollupZeroCopy {
    /// Proof from bridge exit leaf to LER (1056 bytes)
    pub proof_leaf_ler: MerkleProofZeroCopy,
    /// Proof from LER to RER (1056 bytes)
    pub proof_ler_rer: MerkleProofZeroCopy,
    /// Proof from GER to L1Root (1056 bytes)
    pub proof_ger_l1root: MerkleProofZeroCopy,
    /// L1 leaf (176 bytes)
    pub l1_leaf: L1InfoTreeLeafZeroCopy,
}

impl From<&ClaimFromRollup> for ClaimFromRollupZeroCopy {
    fn from(claim: &ClaimFromRollup) -> Self {
        Self {
            proof_leaf_ler: (&claim.proof_leaf_ler).into(),
            proof_ler_rer: (&claim.proof_ler_rer).into(),
            proof_ger_l1root: (&claim.proof_ger_l1root).into(),
            l1_leaf: (&claim.l1_leaf).into(),
        }
    }
}

impl From<&ClaimFromRollupZeroCopy> for ClaimFromRollup {
    fn from(zc: &ClaimFromRollupZeroCopy) -> Self {
        ClaimFromRollup {
            proof_leaf_ler: (&zc.proof_leaf_ler).into(),
            proof_ler_rer: (&zc.proof_ler_rer).into(),
            proof_ger_l1root: (&zc.proof_ger_l1root).into(),
            l1_leaf: (&zc.l1_leaf).into(),
        }
    }
}

/// Helper struct for claim data chunks to work around bytemuck array size
/// limitations Using 256-byte chunks as a reasonable size for bytemuck derives
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct ClaimDataChunk256(pub [u8; 256]);

/// Zero-copy compatible Claim for bytemuck operations.
/// This union-like structure can hold either Mainnet or Rollup claim data.
/// Now uses safe Pod/Zeroable derives by breaking large array into chunks.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct ClaimZeroCopy {
    /// Claim type (u8: 0=Mainnet, 1=Rollup)
    pub claim_type: u8,
    /// Padding to ensure proper alignment for 8-byte boundaries.
    /// The claim_type field is 1 byte, so we need 7 bytes of padding
    /// to align the claim_data chunks to 8-byte boundaries for optimal memory
    /// access.
    pub _padding: [u8; 7],
    /// Union of claim data broken into 14 chunks of 256 bytes each
    /// Total: 14 * 256 = 3584 bytes (vs original 3344, providing 240 bytes
    /// buffer) Mainnet: 2288 bytes (uses ~9 chunks), Rollup: 3344 bytes
    /// (uses ~14 chunks)
    pub chunk1: ClaimDataChunk256,
    pub chunk2: ClaimDataChunk256,
    pub chunk3: ClaimDataChunk256,
    pub chunk4: ClaimDataChunk256,
    pub chunk5: ClaimDataChunk256,
    pub chunk6: ClaimDataChunk256,
    pub chunk7: ClaimDataChunk256,
    pub chunk8: ClaimDataChunk256,
    pub chunk9: ClaimDataChunk256,
    pub chunk10: ClaimDataChunk256,
    pub chunk11: ClaimDataChunk256,
    pub chunk12: ClaimDataChunk256,
    pub chunk13: ClaimDataChunk256,
    pub chunk14: ClaimDataChunk256,
}

impl From<&Claim> for ClaimZeroCopy {
    fn from(claim: &Claim) -> Self {
        match claim {
            Claim::Mainnet(mainnet_claim) => {
                let mainnet_zero_copy = ClaimFromMainnetZeroCopy::from(&**mainnet_claim);
                let mainnet_bytes = bytemuck::bytes_of(&mainnet_zero_copy);

                // Distribute bytes across chunks
                let chunks = Self::bytes_to_chunks(mainnet_bytes);

                Self {
                    claim_type: CLAIM_TYPE_MAINNET,
                    _padding: [0; 7],
                    chunk1: chunks[0],
                    chunk2: chunks[1],
                    chunk3: chunks[2],
                    chunk4: chunks[3],
                    chunk5: chunks[4],
                    chunk6: chunks[5],
                    chunk7: chunks[6],
                    chunk8: chunks[7],
                    chunk9: chunks[8],
                    chunk10: chunks[9],
                    chunk11: chunks[10],
                    chunk12: chunks[11],
                    chunk13: chunks[12],
                    chunk14: chunks[13],
                }
            }
            Claim::Rollup(rollup_claim) => {
                let rollup_zero_copy = ClaimFromRollupZeroCopy::from(&**rollup_claim);
                let rollup_bytes = bytemuck::bytes_of(&rollup_zero_copy);

                // Distribute bytes across chunks
                let chunks = Self::bytes_to_chunks(rollup_bytes);

                Self {
                    claim_type: CLAIM_TYPE_ROLLUP,
                    _padding: [0; 7],
                    chunk1: chunks[0],
                    chunk2: chunks[1],
                    chunk3: chunks[2],
                    chunk4: chunks[3],
                    chunk5: chunks[4],
                    chunk6: chunks[5],
                    chunk7: chunks[6],
                    chunk8: chunks[7],
                    chunk9: chunks[8],
                    chunk10: chunks[9],
                    chunk11: chunks[10],
                    chunk12: chunks[11],
                    chunk13: chunks[12],
                    chunk14: chunks[13],
                }
            }
        }
    }
}

impl ClaimZeroCopy {
    /// Helper function to convert bytes to chunks
    fn bytes_to_chunks(bytes: &[u8]) -> [ClaimDataChunk256; 14] {
        let mut chunks = [ClaimDataChunk256([0u8; 256]); 14];

        for (chunk_idx, chunk) in chunks.iter_mut().enumerate() {
            let start = chunk_idx * 256;
            let end = std::cmp::min(start + 256, bytes.len());

            if start < bytes.len() {
                let copy_len = end - start;
                chunk.0[..copy_len].copy_from_slice(&bytes[start..end]);
            }
            // Remaining bytes in chunk are already zeroed
        }

        chunks
    }

    /// Helper function to convert chunks back to bytes
    fn chunks_to_bytes(&self, data_len: usize) -> Vec<u8> {
        let chunks = [
            &self.chunk1,
            &self.chunk2,
            &self.chunk3,
            &self.chunk4,
            &self.chunk5,
            &self.chunk6,
            &self.chunk7,
            &self.chunk8,
            &self.chunk9,
            &self.chunk10,
            &self.chunk11,
            &self.chunk12,
            &self.chunk13,
            &self.chunk14,
        ];

        let mut result = Vec::with_capacity(data_len);
        let mut remaining = data_len;

        for chunk in chunks.iter() {
            if remaining == 0 {
                break;
            }

            let copy_len = std::cmp::min(256, remaining);
            result.extend_from_slice(&chunk.0[..copy_len]);
            remaining -= copy_len;
        }

        result
    }
}

impl TryFrom<&ClaimZeroCopy> for Claim {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(zc: &ClaimZeroCopy) -> Result<Self, Self::Error> {
        match zc.claim_type {
            CLAIM_TYPE_MAINNET => {
                // Mainnet claim
                let mainnet_size = std::mem::size_of::<ClaimFromMainnetZeroCopy>();
                let claim_bytes = zc.chunks_to_bytes(mainnet_size);
                let mainnet_zero_copy = bytemuck::pod_read_unaligned::<ClaimFromMainnetZeroCopy>(
                    &claim_bytes[..mainnet_size],
                );
                Ok(Claim::Mainnet(Box::new((&mainnet_zero_copy).into())))
            }
            CLAIM_TYPE_ROLLUP => {
                // Rollup claim
                let rollup_size = std::mem::size_of::<ClaimFromRollupZeroCopy>();
                let claim_bytes = zc.chunks_to_bytes(rollup_size);
                let rollup_zero_copy = bytemuck::pod_read_unaligned::<ClaimFromRollupZeroCopy>(
                    &claim_bytes[..rollup_size],
                );
                Ok(Claim::Rollup(Box::new((&rollup_zero_copy).into())))
            }
            _ => Err("Invalid claim type".into()),
        }
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
    /// Balance amount (32 bytes) - big-endian U256 representation
    pub balance: U256Bytes,
    /// Padding to ensure proper alignment for 8-byte boundaries.
    /// The token_info is 24 bytes and balance is 32 bytes, totaling 56 bytes.
    /// 8 bytes of padding align the struct to 8-byte boundaries for optimal
    /// memory access. Total size: 64 bytes.
    pub _padding: [u8; 8],
}

/// Helper struct for ECDSA signer data (20 bytes)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct EcdsaSignerData([u8; 20]);

/// Helper struct for ECDSA signature data (64 bytes for r,s + 1 byte for v)
/// Split into two parts to work around bytemuck array size limitations
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct EcdsaSignatureData {
    /// First 64 bytes of signature (r and s values)
    pub rs_data: [u8; 64],
    /// Last byte of signature (v value)
    pub v_data: u8,
}

/// Helper struct for Generic aggchain params (32 bytes)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct GenericParamsData([u8; 32]);

/// Helper struct for Generic vkey data (32 bytes = 8 * u32)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct GenericVkeyData([u8; 32]);

/// Zero-copy compatible AggchainData for bytemuck operations.
/// This captures the fixed-size data from AggchainData variants.
/// Now uses smaller component structs to avoid unsafe implementations.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct AggchainDataZeroCopy {
    /// Aggchain proof type (u8: 0=ECDSA, 1=Generic)
    pub aggchain_proof_type: u8,
    /// Padding to ensure proper alignment for 8-byte boundaries.
    /// The aggchain_proof_type field is 1 byte, so we need 7 bytes of padding
    /// to align the data fields to 8-byte boundaries for optimal memory access.
    pub _padding: [u8; 7],
    /// ECDSA signer data (used when type=0, 20 bytes)
    pub ecdsa_signer: EcdsaSignerData,
    /// ECDSA signature data (used when type=0, 65 bytes)
    pub ecdsa_signature: EcdsaSignatureData,
    /// Generic params data (used when type=1, 32 bytes)
    pub generic_params: GenericParamsData,
    /// Generic vkey data (used when type=1, 32 bytes)
    pub generic_vkey: GenericVkeyData,
    /// End padding to ensure the struct size is a multiple of 8 bytes.
    /// Total size: 1+7+20+65+32+32+3 = 160 bytes (multiple of 8).
    /// This is larger than the previous 96 bytes but eliminates unsafe code.
    pub _end_padding: [u8; 3],
}

impl From<&AggchainData> for AggchainDataZeroCopy {
    fn from(aggchain_data: &AggchainData) -> Self {
        match aggchain_data {
            AggchainData::ECDSA { signer, signature } => {
                // Copy signer address (20 bytes)
                let mut ecdsa_signer_data = [0u8; 20];
                ecdsa_signer_data.copy_from_slice(signer.as_slice());

                // Copy signature bytes (65 bytes)
                let sig_bytes = signature.as_bytes();
                let mut rs_data = [0u8; 64];
                rs_data.copy_from_slice(&sig_bytes[..64]);
                let v_data = sig_bytes[64];

                Self {
                    aggchain_proof_type: AGGCHAIN_PROOF_TYPE_ECDSA,
                    _padding: [0; 7],
                    ecdsa_signer: EcdsaSignerData(ecdsa_signer_data),
                    ecdsa_signature: EcdsaSignatureData { rs_data, v_data },
                    generic_params: GenericParamsData([0; 32]), // Unused for ECDSA
                    generic_vkey: GenericVkeyData([0; 32]),     // Unused for ECDSA
                    _end_padding: [0; 3],
                }
            }
            AggchainData::Generic {
                aggchain_params,
                aggchain_vkey,
            } => {
                // Copy aggchain_params (32 bytes)
                let mut generic_params_data = [0u8; 32];
                generic_params_data.copy_from_slice(aggchain_params.as_slice());

                // Convert vkey from [u32; 8] to bytes using bytemuck
                let vkey_bytes = bytemuck::cast_slice::<u32, u8>(aggchain_vkey);
                let mut generic_vkey_data = [0u8; 32];
                generic_vkey_data.copy_from_slice(vkey_bytes);

                Self {
                    aggchain_proof_type: AGGCHAIN_PROOF_TYPE_GENERIC,
                    _padding: [0; 7],
                    ecdsa_signer: EcdsaSignerData([0; 20]), // Unused for Generic
                    ecdsa_signature: EcdsaSignatureData {
                        rs_data: [0; 64],
                        v_data: 0,
                    }, // Unused for Generic
                    generic_params: GenericParamsData(generic_params_data),
                    generic_vkey: GenericVkeyData(generic_vkey_data),
                    _end_padding: [0; 3],
                }
            }
        }
    }
}

impl TryFrom<&AggchainDataZeroCopy> for AggchainData {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(zero_copy: &AggchainDataZeroCopy) -> Result<Self, Self::Error> {
        match zero_copy.aggchain_proof_type {
            AGGCHAIN_PROOF_TYPE_ECDSA => {
                // ECDSA - reconstruct signer and signature from dedicated fields
                let signer = Address::from(zero_copy.ecdsa_signer.0);

                // Reconstruct the 65-byte signature from rs_data + v_data
                let mut sig_bytes = [0u8; 65];
                sig_bytes[..64].copy_from_slice(&zero_copy.ecdsa_signature.rs_data);
                sig_bytes[64] = zero_copy.ecdsa_signature.v_data;

                let signature = Signature::try_from(&sig_bytes[..])
                    .map_err(|e| format!("Failed to parse signature: {e}"))?;
                Ok(AggchainData::ECDSA { signer, signature })
            }
            AGGCHAIN_PROOF_TYPE_GENERIC => {
                // Generic - reconstruct params and vkey from dedicated fields
                let aggchain_params = Digest::from(zero_copy.generic_params.0);
                // Reconstruct vkey from bytes using bytemuck
                let aggchain_vkey = bytemuck::cast::<[u8; 32], [u32; 8]>(zero_copy.generic_vkey.0);
                Ok(AggchainData::Generic {
                    aggchain_params,
                    aggchain_vkey,
                })
            }
            _ => Err(format!(
                "Invalid aggchain proof type: {}",
                zero_copy.aggchain_proof_type
            )
            .into()),
        }
    }
}

/// Zero-copy representation of MultiBatchHeader for bytemuck operations.
/// This struct has a stable C-compatible memory layout with fixed-size fields
/// and offsets to variable-length data.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
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
    pub prev_pessimistic_root: Hash256,
    /// L1 info root used to import bridge exits (32 bytes)
    pub l1_info_root: Hash256,
    /// Aggchain proof data (zero-copy struct)
    pub aggchain_proof: AggchainDataZeroCopy,
}

/// Represents the chain state transition for the pessimistic proof.
#[derive(Clone, Debug, Serialize)]
pub struct MultiBatchHeader {
    /// Network that emitted this [MultiBatchHeader].
    pub origin_network: NetworkId,
    /// Current certificate height of the L2 chain.
    pub height: u64,
    /// Previous pessimistic root.
    pub prev_pessimistic_root: Digest,
    /// List of bridge exits created in this batch.
    pub bridge_exits: Vec<BridgeExit>,
    /// List of imported bridge exits claimed in this batch.
    pub imported_bridge_exits: Vec<(ImportedBridgeExit, NullifierPath)>,
    /// L1 info root used to import bridge exits.
    pub l1_info_root: Digest,
    /// Token balances of the origin network before processing bridge events,
    /// with Merkle proofs of these balances in the local balance tree.
    pub balances_proofs: Vec<(TokenInfo, (U256, LocalBalancePath))>,
    /// Aggchain proof.
    pub aggchain_proof: AggchainData,
}

impl From<&MultiBatchHeader> for MultiBatchHeaderZeroCopy {
    fn from(self_: &MultiBatchHeader) -> Self {
        MultiBatchHeaderZeroCopy {
            height: self_.height,
            origin_network: self_.origin_network.to_u32(),
            bridge_exits_count: self_.bridge_exits.len() as u32,
            imported_bridge_exits_count: self_.imported_bridge_exits.len() as u32,
            balances_proofs_count: self_.balances_proofs.len() as u32,
            prev_pessimistic_root: self_.prev_pessimistic_root.0,
            l1_info_root: self_.l1_info_root.0,
            aggchain_proof: (&self_.aggchain_proof).into(),
        }
    }
}

impl TryFrom<&MultiBatchHeaderZeroCopy> for MultiBatchHeader {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(zero_copy: &MultiBatchHeaderZeroCopy) -> Result<Self, Self::Error> {
        let origin_network = NetworkId::new(zero_copy.origin_network);
        let prev_pessimistic_root = Digest(zero_copy.prev_pessimistic_root);
        let l1_info_root = Digest(zero_copy.l1_info_root);

        let aggchain_proof = AggchainData::try_from(&zero_copy.aggchain_proof)?;

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
}

/// This struct contains all the byte arrays needed for zero-copy
/// serialization of a MultiBatchHeader.
#[derive(Debug, Clone)]
pub struct ZeroCopyComponents {
    /// Serialized header bytes
    pub header_bytes: Vec<u8>,
    /// Serialized bridge exits bytes
    pub bridge_exits_bytes: Vec<u8>,
    /// Serialized imported bridge exits bytes
    pub imported_bridge_exits_bytes: Vec<u8>,
    /// Serialized nullifier paths bytes
    pub nullifier_paths_bytes: Vec<u8>,
    /// Serialized balance proofs bytes
    pub balances_proofs_bytes: Vec<u8>,
    /// Serialized balance merkle paths bytes
    pub balance_merkle_paths_bytes: Vec<u8>,
}

// Specific implementation for MultiBatchHeader with zero-copy component helpers
impl MultiBatchHeader {
    /// Verify that `bytes.len()` is exactly `size_of::<T>()` for a single
    /// struct.
    fn verify_exact_struct_len<T>(
        bytes: &[u8],
        label: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let expected_len = std::mem::size_of::<T>();
        if bytes.len() != expected_len {
            return Err(format!(
                "{label}: invalid length {}, expected {}",
                bytes.len(),
                expected_len
            )
            .into());
        }
        Ok(())
    }

    /// Reconstruct a MultiBatchHeaderRef (borrowed view) from zero-copy
    /// components. This is the complete reconstruction pattern used in SP1
    /// zkvm environments. Returns a borrowed view to avoid allocations for
    /// variable fields.
    pub fn from_zero_copy_components<'a>(
        header_bytes: &'a [u8],
        bridge_exits_bytes: &'a [u8],
        imported_bridge_exits_bytes: &'a [u8],
        nullifier_paths_bytes: &'a [u8],
        balances_proofs_bytes: &'a [u8],
        balance_merkle_paths_bytes: &'a [u8],
    ) -> Result<MultiBatchHeaderRef<'a>, Box<dyn std::error::Error + Send + Sync>> {
        // 1) Validate header blob size matches exactly one header
        Self::verify_exact_struct_len::<MultiBatchHeaderZeroCopy>(header_bytes, "header_bytes")?;

        // Deserialize header using pod_read_unaligned for robustness against alignment
        // issues
        let header_zero_copy =
            bytemuck::pod_read_unaligned::<MultiBatchHeaderZeroCopy>(header_bytes);

        // 2) Create borrowed slices for zero-copy components using try_cast_slice first
        // Doing this before count validation ensures that alignment/multiple-of-size
        // errors surface as casting errors (as expected by tests), rather than
        // length errors.
        let be_count = header_zero_copy.bridge_exits_count as usize;
        let ibe_count = header_zero_copy.imported_bridge_exits_count as usize;
        let bp_count = header_zero_copy.balances_proofs_count as usize;

        let bridge_exits: &'a [BridgeExitZeroCopy] = if bridge_exits_bytes.is_empty() {
            &[]
        } else {
            bytemuck::try_cast_slice(bridge_exits_bytes)
                .map_err(|e| format!("Failed to cast bridge_exits_bytes: {e}"))?
        };

        let imported_bridge_exits: &'a [ImportedBridgeExitZeroCopy] =
            if imported_bridge_exits_bytes.is_empty() {
                &[]
            } else {
                bytemuck::try_cast_slice(imported_bridge_exits_bytes)
                    .map_err(|e| format!("Failed to cast imported_bridge_exits_bytes: {e}"))?
            };

        let nullifier_paths: &'a [SmtNonInclusionProofZeroCopy] =
            if nullifier_paths_bytes.is_empty() {
                &[]
            } else {
                bytemuck::try_cast_slice(nullifier_paths_bytes)
                    .map_err(|e| format!("Failed to cast nullifier_paths_bytes: {e}"))?
            };

        let balances_proofs: &'a [BalanceProofEntryZeroCopy] = if balances_proofs_bytes.is_empty() {
            &[]
        } else {
            bytemuck::try_cast_slice(balances_proofs_bytes)
                .map_err(|e| format!("Failed to cast balances_proofs_bytes: {e}"))?
        };

        let balance_merkle_paths: &'a [BalanceMerkleProofZeroCopy] =
            if balance_merkle_paths_bytes.is_empty() {
                &[]
            } else {
                bytemuck::try_cast_slice(balance_merkle_paths_bytes)
                    .map_err(|e| format!("Failed to cast balance_merkle_paths_bytes: {e}"))?
            };

        // 3) Cross-validate array counts where required using the casted slice lengths
        if bridge_exits.len() != be_count {
            return Err(format!(
                "bridge_exits_bytes count {} does not match bridge_exits_count {}",
                bridge_exits.len(),
                be_count
            )
            .into());
        }
        if imported_bridge_exits.len() != ibe_count {
            return Err(format!(
                "imported_bridge_exits_bytes count {} does not match imported_bridge_exits_count \
                 {}",
                imported_bridge_exits.len(),
                ibe_count
            )
            .into());
        }
        if balances_proofs.len() != bp_count {
            return Err(format!(
                "balances_proofs_bytes count {} does not match balances_proofs_count {}",
                balances_proofs.len(),
                bp_count
            )
            .into());
        }

        // For nullifier_paths and balance_merkle_paths, derive counts from the slice
        // lengths
        let derived_nullifier_count = nullifier_paths.len();
        let derived_balance_paths_count = balance_merkle_paths.len();

        if derived_nullifier_count != ibe_count {
            return Err(format!(
                "nullifier_paths_bytes count {} does not match imported_bridge_exits_count {}",
                derived_nullifier_count, ibe_count
            )
            .into());
        }
        if derived_balance_paths_count != bp_count {
            return Err(format!(
                "balance_merkle_paths_bytes count {} does not match balances_proofs_count {}",
                derived_balance_paths_count, bp_count
            )
            .into());
        }

        // 5) has_metadata consistency check: if has_metadata == 0 then metadata_hash
        //    must be 0s. Also validate leaf_type discriminants.
        for (idx, be) in bridge_exits.iter().enumerate() {
            if be.has_metadata == 0 && be.metadata_hash != [0u8; 32] {
                return Err(format!(
                    "bridge_exits_bytes[{}]: has_metadata=0 but metadata_hash is non-zero",
                    idx
                )
                .into());
            }
            <LeafType as core::convert::TryFrom<u8>>::try_from(be.leaf_type).map_err(|_| {
                format!(
                    "bridge_exits_bytes[{}]: invalid leaf_type {}",
                    idx, be.leaf_type
                )
            })?;
        }

        // has_metadata consistency for imported bridge exits' inner bridge_exit too
        // and validate leaf_type for nested bridge exits
        for (idx, ibe) in imported_bridge_exits.iter().enumerate() {
            let be = &ibe.bridge_exit;
            if be.has_metadata == 0 && be.metadata_hash != [0u8; 32] {
                return Err(format!(
                    "imported_bridge_exits_bytes[{}].bridge_exit: has_metadata=0 but \
                     metadata_hash is non-zero",
                    idx
                )
                .into());
            }
            <LeafType as core::convert::TryFrom<u8>>::try_from(be.leaf_type).map_err(|_| {
                format!(
                    "imported_bridge_exits_bytes[{}]: invalid leaf_type {}",
                    idx, be.leaf_type
                )
            })?;
        }

        // 6) Validate SmtNonInclusionProofZeroCopy.num_siblings bounds (avoid asserts)
        for (idx, np) in nullifier_paths.iter().enumerate() {
            if np.num_siblings == 0 || np.num_siblings > 64 {
                return Err(format!(
                    "nullifier_paths_bytes[{}]: num_siblings={} out of bounds (expected 1..=64)",
                    idx, np.num_siblings
                )
                .into());
            }
        }

        // Extract aggchain_proof from header
        let aggchain_proof = AggchainData::try_from(&header_zero_copy.aggchain_proof)?;

        // Reconstruct the MultiBatchHeaderRef from zero-copy components
        let origin_network = NetworkId::new(header_zero_copy.origin_network);
        let prev_pessimistic_root = Digest(header_zero_copy.prev_pessimistic_root);
        let l1_info_root = Digest(header_zero_copy.l1_info_root);

        Ok(MultiBatchHeaderRef {
            origin_network,
            height: header_zero_copy.height,
            prev_pessimistic_root,
            bridge_exits,
            imported_bridge_exits,
            nullifier_paths,
            l1_info_root,
            balances_proofs,
            balance_merkle_paths,
            aggchain_proof,
        })
    }

    /// Prepare zero-copy components for serialization.
    /// This returns all the components needed for zero-copy serialization.
    pub fn to_zero_copy_components(
        &self,
    ) -> Result<ZeroCopyComponents, Box<dyn std::error::Error + Send + Sync>> {
        // Convert header to zero-copy
        let header_zero_copy: MultiBatchHeaderZeroCopy = self.into();
        let header_bytes = bytemuck::bytes_of(&header_zero_copy).to_vec();

        // Convert bridge_exits to zero-copy
        let bridge_exits_zero_copy: Vec<BridgeExitZeroCopy> =
            self.bridge_exits.iter().map(|be| be.into()).collect();
        let bridge_exits_bytes = bytemuck::cast_slice(&bridge_exits_zero_copy).to_vec();

        // Convert imported_bridge_exits to zero-copy
        let imported_bridge_exits_zero_copy: Vec<ImportedBridgeExitZeroCopy> = self
            .imported_bridge_exits
            .iter()
            .map(|(ibe, _)| ImportedBridgeExitZeroCopy::try_from(ibe))
            .collect::<Result<Vec<_>, _>>()?;
        let imported_bridge_exits_bytes =
            bytemuck::cast_slice(&imported_bridge_exits_zero_copy).to_vec();

        // Extract nullifier paths
        let nullifier_paths_zero_copy: Vec<SmtNonInclusionProofZeroCopy> = self
            .imported_bridge_exits
            .iter()
            .map(|(_, path)| path.into())
            .collect();
        let nullifier_paths_bytes = bytemuck::cast_slice(&nullifier_paths_zero_copy).to_vec();

        // Convert balances_proofs to zero-copy
        let balances_proofs_zero_copy: Vec<BalanceProofEntryZeroCopy> = self
            .balances_proofs
            .iter()
            .map(|(token_info, (balance, _))| BalanceProofEntryZeroCopy {
                token_info: token_info.into(),
                balance: balance.to_be_bytes(),
                _padding: [0; 8],
            })
            .collect();
        let balances_proofs_bytes = bytemuck::cast_slice(&balances_proofs_zero_copy).to_vec();

        // Extract balance Merkle paths
        let balance_merkle_paths_zero_copy: Vec<BalanceMerkleProofZeroCopy> = self
            .balances_proofs
            .iter()
            .map(|(_, (_, path))| balance_merkle_proof_to_zero_copy(path))
            .collect();
        let balance_merkle_paths_bytes =
            bytemuck::cast_slice(&balance_merkle_paths_zero_copy).to_vec();

        Ok(ZeroCopyComponents {
            header_bytes,
            bridge_exits_bytes,
            imported_bridge_exits_bytes,
            nullifier_paths_bytes,
            balances_proofs_bytes,
            balance_merkle_paths_bytes,
        })
    }
}

/// Zero-copy borrowed view of MultiBatchHeader that avoids allocations for
/// variable fields. This struct holds borrowed slices for large variable-length
/// data while keeping small fields owned.
#[derive(Debug, Clone)]
pub struct MultiBatchHeaderRef<'a> {
    /// Network that emitted this [MultiBatchHeaderRef].
    pub origin_network: NetworkId,
    /// Current certificate height of the L2 chain.
    pub height: u64,
    /// Previous pessimistic root.
    pub prev_pessimistic_root: Digest,
    /// List of bridge exits created in this batch (borrowed).
    pub bridge_exits: &'a [BridgeExitZeroCopy],
    /// List of imported bridge exits claimed in this batch (borrowed).
    pub imported_bridge_exits: &'a [ImportedBridgeExitZeroCopy],
    /// Nullifier paths for imported bridge exits (borrowed).
    pub nullifier_paths: &'a [SmtNonInclusionProofZeroCopy],
    /// L1 info root used to import bridge exits.
    pub l1_info_root: Digest,
    /// Token balances of the origin network before processing bridge events
    /// (borrowed).
    pub balances_proofs: &'a [BalanceProofEntryZeroCopy],
    /// Balance Merkle paths (borrowed).
    pub balance_merkle_paths: &'a [BalanceMerkleProofZeroCopy],
    /// Aggchain proof.
    pub aggchain_proof: AggchainData,
}

// Implementation for MultiBatchHeaderRef
impl MultiBatchHeaderRef<'_> {
    /// Convert to owned MultiBatchHeader by cloning all borrowed data.
    pub fn to_owned(&self) -> Result<MultiBatchHeader, Box<dyn std::error::Error + Send + Sync>> {
        // Convert bridge_exits
        let bridge_exits: Vec<BridgeExit> = self.bridge_exits.iter().map(|be| be.into()).collect();

        // Convert imported_bridge_exits and nullifier_paths
        let imported_bridge_exits: Vec<(ImportedBridgeExit, NullifierPath)> = self
            .imported_bridge_exits
            .iter()
            .zip(self.nullifier_paths.iter())
            .map(|(ibe, path)| {
                let imported_bridge_exit = ImportedBridgeExit::try_from(ibe)?;
                let nullifier_path = path.into();
                Ok((imported_bridge_exit, nullifier_path))
            })
            .collect::<Result<_, Box<dyn std::error::Error + Send + Sync>>>()?;

        // Convert balances_proofs and balance_merkle_paths
        let balances_proofs: Vec<(TokenInfo, (U256, LocalBalancePath))> = self
            .balances_proofs
            .iter()
            .zip(self.balance_merkle_paths.iter())
            .map(|(bp, path)| {
                let token_info = (&bp.token_info).into();
                let balance = U256::from_be_bytes(bp.balance);
                let merkle_path = balance_merkle_proof_from_zero_copy(path);
                (token_info, (balance, merkle_path))
            })
            .collect();

        Ok(MultiBatchHeader {
            origin_network: self.origin_network,
            height: self.height,
            prev_pessimistic_root: self.prev_pessimistic_root,
            bridge_exits,
            imported_bridge_exits,
            l1_info_root: self.l1_info_root,
            balances_proofs,
            aggchain_proof: self.aggchain_proof.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deep comparison function to check for lossy conversions
    /// This function compares all fields including nested structures
    /// Uses Eq where available, manual comparison where needed
    fn deep_equals(original: &MultiBatchHeader, reconstructed: &MultiBatchHeader) -> bool {
        // Compare basic fields (all have Eq)
        if original.origin_network != reconstructed.origin_network
            || original.height != reconstructed.height
            || original.prev_pessimistic_root != reconstructed.prev_pessimistic_root
            || original.bridge_exits != reconstructed.bridge_exits
            || original.l1_info_root != reconstructed.l1_info_root
            || original.aggchain_proof != reconstructed.aggchain_proof
        {
            return false;
        }

        // Compare imported_bridge_exits (most fields have Eq, only nullifier paths need
        // manual comparison)
        if original.imported_bridge_exits.len() != reconstructed.imported_bridge_exits.len() {
            return false;
        }
        for (orig, rec) in original
            .imported_bridge_exits
            .iter()
            .zip(reconstructed.imported_bridge_exits.iter())
        {
            // Compare ImportedBridgeExit (has Eq)
            if orig.0 != rec.0 {
                return false;
            }
            // Compare nullifier paths manually (SmtNonInclusionProof doesn't have Eq)
            if orig.1.siblings != rec.1.siblings {
                return false;
            }
        }

        // Compare balances_proofs (most fields have Eq, only merkle paths need manual
        // comparison)
        if original.balances_proofs.len() != reconstructed.balances_proofs.len() {
            return false;
        }
        for (orig, rec) in original
            .balances_proofs
            .iter()
            .zip(reconstructed.balances_proofs.iter())
        {
            // Compare TokenInfo and U256 (both have Eq)
            if orig.0 != rec.0 || orig.1 .0 != rec.1 .0 {
                return false;
            }
            // Compare merkle paths manually (SmtMerkleProof doesn't have Eq)
            if orig.1 .1.siblings != rec.1 .1.siblings {
                return false;
            }
        }

        true
    }

    /// Test helper to create a sample BridgeExit
    fn create_sample_bridge_exit() -> BridgeExit {
        BridgeExit {
            leaf_type: LeafType::Message,
            token_info: TokenInfo {
                origin_network: NetworkId::new(1),
                origin_token_address: Address::new([1u8; 20]),
            },
            dest_network: NetworkId::new(2),
            dest_address: Address::new([2u8; 20]),
            amount: U256::from(1000u64),
            metadata: Some(Digest([3u8; 32])),
        }
    }

    /// Test helper to create a sample ImportedBridgeExit
    fn create_sample_imported_bridge_exit() -> ImportedBridgeExit {
        ImportedBridgeExit {
            bridge_exit: create_sample_bridge_exit(),
            claim_data: Claim::Mainnet(Box::new(ClaimFromMainnet {
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
            })),
            global_index: GlobalIndex::new(NetworkId::new(3), 123),
        }
    }

    /// Test helper to create a sample ImportedBridgeExit with Rollup claim
    fn create_sample_imported_bridge_exit_rollup() -> ImportedBridgeExit {
        ImportedBridgeExit {
            bridge_exit: create_sample_bridge_exit(),
            claim_data: Claim::Rollup(Box::new(ClaimFromRollup {
                proof_leaf_ler: MerkleProof {
                    proof: LETMerkleProof {
                        siblings: [Digest([12u8; 32]); 32],
                    },
                    root: Digest([13u8; 32]),
                },
                proof_ler_rer: MerkleProof {
                    proof: LETMerkleProof {
                        siblings: [Digest([14u8; 32]); 32],
                    },
                    root: Digest([15u8; 32]),
                },
                proof_ger_l1root: MerkleProof {
                    proof: LETMerkleProof {
                        siblings: [Digest([16u8; 32]); 32],
                    },
                    root: Digest([17u8; 32]),
                },
                l1_leaf: L1InfoTreeLeaf {
                    l1_info_tree_index: 43,
                    rer: Digest([18u8; 32]),
                    mer: Digest([19u8; 32]),
                    inner: L1InfoTreeLeafInner {
                        block_hash: Digest([20u8; 32]),
                        timestamp: 1234567891,
                        global_exit_root: Digest([21u8; 32]),
                    },
                },
            })),
            global_index: GlobalIndex::new(NetworkId::new(4), 124),
        }
    }

    /// Test helper to create a sample TokenInfo
    fn create_sample_token_info() -> TokenInfo {
        TokenInfo {
            origin_network: NetworkId::new(4),
            origin_token_address: Address::new([12u8; 20]),
        }
    }

    /// Test helper to create a sample BalanceMerkleProof
    fn create_sample_balance_merkle_proof() -> BalanceMerkleProof {
        BalanceMerkleProof {
            siblings: [Digest([13u8; 32]); 192],
        }
    }

    /// Test helper to create a sample NullifierNonInclusionProof
    fn create_sample_nullifier_non_inclusion_proof() -> NullifierNonInclusionProof {
        NullifierNonInclusionProof {
            siblings: vec![Digest([14u8; 32]); 64],
        }
    }

    /// Test helper to create a sample NullifierNonInclusionProof with fewer
    /// siblings
    fn create_sample_nullifier_non_inclusion_proof_partial() -> NullifierNonInclusionProof {
        NullifierNonInclusionProof {
            siblings: vec![Digest([15u8; 32]); 32], // Only 32 siblings instead of 64
        }
    }

    /// Test helper to create a sample MultiBatchHeader
    fn create_sample_multi_batch_header() -> MultiBatchHeader {
        MultiBatchHeader {
            origin_network: NetworkId::new(5),
            height: 1000,
            prev_pessimistic_root: Digest([15u8; 32]),
            bridge_exits: vec![create_sample_bridge_exit()],
            imported_bridge_exits: vec![(
                create_sample_imported_bridge_exit(),
                create_sample_nullifier_non_inclusion_proof(),
            )],
            l1_info_root: Digest([16u8; 32]),
            balances_proofs: vec![(
                create_sample_token_info(),
                (U256::from(5000u64), create_sample_balance_merkle_proof()),
            )],
            aggchain_proof: AggchainData::ECDSA {
                signer: Address::new([17u8; 20]),
                signature: Signature::new(U256::from(18u64), U256::from(19u64), true),
            },
        }
    }

    /// Test helper to create a sample MultiBatchHeader with Generic aggchain
    /// proof
    fn create_sample_multi_batch_header_generic() -> MultiBatchHeader {
        MultiBatchHeader {
            origin_network: NetworkId::new(6),
            height: 2000,
            prev_pessimistic_root: Digest([20u8; 32]),
            bridge_exits: vec![create_sample_bridge_exit()],
            imported_bridge_exits: vec![(
                create_sample_imported_bridge_exit(),
                create_sample_nullifier_non_inclusion_proof(),
            )],
            l1_info_root: Digest([21u8; 32]),
            balances_proofs: vec![(
                create_sample_token_info(),
                (U256::from(7000u64), create_sample_balance_merkle_proof()),
            )],
            aggchain_proof: AggchainData::Generic {
                aggchain_params: Digest([22u8; 32]),
                aggchain_vkey: [23u32, 24u32, 25u32, 26u32, 27u32, 28u32, 29u32, 30u32],
            },
        }
    }

    /// Test helper to create a sample MultiBatchHeader with Rollup claims
    fn create_sample_multi_batch_header_rollup() -> MultiBatchHeader {
        MultiBatchHeader {
            origin_network: NetworkId::new(7),
            height: 3000,
            prev_pessimistic_root: Digest([30u8; 32]),
            bridge_exits: vec![create_sample_bridge_exit()],
            imported_bridge_exits: vec![(
                create_sample_imported_bridge_exit_rollup(),
                create_sample_nullifier_non_inclusion_proof(),
            )],
            l1_info_root: Digest([31u8; 32]),
            balances_proofs: vec![(
                create_sample_token_info(),
                (U256::from(8000u64), create_sample_balance_merkle_proof()),
            )],
            aggchain_proof: AggchainData::ECDSA {
                signer: Address::new([32u8; 20]),
                signature: Signature::new(U256::from(33u64), U256::from(34u64), false),
            },
        }
    }

    /// Test helper to create a sample MultiBatchHeader with mixed claims
    fn create_sample_multi_batch_header_mixed() -> MultiBatchHeader {
        MultiBatchHeader {
            origin_network: NetworkId::new(8),
            height: 4000,
            prev_pessimistic_root: Digest([40u8; 32]),
            bridge_exits: vec![create_sample_bridge_exit()],
            imported_bridge_exits: vec![
                (
                    create_sample_imported_bridge_exit(),
                    create_sample_nullifier_non_inclusion_proof(),
                ),
                (
                    create_sample_imported_bridge_exit_rollup(),
                    create_sample_nullifier_non_inclusion_proof(),
                ),
            ],
            l1_info_root: Digest([41u8; 32]),
            balances_proofs: vec![(
                create_sample_token_info(),
                (U256::from(9000u64), create_sample_balance_merkle_proof()),
            )],
            aggchain_proof: AggchainData::Generic {
                aggchain_params: Digest([42u8; 32]),
                aggchain_vkey: [43u32, 44u32, 45u32, 46u32, 47u32, 48u32, 49u32, 50u32],
            },
        }
    }

    #[test]
    fn test_bridge_exit_zero_copy_edge_cases() {
        // Test with maximum values
        let mut bridge_exit = create_sample_bridge_exit();
        bridge_exit.amount = U256::MAX;
        bridge_exit.metadata = Some(Digest([0xFFu8; 32]));

        let zero_copy: BridgeExitZeroCopy = (&bridge_exit).into();
        let reconstructed: BridgeExit = (&zero_copy).into();

        assert_eq!(bridge_exit.amount, reconstructed.amount);
        assert_eq!(bridge_exit.metadata, reconstructed.metadata);
        assert_eq!(zero_copy.has_metadata, 1);

        // Test with zero values
        let mut bridge_exit = create_sample_bridge_exit();
        bridge_exit.amount = U256::ZERO;
        bridge_exit.metadata = None;

        let zero_copy: BridgeExitZeroCopy = (&bridge_exit).into();
        let reconstructed: BridgeExit = (&zero_copy).into();

        assert_eq!(bridge_exit.amount, reconstructed.amount);
        assert_eq!(bridge_exit.metadata, reconstructed.metadata);
        assert_eq!(zero_copy.has_metadata, 0);

        // Test with zero metadata hash but has_metadata = 1 (should preserve the hash)
        let mut bridge_exit = create_sample_bridge_exit();
        bridge_exit.metadata = Some(Digest([0u8; 32])); // Zero hash

        let zero_copy: BridgeExitZeroCopy = (&bridge_exit).into();
        let reconstructed: BridgeExit = (&zero_copy).into();

        assert_eq!(bridge_exit.metadata, reconstructed.metadata);
        assert_eq!(zero_copy.has_metadata, 1);
        assert_eq!(zero_copy.metadata_hash, [0u8; 32]);
    }

    #[test]
    fn test_invalid_aggchain_proof_type() {
        let original = create_sample_multi_batch_header();
        let mut zero_copy: MultiBatchHeaderZeroCopy = (&original).into();
        zero_copy.aggchain_proof.aggchain_proof_type = 255; // Invalid type

        let result: Result<MultiBatchHeader, _> = MultiBatchHeader::try_from(&zero_copy);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid aggchain proof type"));
    }

    /// Test demonstrating full recovery and borrowed view functionality for all
    /// claim types and aggchain proof types.
    #[test]
    fn test_zero_copy_recovery_and_borrowed_view() {
        let test_cases = vec![
            ("ECDSA + Mainnet", create_sample_multi_batch_header()),
            (
                "Generic + Mainnet",
                create_sample_multi_batch_header_generic(),
            ),
            ("ECDSA + Rollup", create_sample_multi_batch_header_rollup()),
            ("Generic + Mixed", create_sample_multi_batch_header_mixed()),
        ];

        for (_case_name, original) in test_cases {
            // Test full recovery
            test_zero_copy_recovery(&original);

            // Test borrowed view recovery
            test_borrowed_view_recovery(&original);
        }
    }

    /// Test that alignment errors are handled correctly when using
    /// try_cast_slice.
    #[test]
    fn test_alignment_error_handling() {
        let original = create_sample_multi_batch_header();
        let components = original
            .to_zero_copy_components()
            .expect("Failed to convert to zero-copy components");

        // Test with misaligned data by adding a single byte
        let mut misaligned_bridge_exits = vec![0u8];
        misaligned_bridge_exits.extend_from_slice(&components.bridge_exits_bytes);

        let result = MultiBatchHeader::from_zero_copy_components(
            &components.header_bytes,
            &misaligned_bridge_exits,
            &components.imported_bridge_exits_bytes,
            &components.nullifier_paths_bytes,
            &components.balances_proofs_bytes,
            &components.balance_merkle_paths_bytes,
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to cast bridge_exits_bytes"));
    }

    /// Helper function to test zero-copy recovery for a given MultiBatchHeader
    fn test_zero_copy_recovery(original: &MultiBatchHeader) {
        // Use the new helper function to get all zero-copy components
        let components = original
            .to_zero_copy_components()
            .expect("Failed to convert to zero-copy components");

        // Reconstruct the MultiBatchHeaderRef (borrowed view) from zero-copy components
        let borrowed_view = MultiBatchHeader::from_zero_copy_components(
            &components.header_bytes,
            &components.bridge_exits_bytes,
            &components.imported_bridge_exits_bytes,
            &components.nullifier_paths_bytes,
            &components.balances_proofs_bytes,
            &components.balance_merkle_paths_bytes,
        )
        .expect("Failed to reconstruct MultiBatchHeaderRef");

        // Convert to owned for deep comparison
        let reconstructed = borrowed_view
            .to_owned()
            .expect("Failed to convert to owned");

        // Verify full recovery using comprehensive deep comparison
        // This will catch any lossy conversions in any field
        assert!(
            deep_equals(original, &reconstructed),
            "Deep comparison failed - there are lossy conversions!"
        );
    }

    /// Helper function to test borrowed view recovery for a given
    /// MultiBatchHeader
    fn test_borrowed_view_recovery(original: &MultiBatchHeader) {
        // Use the new helper function to get all zero-copy components
        let components = original
            .to_zero_copy_components()
            .expect("Failed to convert to zero-copy components");

        // Reconstruct the MultiBatchHeaderRef (borrowed view) from zero-copy components
        let borrowed_view = MultiBatchHeader::from_zero_copy_components(
            &components.header_bytes,
            &components.bridge_exits_bytes,
            &components.imported_bridge_exits_bytes,
            &components.nullifier_paths_bytes,
            &components.balances_proofs_bytes,
            &components.balance_merkle_paths_bytes,
        )
        .expect("Failed to reconstruct MultiBatchHeaderRef");

        // Convert to owned and verify full recovery
        let reconstructed = borrowed_view
            .to_owned()
            .expect("Failed to convert to owned");

        // Verify full recovery using comprehensive deep comparison
        // This will catch any lossy conversions in any field
        assert!(
            deep_equals(original, &reconstructed),
            "Deep comparison failed - there are lossy conversions in borrowed view!"
        );

        // Verify that the borrowed view has the correct counts
        assert_eq!(
            borrowed_view.bridge_exits.len(),
            original.bridge_exits.len()
        );
        assert_eq!(
            borrowed_view.imported_bridge_exits.len(),
            original.imported_bridge_exits.len()
        );
        assert_eq!(
            borrowed_view.nullifier_paths.len(),
            original.imported_bridge_exits.len()
        );
        assert_eq!(
            borrowed_view.balances_proofs.len(),
            original.balances_proofs.len()
        );
        assert_eq!(
            borrowed_view.balance_merkle_paths.len(),
            original.balances_proofs.len()
        );

        // Verify that the borrowed view has the correct basic fields
        assert_eq!(borrowed_view.origin_network, original.origin_network);
        assert_eq!(borrowed_view.height, original.height);
        assert_eq!(
            borrowed_view.prev_pessimistic_root,
            original.prev_pessimistic_root
        );
        assert_eq!(borrowed_view.l1_info_root, original.l1_info_root);
    }

    /// Test that ClaimZeroCopy correctly handles both Mainnet and Rollup
    /// claims.
    #[test]
    fn test_claim_zero_copy_conversion() {
        // Test Mainnet claim
        let mainnet_imported_exit = create_sample_imported_bridge_exit();
        let mainnet_claim_zero_copy = ClaimZeroCopy::from(&mainnet_imported_exit.claim_data);
        let reconstructed_mainnet_claim = Claim::try_from(&mainnet_claim_zero_copy).unwrap();

        match (
            &mainnet_imported_exit.claim_data,
            &reconstructed_mainnet_claim,
        ) {
            (Claim::Mainnet(orig), Claim::Mainnet(rec)) => {
                assert_eq!(orig.proof_leaf_mer.root, rec.proof_leaf_mer.root);
                assert_eq!(orig.proof_ger_l1root.root, rec.proof_ger_l1root.root);
                assert_eq!(
                    orig.l1_leaf.l1_info_tree_index,
                    rec.l1_leaf.l1_info_tree_index
                );
            }
            _ => panic!("Expected Mainnet claims"),
        }

        // Test Rollup claim
        let rollup_imported_exit = create_sample_imported_bridge_exit_rollup();
        let rollup_claim_zero_copy = ClaimZeroCopy::from(&rollup_imported_exit.claim_data);
        let reconstructed_rollup_claim = Claim::try_from(&rollup_claim_zero_copy).unwrap();

        match (
            &rollup_imported_exit.claim_data,
            &reconstructed_rollup_claim,
        ) {
            (Claim::Rollup(orig), Claim::Rollup(rec)) => {
                assert_eq!(orig.proof_leaf_ler.root, rec.proof_leaf_ler.root);
                assert_eq!(orig.proof_ler_rer.root, rec.proof_ler_rer.root);
                assert_eq!(orig.proof_ger_l1root.root, rec.proof_ger_l1root.root);
                assert_eq!(
                    orig.l1_leaf.l1_info_tree_index,
                    rec.l1_leaf.l1_info_tree_index
                );
            }
            _ => panic!("Expected Rollup claims"),
        }
    }

    /// Test that invalid claim types are handled correctly.
    #[test]
    fn test_invalid_claim_type() {
        let mut claim_zero_copy =
            ClaimZeroCopy::from(&create_sample_imported_bridge_exit().claim_data);
        claim_zero_copy.claim_type = 255; // Invalid type

        let result = Claim::try_from(&claim_zero_copy);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid claim type"));
    }

    /// Test that SmtNonInclusionProofZeroCopy correctly handles variable-length
    /// siblings.
    #[test]
    fn test_smt_non_inclusion_proof_variable_length() {
        // Test with full-length proof (64 siblings)
        let full_proof = create_sample_nullifier_non_inclusion_proof();
        let full_zero_copy = SmtNonInclusionProofZeroCopy::from(&full_proof);
        let reconstructed_full: NullifierNonInclusionProof = (&full_zero_copy).into();

        assert_eq!(full_proof.siblings.len(), reconstructed_full.siblings.len());
        assert_eq!(full_proof.siblings, reconstructed_full.siblings);
        assert_eq!(full_zero_copy.num_siblings, 64);

        // Test with partial-length proof (32 siblings)
        let partial_proof = create_sample_nullifier_non_inclusion_proof_partial();
        let partial_zero_copy = SmtNonInclusionProofZeroCopy::from(&partial_proof);
        let reconstructed_partial: NullifierNonInclusionProof = (&partial_zero_copy).into();

        assert_eq!(
            partial_proof.siblings.len(),
            reconstructed_partial.siblings.len()
        );
        assert_eq!(partial_proof.siblings, reconstructed_partial.siblings);
        assert_eq!(partial_zero_copy.num_siblings, 32);

        // Verify that the zero-copy struct has the correct size
        assert_eq!(std::mem::size_of::<SmtNonInclusionProofZeroCopy>(), 2052);
    }

    /// Test that signature reconstruction correctly handles Ethereum v values
    /// (27/28).
    #[test]
    fn test_signature_reconstruction_ethereum_v() {
        // Create a sample MultiBatchHeader with ECDSA signature
        let original = create_sample_multi_batch_header();

        // Convert to zero-copy and back
        let zero_copy: MultiBatchHeaderZeroCopy = (&original).into();
        let reconstructed: MultiBatchHeader = MultiBatchHeader::try_from(&zero_copy).unwrap();

        // Verify that the signature was reconstructed correctly
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
                assert_eq!(orig_signer, rec_signer);
                assert_eq!(orig_sig.r(), rec_sig.r());
                assert_eq!(orig_sig.s(), rec_sig.s());
                assert_eq!(orig_sig.v(), rec_sig.v());
            }
            _ => panic!("Expected ECDSA signatures"),
        }
    }

    /// Test that struct sizes and alignments are as expected.
    /// This test verifies both compile-time and runtime struct layouts.
    #[test]
    fn test_struct_sizes_and_alignments() {
        // Compile-time size and alignment assertions
        assert_eq!(std::mem::size_of::<BridgeExitZeroCopy>(), 116);
        assert_eq!(std::mem::size_of::<BalanceMerkleProofZeroCopy>(), 6144);
        assert_eq!(std::mem::size_of::<SmtNonInclusionProofZeroCopy>(), 2052);
        assert_eq!(std::mem::size_of::<ClaimZeroCopy>(), 3592);
        assert_eq!(std::mem::size_of::<MultiBatchHeaderZeroCopy>(), 248);
        assert_eq!(std::mem::size_of::<BalanceProofEntryZeroCopy>(), 64);
        assert_eq!(std::mem::size_of::<AggchainDataZeroCopy>(), 160);

        // Compile-time alignment assertions
        assert_eq!(std::mem::align_of::<BridgeExitZeroCopy>(), 4);
        assert_eq!(std::mem::align_of::<BalanceMerkleProofZeroCopy>(), 1);
        assert_eq!(std::mem::align_of::<SmtNonInclusionProofZeroCopy>(), 1);
        assert_eq!(std::mem::align_of::<ClaimZeroCopy>(), 1);
        assert_eq!(std::mem::align_of::<MultiBatchHeaderZeroCopy>(), 8);
        assert_eq!(std::mem::align_of::<BalanceProofEntryZeroCopy>(), 4);
        assert_eq!(std::mem::align_of::<AggchainDataZeroCopy>(), 1);

        // Runtime size and alignment verification for large structs
        let balance_proof = BalanceMerkleProofZeroCopy {
            chunk1: Hash256Chunk32([[0u8; 32]; 32]),
            chunk2: Hash256Chunk32([[0u8; 32]; 32]),
            chunk3: Hash256Chunk32([[0u8; 32]; 32]),
            chunk4: Hash256Chunk32([[0u8; 32]; 32]),
            chunk5: Hash256Chunk32([[0u8; 32]; 32]),
            chunk6: Hash256Chunk32([[0u8; 32]; 32]),
        };
        assert_eq!(std::mem::size_of_val(&balance_proof), 6144);
        assert_eq!(std::mem::align_of_val(&balance_proof), 1);

        let smt_non_inclusion_proof = SmtNonInclusionProofZeroCopy {
            num_siblings: 64,
            _padding: [0; 3],
            siblings_chunk1: Hash256Chunk32([[0u8; 32]; 32]),
            siblings_chunk2: Hash256Chunk32([[0u8; 32]; 32]),
        };
        assert_eq!(std::mem::size_of_val(&smt_non_inclusion_proof), 2052);
        assert_eq!(std::mem::align_of_val(&smt_non_inclusion_proof), 1);

        let claim = ClaimZeroCopy {
            claim_type: CLAIM_TYPE_MAINNET,
            _padding: [0; 7],
            chunk1: ClaimDataChunk256([0u8; 256]),
            chunk2: ClaimDataChunk256([0u8; 256]),
            chunk3: ClaimDataChunk256([0u8; 256]),
            chunk4: ClaimDataChunk256([0u8; 256]),
            chunk5: ClaimDataChunk256([0u8; 256]),
            chunk6: ClaimDataChunk256([0u8; 256]),
            chunk7: ClaimDataChunk256([0u8; 256]),
            chunk8: ClaimDataChunk256([0u8; 256]),
            chunk9: ClaimDataChunk256([0u8; 256]),
            chunk10: ClaimDataChunk256([0u8; 256]),
            chunk11: ClaimDataChunk256([0u8; 256]),
            chunk12: ClaimDataChunk256([0u8; 256]),
            chunk13: ClaimDataChunk256([0u8; 256]),
            chunk14: ClaimDataChunk256([0u8; 256]),
        };
        assert_eq!(std::mem::size_of_val(&claim), 3592);
        assert_eq!(std::mem::align_of_val(&claim), 1);

        let header = MultiBatchHeaderZeroCopy {
            height: 0,
            origin_network: 0,
            bridge_exits_count: 0,
            imported_bridge_exits_count: 0,
            balances_proofs_count: 0,
            prev_pessimistic_root: [0u8; 32],
            l1_info_root: [0u8; 32],
            aggchain_proof: AggchainDataZeroCopy {
                aggchain_proof_type: 0,
                _padding: [0; 7],
                ecdsa_signer: EcdsaSignerData([0; 20]),
                ecdsa_signature: EcdsaSignatureData {
                    rs_data: [0; 64],
                    v_data: 0,
                },
                generic_params: GenericParamsData([0; 32]),
                generic_vkey: GenericVkeyData([0; 32]),
                _end_padding: [0; 3],
            },
        };
        assert_eq!(std::mem::size_of_val(&header), 248);
        assert_eq!(std::mem::align_of_val(&header), 8);
    }

    /// Test edge cases and serialization for AggchainDataZeroCopy.
    #[test]
    fn test_aggchain_data_zero_copy_edge_cases() {
        // Test with maximum values for ECDSA
        let max_ecdsa = AggchainData::ECDSA {
            signer: Address::new([0xFFu8; 20]),
            signature: Signature::new(U256::MAX, U256::MAX, true),
        };

        let zero_copy = AggchainDataZeroCopy::from(&max_ecdsa);
        let reconstructed = AggchainData::try_from(&zero_copy).unwrap();

        match (&max_ecdsa, &reconstructed) {
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
                assert_eq!(orig_signer, rec_signer);
                assert_eq!(orig_sig.r(), rec_sig.r());
                assert_eq!(orig_sig.s(), rec_sig.s());
                assert_eq!(orig_sig.v(), rec_sig.v());
            }
            _ => panic!("Expected ECDSA variants"),
        }

        // Test with maximum values for Generic
        let max_generic = AggchainData::Generic {
            aggchain_params: Digest([0xFFu8; 32]),
            aggchain_vkey: [u32::MAX; 8],
        };

        let zero_copy = AggchainDataZeroCopy::from(&max_generic);
        let reconstructed = AggchainData::try_from(&zero_copy).unwrap();

        match (&max_generic, &reconstructed) {
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
                assert_eq!(orig_params, rec_params);
                assert_eq!(orig_vkey, rec_vkey);
            }
            _ => panic!("Expected Generic variants"),
        }

        // Test with zero values
        let zero_ecdsa = AggchainData::ECDSA {
            signer: Address::new([0u8; 20]),
            signature: Signature::new(U256::ZERO, U256::ZERO, false),
        };

        let zero_copy = AggchainDataZeroCopy::from(&zero_ecdsa);
        let reconstructed = AggchainData::try_from(&zero_copy).unwrap();

        match (&zero_ecdsa, &reconstructed) {
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
                assert_eq!(orig_signer, rec_signer);
                assert_eq!(orig_sig.r(), rec_sig.r());
                assert_eq!(orig_sig.s(), rec_sig.s());
                assert_eq!(orig_sig.v(), rec_sig.v());
            }
            _ => panic!("Expected ECDSA variants"),
        }

        // Test that padding doesn't interfere with data
        let mut zero_copy_with_padding = AggchainDataZeroCopy::from(&max_ecdsa);
        zero_copy_with_padding._padding = [0xAAu8; 7]; // Set padding to non-zero values to verify it's ignored

        let reconstructed = AggchainData::try_from(&zero_copy_with_padding).unwrap();

        match (&max_ecdsa, &reconstructed) {
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
                assert_eq!(orig_signer, rec_signer);
                assert_eq!(orig_sig.r(), rec_sig.r());
                assert_eq!(orig_sig.s(), rec_sig.s());
                assert_eq!(orig_sig.v(), rec_sig.v());
            }
            _ => panic!("Expected ECDSA variants"),
        }

        // Test Generic serialization through MultiBatchHeader
        let original = create_sample_multi_batch_header_generic();
        let zero_copy: MultiBatchHeaderZeroCopy = (&original).into();
        let reconstructed: MultiBatchHeader = MultiBatchHeader::try_from(&zero_copy).unwrap();

        match (&original.aggchain_proof, &reconstructed.aggchain_proof) {
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
                assert_eq!(orig_params, rec_params);
                assert_eq!(orig_vkey, rec_vkey);
            }
            _ => panic!("Expected Generic aggchain data"),
        }
    }
}
