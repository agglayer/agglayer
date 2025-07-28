#![allow(clippy::too_many_arguments)]

use std::hash::Hash;

use agglayer_primitives::{
    keccak::{Hasher, Keccak256Hasher},
    Address, Digest, Signature, U256,
};
use agglayer_tries::proof::{SmtMerkleProof, SmtNonInclusionProof};
use bytemuck::{Pod, Zeroable};
use serde::{de::DeserializeOwned, Serialize};
use serde_with::serde_as;
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

/// Type aliases for agglayer_tries proof types to improve readability
pub type BalanceMerkleProof = SmtMerkleProof<Keccak256Hasher, 192>;
pub type NullifierNonInclusionProof = SmtNonInclusionProof<Keccak256Hasher, 64>;

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
    assert!(std::mem::size_of::<ClaimZeroCopy>() == 3352);
    assert!(std::mem::align_of::<ClaimZeroCopy>() == 1);
};

const _MULTI_BATCH_HEADER_SIZE: () = {
    assert!(std::mem::size_of::<MultiBatchHeaderZeroCopy>() == 184);
    assert!(std::mem::align_of::<MultiBatchHeaderZeroCopy>() == 8);
};

const _BRIDGE_EXIT_ZERO_COPY_SIZE: () = {
    assert!(std::mem::size_of::<BridgeExitZeroCopy>() == 116);
    assert!(std::mem::align_of::<BridgeExitZeroCopy>() == 4);
};

const _AGGCHAIN_DATA_ZERO_COPY_SIZE: () = {
    assert!(std::mem::size_of::<AggchainDataZeroCopy>() == 96);
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
    pub origin_token_address: [u8; 20],
    /// Destination address (20 bytes)
    pub dest_address: [u8; 20],
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
    pub origin_token_address: [u8; 20],
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

/// Zero-copy compatible merkle proof types for specific depths.
/// These are newtype wrappers for the specific depths we actually use.
///
/// Balance merkle proof (192 siblings, 6144 bytes)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BalanceMerkleProofZeroCopy(pub [Hash256; 192]);

/// LET merkle proof (32 siblings, 1024 bytes)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct LETMerkleProofZeroCopy(pub [Hash256; 32]);

// SAFETY: BalanceMerkleProofZeroCopy has a stable C-compatible memory layout
// - #[repr(C)] ensures C-compatible layout
// - All fields are fixed-size arrays of Hash256, which are Pod and Zeroable
// - Cannot use derive due to large array size (6144 bytes) exceeding bytemuck's
//   derive limits
unsafe impl Pod for BalanceMerkleProofZeroCopy {}
unsafe impl Zeroable for BalanceMerkleProofZeroCopy {}

/// Helper function to convert BalanceMerkleProof to BalanceMerkleProofZeroCopy
fn balance_merkle_proof_to_zero_copy(proof: &BalanceMerkleProof) -> BalanceMerkleProofZeroCopy {
    BalanceMerkleProofZeroCopy(digest_array_to_bytes(&proof.siblings))
}

/// Helper function to convert BalanceMerkleProofZeroCopy to BalanceMerkleProof
fn balance_merkle_proof_from_zero_copy(zc: &BalanceMerkleProofZeroCopy) -> BalanceMerkleProof {
    BalanceMerkleProof {
        siblings: bytes_array_to_digests(&zc.0),
    }
}

/// Helper function to convert LETMerkleProof to LETMerkleProofZeroCopy
fn let_merkle_proof_to_zero_copy(
    proof: &LETMerkleProof<Keccak256Hasher>,
) -> LETMerkleProofZeroCopy {
    LETMerkleProofZeroCopy(digest_array_to_bytes(&proof.siblings))
}

/// Helper function to convert LETMerkleProofZeroCopy to LETMerkleProof
fn let_merkle_proof_from_zero_copy(zc: &LETMerkleProofZeroCopy) -> LETMerkleProof<Keccak256Hasher> {
    LETMerkleProof {
        siblings: bytes_array_to_digests(&zc.0),
    }
}

/// Zero-copy compatible SmtNonInclusionProof for bytemuck operations.
/// This captures the variable-length siblings as a fixed-size array with length
/// tracking.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SmtNonInclusionProofZeroCopy {
    /// Number of actual siblings (u8, max 64)
    pub num_siblings: u8,
    /// Padding to ensure proper alignment for 4-byte boundaries.
    /// The num_siblings field is 1 byte, so we need 3 bytes of padding
    /// to align the siblings array to 4-byte boundaries for optimal memory
    /// access.
    pub _padding: [u8; 3],
    /// Siblings array (64 * 32 = 2048 bytes)
    pub siblings: [Hash256; 64],
}

// SAFETY: This struct has a stable C-compatible memory layout
// - #[repr(C)] ensures C-compatible layout
// - Fields are ordered by alignment (u8 first, then padding, then array)
// - Total size is 2052 bytes: 1+3+2048 = 2052
// - Padding ensures 4-byte alignment for the siblings array
// - Cannot use derive due to large array size and explicit padding requirements
unsafe impl Pod for SmtNonInclusionProofZeroCopy {}
unsafe impl Zeroable for SmtNonInclusionProofZeroCopy {}

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
        let siblings = std::array::from_fn(|i| {
            if i < num_siblings as usize {
                proof.siblings[i].0
            } else {
                [0u8; 32]
            }
        });

        Self {
            num_siblings,
            _padding: [0; 3],
            siblings,
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
        let siblings: Vec<Digest> = zc
            .siblings
            .iter()
            .take(num_siblings)
            .map(|s| Digest(*s))
            .collect();
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

/// Zero-copy compatible Claim for bytemuck operations.
/// This union-like structure can hold either Mainnet or Rollup claim data.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClaimZeroCopy {
    /// Claim type (u8: 0=Mainnet, 1=Rollup)
    pub claim_type: u8,
    /// Padding to ensure proper alignment for 8-byte boundaries.
    /// The claim_type field is 1 byte, so we need 7 bytes of padding
    /// to align the claim_data array to 8-byte boundaries for optimal memory
    /// access. This is particularly important for the large claim_data
    /// array (3344 bytes).
    pub _padding: [u8; 7],
    /// Union of claim data - size matches the larger of the two claim types
    /// Mainnet: 2288 bytes, Rollup: 3344 bytes, so we use 3344 bytes
    pub claim_data: [u8; 3344],
}

// SAFETY: This struct has a stable C-compatible memory layout
// - #[repr(C)] ensures C-compatible layout
// - Fields are ordered by alignment (u8 first, then padding, then array)
// - Total size is 3352 bytes: 1+7+3344 = 3352
// - Padding ensures 8-byte alignment for the large claim_data array
// - Cannot use derive due to complex field layout and explicit padding
// requirements
unsafe impl Pod for ClaimZeroCopy {}
unsafe impl Zeroable for ClaimZeroCopy {}

impl From<&Claim> for ClaimZeroCopy {
    fn from(claim: &Claim) -> Self {
        match claim {
            Claim::Mainnet(mainnet_claim) => {
                let mainnet_zero_copy = ClaimFromMainnetZeroCopy::from(&**mainnet_claim);
                let mainnet_bytes = bytemuck::bytes_of(&mainnet_zero_copy);
                let mut claim_data = [0u8; 3344];
                claim_data[..mainnet_bytes.len()].copy_from_slice(mainnet_bytes);

                Self {
                    claim_type: 0,
                    _padding: [0; 7],
                    claim_data,
                }
            }
            Claim::Rollup(rollup_claim) => {
                let rollup_zero_copy = ClaimFromRollupZeroCopy::from(&**rollup_claim);
                let rollup_bytes = bytemuck::bytes_of(&rollup_zero_copy);
                let mut claim_data = [0u8; 3344];
                claim_data[..rollup_bytes.len()].copy_from_slice(rollup_bytes);

                Self {
                    claim_type: 1,
                    _padding: [0; 7],
                    claim_data,
                }
            }
        }
    }
}

impl TryFrom<&ClaimZeroCopy> for Claim {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(zc: &ClaimZeroCopy) -> Result<Self, Self::Error> {
        match zc.claim_type {
            0 => {
                // Mainnet claim
                let mainnet_size = std::mem::size_of::<ClaimFromMainnetZeroCopy>();
                let mainnet_zero_copy = bytemuck::pod_read_unaligned::<ClaimFromMainnetZeroCopy>(
                    &zc.claim_data[..mainnet_size],
                );
                Ok(Claim::Mainnet(Box::new((&mainnet_zero_copy).into())))
            }
            1 => {
                // Rollup claim
                let rollup_size = std::mem::size_of::<ClaimFromRollupZeroCopy>();
                let rollup_zero_copy = bytemuck::pod_read_unaligned::<ClaimFromRollupZeroCopy>(
                    &zc.claim_data[..rollup_size],
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

/// Zero-copy compatible AggchainData for bytemuck operations.
/// This captures the fixed-size data from AggchainData variants.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AggchainDataZeroCopy {
    /// Aggchain proof type (u8: 0=ECDSA, 1=Generic)
    pub aggchain_proof_type: u8,
    /// Padding to ensure proper alignment for 8-byte boundaries.
    /// The aggchain_proof_type field is 1 byte, so we need 7 bytes of padding
    /// to align the aggchain_proof_data array to 8-byte boundaries for optimal
    /// memory access. This is important for the large proof data array (85
    /// bytes).
    pub _padding: [u8; 7],
    /// Aggchain proof data (fixed buffer sized for the larger variant)
    /// For ECDSA: 20 bytes signer + 65 bytes signature = 85 bytes
    /// For Generic: 32 bytes aggchain_params + 32 bytes vkey = 64 bytes
    pub aggchain_proof_data: [u8; 85],
    /// End padding to ensure the struct size is a multiple of 8 bytes.
    /// This ensures proper alignment when the struct is used in arrays or
    /// as part of larger structures. Total size: 96 bytes.
    pub _end_padding: [u8; 3],
}

// SAFETY: This struct has a stable C-compatible memory layout
// - #[repr(C)] ensures C-compatible layout
// - Fields are ordered by alignment (u8 first, then padding, then array, then
//   end padding)
// - Total size is 96 bytes: 1+7+85+3 = 96 (multiple of 8 for optimal array
//   alignment)
// - Cannot use derive due to [u8; 85] not being supported by bytemuck derive
unsafe impl Pod for AggchainDataZeroCopy {}
unsafe impl Zeroable for AggchainDataZeroCopy {}

impl From<&AggchainData> for AggchainDataZeroCopy {
    fn from(aggchain_data: &AggchainData) -> Self {
        match aggchain_data {
            AggchainData::ECDSA { signer, signature } => {
                let mut aggchain_proof_data = [0u8; 85];
                // Copy signer address (20 bytes) + signature bytes (65 bytes)
                aggchain_proof_data[..20].copy_from_slice(signer.as_slice());
                let sig_bytes = signature.as_bytes();
                aggchain_proof_data[20..85].copy_from_slice(&sig_bytes[..65]);

                Self {
                    aggchain_proof_type: 0,
                    _padding: [0; 7],
                    aggchain_proof_data,
                    _end_padding: [0; 3],
                }
            }
            AggchainData::Generic {
                aggchain_params,
                aggchain_vkey,
            } => {
                let mut aggchain_proof_data = [0u8; 85];
                // Copy aggchain_params (32 bytes)
                aggchain_proof_data[..32].copy_from_slice(aggchain_params.as_slice());
                // Convert vkey from [u32; 8] to bytes using bytemuck
                let vkey_bytes = bytemuck::cast_slice::<u32, u8>(aggchain_vkey);
                aggchain_proof_data[32..64].copy_from_slice(vkey_bytes);

                Self {
                    aggchain_proof_type: 1,
                    _padding: [0; 7],
                    aggchain_proof_data,
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
            0 => {
                // ECDSA - reconstruct signer (20 bytes) + signature (65 bytes)
                let signer = Address::from(
                    <[u8; 20]>::try_from(&zero_copy.aggchain_proof_data[..20])
                        .map_err(|e| format!("Failed to convert signer bytes: {}", e))?,
                );
                let signature_bytes = <[u8; 65]>::try_from(&zero_copy.aggchain_proof_data[20..85])
                    .map_err(|e| format!("Failed to convert signature bytes: {}", e))?;
                let signature = Signature::try_from(&signature_bytes[..])
                    .map_err(|e| format!("Failed to parse signature: {}", e))?;
                Ok(AggchainData::ECDSA { signer, signature })
            }
            1 => {
                // Generic
                let aggchain_params = Digest::from(
                    <[u8; 32]>::try_from(&zero_copy.aggchain_proof_data[..32])
                        .map_err(|e| format!("Failed to convert aggchain_params bytes: {}", e))?,
                );
                // Reconstruct vkey from bytes using bytemuck
                let vkey_bytes = <[u8; 32]>::try_from(&zero_copy.aggchain_proof_data[32..64])
                    .map_err(|e| format!("Failed to convert vkey bytes: {}", e))?;
                let aggchain_vkey = bytemuck::cast::<[u8; 32], [u32; 8]>(vkey_bytes);
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
    pub prev_pessimistic_root: Hash256,
    /// L1 info root used to import bridge exits (32 bytes)
    pub l1_info_root: Hash256,
    /// Aggchain proof data (zero-copy struct)
    pub aggchain_proof: AggchainDataZeroCopy,
}

// SAFETY: This struct has a stable C-compatible memory layout
// - #[repr(C)] ensures C-compatible layout
// - Fields are ordered by alignment (u64 first, then u32, then arrays, then
//   struct)
// - Total size is 184 bytes (includes internal padding for optimal alignment)
// - Cannot use derive due to AggchainDataZeroCopy not being supported by
//   bytemuck derive
unsafe impl Pod for MultiBatchHeaderZeroCopy {}
unsafe impl Zeroable for MultiBatchHeaderZeroCopy {}

/// Represents the chain state transition for the pessimistic proof.
#[serde_as]
#[derive(Clone, Debug, Serialize)]
pub struct MultiBatchHeader<H>
where
    H: Hasher,
    H::Digest: Eq + Hash + Copy + Serialize + DeserializeOwned,
{
    /// Network that emitted this [MultiBatchHeader].
    pub origin_network: NetworkId,
    /// Current certificate height of the L2 chain.
    pub height: u64,
    /// Previous pessimistic root.
    pub prev_pessimistic_root: H::Digest,
    /// List of bridge exits created in this batch.
    pub bridge_exits: Vec<BridgeExit>,
    /// List of imported bridge exits claimed in this batch.
    pub imported_bridge_exits: Vec<(ImportedBridgeExit, NullifierPath<H>)>,
    /// L1 info root used to import bridge exits.
    pub l1_info_root: H::Digest,
    /// Token balances of the origin network before processing bridge events,
    /// with Merkle proofs of these balances in the local balance tree.
    pub balances_proofs: Vec<(TokenInfo, (U256, LocalBalancePath<H>))>,
    /// Aggchain proof.
    pub aggchain_proof: AggchainData,
}

impl<H> From<&MultiBatchHeader<H>> for MultiBatchHeaderZeroCopy
where
    H: Hasher,
    H::Digest: Eq + Hash + Copy + Serialize + DeserializeOwned + AsRef<[u8]> + From<Hash256>,
{
    fn from(self_: &MultiBatchHeader<H>) -> Self {
        MultiBatchHeaderZeroCopy {
            height: self_.height,
            origin_network: self_.origin_network.to_u32(),
            bridge_exits_count: self_.bridge_exits.len() as u32,
            imported_bridge_exits_count: self_.imported_bridge_exits.len() as u32,
            balances_proofs_count: self_.balances_proofs.len() as u32,
            prev_pessimistic_root: self_
                .prev_pessimistic_root
                .as_ref()
                .try_into()
                .expect("Digest must be exactly 32 bytes"),
            l1_info_root: self_
                .l1_info_root
                .as_ref()
                .try_into()
                .expect("Digest must be exactly 32 bytes"),
            aggchain_proof: (&self_.aggchain_proof).into(),
        }
    }
}

impl<H> TryFrom<&MultiBatchHeaderZeroCopy> for MultiBatchHeader<H>
where
    H: Hasher,
    H::Digest: Eq + Hash + Copy + Serialize + DeserializeOwned + AsRef<[u8]> + From<Hash256>,
{
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(zero_copy: &MultiBatchHeaderZeroCopy) -> Result<Self, Self::Error> {
        let origin_network = NetworkId::new(zero_copy.origin_network);
        let prev_pessimistic_root =
            <H::Digest as From<Hash256>>::from(zero_copy.prev_pessimistic_root);
        let l1_info_root = <H::Digest as From<Hash256>>::from(zero_copy.l1_info_root);

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

// Specific implementation for Keccak256Hasher with zero-copy component helpers
impl MultiBatchHeader<Keccak256Hasher> {
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
    ) -> Result<MultiBatchHeaderRef<'a, Keccak256Hasher>, Box<dyn std::error::Error + Send + Sync>>
    {
        // Deserialize header using pod_read_unaligned for robustness against alignment
        // issues
        let header_zero_copy =
            bytemuck::pod_read_unaligned::<MultiBatchHeaderZeroCopy>(header_bytes);

        // Create borrowed slices for zero-copy components using try_cast_slice
        let bridge_exits: &'a [BridgeExitZeroCopy] = if bridge_exits_bytes.is_empty() {
            &[]
        } else {
            bytemuck::try_cast_slice(bridge_exits_bytes)
                .map_err(|e| format!("Failed to cast bridge_exits_bytes: {}", e))?
        };

        let imported_bridge_exits: &'a [ImportedBridgeExitZeroCopy] =
            if imported_bridge_exits_bytes.is_empty() {
                &[]
            } else {
                bytemuck::try_cast_slice(imported_bridge_exits_bytes)
                    .map_err(|e| format!("Failed to cast imported_bridge_exits_bytes: {}", e))?
            };

        let nullifier_paths: &'a [SmtNonInclusionProofZeroCopy] =
            if nullifier_paths_bytes.is_empty() {
                &[]
            } else {
                bytemuck::try_cast_slice(nullifier_paths_bytes)
                    .map_err(|e| format!("Failed to cast nullifier_paths_bytes: {}", e))?
            };

        let balances_proofs: &'a [BalanceProofEntryZeroCopy] = if balances_proofs_bytes.is_empty() {
            &[]
        } else {
            bytemuck::try_cast_slice(balances_proofs_bytes)
                .map_err(|e| format!("Failed to cast balances_proofs_bytes: {}", e))?
        };

        let balance_merkle_paths: &'a [BalanceMerkleProofZeroCopy] =
            if balance_merkle_paths_bytes.is_empty() {
                &[]
            } else {
                bytemuck::try_cast_slice(balance_merkle_paths_bytes)
                    .map_err(|e| format!("Failed to cast balance_merkle_paths_bytes: {}", e))?
            };

        // Extract aggchain_proof from header
        let aggchain_proof = AggchainData::try_from(&header_zero_copy.aggchain_proof)?;

        // Reconstruct the MultiBatchHeaderRef from zero-copy components
        let origin_network = NetworkId::new(header_zero_copy.origin_network);
        let prev_pessimistic_root = <<Keccak256Hasher as Hasher>::Digest as From<Hash256>>::from(
            header_zero_copy.prev_pessimistic_root,
        );
        let l1_info_root = <<Keccak256Hasher as Hasher>::Digest as From<Hash256>>::from(
            header_zero_copy.l1_info_root,
        );

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
pub struct MultiBatchHeaderRef<'a, H>
where
    H: Hasher,
    H::Digest: Eq + Hash + Copy + Serialize + DeserializeOwned + AsRef<[u8]> + From<Hash256>,
{
    /// Network that emitted this [MultiBatchHeaderRef].
    pub origin_network: NetworkId,
    /// Current certificate height of the L2 chain.
    pub height: u64,
    /// Previous pessimistic root.
    pub prev_pessimistic_root: H::Digest,
    /// List of bridge exits created in this batch (borrowed).
    pub bridge_exits: &'a [BridgeExitZeroCopy],
    /// List of imported bridge exits claimed in this batch (borrowed).
    pub imported_bridge_exits: &'a [ImportedBridgeExitZeroCopy],
    /// Nullifier paths for imported bridge exits (borrowed).
    pub nullifier_paths: &'a [SmtNonInclusionProofZeroCopy],
    /// L1 info root used to import bridge exits.
    pub l1_info_root: H::Digest,
    /// Token balances of the origin network before processing bridge events
    /// (borrowed).
    pub balances_proofs: &'a [BalanceProofEntryZeroCopy],
    /// Balance Merkle paths (borrowed).
    pub balance_merkle_paths: &'a [BalanceMerkleProofZeroCopy],
    /// Aggchain proof.
    pub aggchain_proof: AggchainData,
}

// Specific implementation for Keccak256Hasher
impl MultiBatchHeaderRef<'_, Keccak256Hasher> {
    /// Convert to owned MultiBatchHeader by cloning all borrowed data.
    /// This is a specialized version for Keccak256Hasher to avoid type
    /// conflicts.
    pub fn to_owned_keccak(
        &self,
    ) -> Result<MultiBatchHeader<Keccak256Hasher>, Box<dyn std::error::Error + Send + Sync>> {
        // Convert bridge_exits
        let bridge_exits: Vec<BridgeExit> = self.bridge_exits.iter().map(|be| be.into()).collect();

        // Convert imported_bridge_exits and nullifier_paths
        let imported_bridge_exits: Vec<(ImportedBridgeExit, NullifierPath<Keccak256Hasher>)> = self
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
        let balances_proofs: Vec<(TokenInfo, (U256, LocalBalancePath<Keccak256Hasher>))> = self
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
    fn deep_equals<H>(original: &MultiBatchHeader<H>, reconstructed: &MultiBatchHeader<H>) -> bool
    where
        H: Hasher,
        H::Digest: Eq + Hash + Copy + Serialize + DeserializeOwned + AsRef<[u8]> + From<Hash256>,
    {
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
    fn create_sample_multi_batch_header() -> MultiBatchHeader<Keccak256Hasher> {
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
    fn create_sample_multi_batch_header_generic() -> MultiBatchHeader<Keccak256Hasher> {
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
    fn create_sample_multi_batch_header_rollup() -> MultiBatchHeader<Keccak256Hasher> {
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
    fn create_sample_multi_batch_header_mixed() -> MultiBatchHeader<Keccak256Hasher> {
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

        let result: Result<MultiBatchHeader<Keccak256Hasher>, _> =
            MultiBatchHeader::try_from(&zero_copy);
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

        let result = MultiBatchHeader::<Keccak256Hasher>::from_zero_copy_components(
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
    fn test_zero_copy_recovery(original: &MultiBatchHeader<Keccak256Hasher>) {
        // Use the new helper function to get all zero-copy components
        let components = original
            .to_zero_copy_components()
            .expect("Failed to convert to zero-copy components");

        // Reconstruct the MultiBatchHeaderRef (borrowed view) from zero-copy components
        let borrowed_view = MultiBatchHeader::<Keccak256Hasher>::from_zero_copy_components(
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
            .to_owned_keccak()
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
    fn test_borrowed_view_recovery(original: &MultiBatchHeader<Keccak256Hasher>) {
        // Use the new helper function to get all zero-copy components
        let components = original
            .to_zero_copy_components()
            .expect("Failed to convert to zero-copy components");

        // Reconstruct the MultiBatchHeaderRef (borrowed view) from zero-copy components
        let borrowed_view = MultiBatchHeader::<Keccak256Hasher>::from_zero_copy_components(
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
            .to_owned_keccak()
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
        let reconstructed: MultiBatchHeader<Keccak256Hasher> =
            MultiBatchHeader::try_from(&zero_copy).unwrap();

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
        assert_eq!(std::mem::size_of::<ClaimZeroCopy>(), 3352);
        assert_eq!(std::mem::size_of::<MultiBatchHeaderZeroCopy>(), 184);
        assert_eq!(std::mem::size_of::<BalanceProofEntryZeroCopy>(), 64);
        assert_eq!(std::mem::size_of::<AggchainDataZeroCopy>(), 96);

        // Compile-time alignment assertions
        assert_eq!(std::mem::align_of::<BridgeExitZeroCopy>(), 4);
        assert_eq!(std::mem::align_of::<BalanceMerkleProofZeroCopy>(), 1);
        assert_eq!(std::mem::align_of::<SmtNonInclusionProofZeroCopy>(), 1);
        assert_eq!(std::mem::align_of::<ClaimZeroCopy>(), 1);
        assert_eq!(std::mem::align_of::<MultiBatchHeaderZeroCopy>(), 8);
        assert_eq!(std::mem::align_of::<BalanceProofEntryZeroCopy>(), 4);
        assert_eq!(std::mem::align_of::<AggchainDataZeroCopy>(), 1);

        // Runtime size and alignment verification for large structs
        let balance_proof = BalanceMerkleProofZeroCopy([[0u8; 32]; 192]);
        assert_eq!(std::mem::size_of_val(&balance_proof), 6144);
        assert_eq!(std::mem::align_of_val(&balance_proof), 1);

        let smt_non_inclusion_proof = SmtNonInclusionProofZeroCopy {
            num_siblings: 64,
            _padding: [0; 3],
            siblings: [[0u8; 32]; 64],
        };
        assert_eq!(std::mem::size_of_val(&smt_non_inclusion_proof), 2052);
        assert_eq!(std::mem::align_of_val(&smt_non_inclusion_proof), 1);

        let claim = ClaimZeroCopy {
            claim_type: 0,
            _padding: [0; 7],
            claim_data: [0u8; 3344],
        };
        assert_eq!(std::mem::size_of_val(&claim), 3352);
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
                aggchain_proof_data: [0u8; 85],
                _end_padding: [0; 3],
            },
        };
        assert_eq!(std::mem::size_of_val(&header), 184);
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
        let reconstructed: MultiBatchHeader<Keccak256Hasher> =
            MultiBatchHeader::try_from(&zero_copy).unwrap();

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
