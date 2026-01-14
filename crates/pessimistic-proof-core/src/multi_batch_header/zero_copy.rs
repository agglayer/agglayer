use core::{fmt, mem};

use agglayer_primitives::{Address, Digest, Signature, U256};
use bytemuck::{Pod, Zeroable};
use static_assertions::const_assert_eq;
use unified_bridge::{
    BridgeExit, Claim, ClaimFromMainnet, ClaimFromRollup, GlobalIndex, ImportedBridgeExit,
    L1InfoTreeLeaf, L1InfoTreeLeafInner, LeafType, MerkleProof, NetworkId, TokenInfo,
    LETMerkleProof,
};

use crate::{
    aggchain_data::{AggchainData, AggchainProof, MultiSignature, Vkey},
    local_balance_tree::LocalBalancePath,
    nullifier_tree::NullifierPath,
};
use super::MultiBatchHeader;

// Endianness guard: zero-copy wire format assumes little-endian targets.
#[cfg(target_endian = "big")]
compile_error!("MultiBatchHeader zero-copy wire format assumes little-endian targets");

pub const AGGCHAIN_PROOF_TYPE_LEGACY_ECDSA: u8 = 0;
pub const AGGCHAIN_PROOF_TYPE_MULTISIG_ONLY: u8 = 2;
pub const AGGCHAIN_PROOF_TYPE_MULTISIG_AND_AGGCHAIN: u8 = 3;

pub const CLAIM_TYPE_MAINNET: u8 = 0;
pub const CLAIM_TYPE_ROLLUP: u8 = 1;

pub type Hash32 = [u8; 32];
pub type AddressBytes = [u8; 20];
pub type U256Bytes = [u8; 32];

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bytes65([u8; 65]);

unsafe impl Zeroable for Bytes65 {}
unsafe impl Pod for Bytes65 {}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClaimDataBytes([u8; 3584]);

unsafe impl Zeroable for ClaimDataBytes {}
unsafe impl Pod for ClaimDataBytes {}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hash32x192([[u8; 32]; 192]);

unsafe impl Zeroable for Hash32x192 {}
unsafe impl Pod for Hash32x192 {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZeroCopyError {
    InvalidSize { expected: usize, actual: usize },
    CountMismatch { field: &'static str, header: u32, actual: usize },
    InvalidDiscriminant { field: &'static str, value: u8 },
    InvalidSiblingsCount { value: u8 },
    InvalidIndex { field: &'static str, index: u32 },
    BytemuckCast,
}

impl fmt::Display for ZeroCopyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZeroCopyError::InvalidSize { expected, actual } => {
                write!(f, "Invalid size: expected {expected}, actual {actual}")
            }
            ZeroCopyError::CountMismatch {
                field,
                header,
                actual,
            } => write!(
                f,
                "Count mismatch for {field}: header {header}, actual {actual}"
            ),
            ZeroCopyError::InvalidDiscriminant { field, value } => {
                write!(f, "Invalid discriminant for {field}: {value}")
            }
            ZeroCopyError::InvalidSiblingsCount { value } => {
                write!(f, "Invalid siblings count: {value}")
            }
            ZeroCopyError::InvalidIndex { field, index } => {
                write!(f, "Invalid index for {field}: {index}")
            }
            ZeroCopyError::BytemuckCast => write!(f, "Bytemuck cast error"),
        }
    }
}

#[repr(C, align(1))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct MultiBatchHeaderWire {
    pub bridge_exits_count: [u8; 4],
    pub imported_bridge_exits_count: [u8; 4],
    pub balances_proofs_count: [u8; 4],
    pub multisig_signatures_count: [u8; 4],
    pub multisig_expected_signers_count: [u8; 4],
    pub height: [u8; 8],
    pub origin_network: [u8; 4],
    pub prev_pessimistic_root: Hash32,
    pub l1_info_root: Hash32,
    pub certificate_id: Hash32,
    pub aggchain_proof_type: u8,
    pub _aggchain_padding: [u8; 7],
    pub multisig_threshold: [u8; 8],
    pub ecdsa_signer: AddressBytes,
    pub ecdsa_signature: Bytes65,
    pub _sig_padding: [u8; 3],
    pub generic_params: Hash32,
    pub generic_vkey: Hash32,
}

#[repr(C, align(1))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct BridgeExitWire {
    pub origin_network: [u8; 4],
    pub dest_network: [u8; 4],
    pub origin_token_address: AddressBytes,
    pub dest_address: AddressBytes,
    pub amount: U256Bytes,
    pub metadata_hash: Hash32,
    pub leaf_type: u8,
    pub has_metadata: u8,
    pub _padding: [u8; 2],
}

#[repr(C, align(1))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct TokenInfoWire {
    pub origin_network: [u8; 4],
    pub origin_token_address: AddressBytes,
}

#[repr(C, align(1))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct MerkleProofWire {
    pub siblings: [Hash32; 32],
    pub root: Hash32,
}

#[repr(C, align(1))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct L1InfoTreeLeafWire {
    pub l1_info_tree_index: [u8; 4],
    pub rer: Hash32,
    pub mer: Hash32,
    pub global_exit_root: Hash32,
    pub block_hash: Hash32,
    pub timestamp: [u8; 8],
}

#[repr(C, align(1))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct ClaimFromMainnetWire {
    pub proof_leaf_mer: MerkleProofWire,
    pub proof_ger_l1root: MerkleProofWire,
    pub l1_leaf: L1InfoTreeLeafWire,
}

#[repr(C, align(1))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct ClaimFromRollupWire {
    pub proof_leaf_ler: MerkleProofWire,
    pub proof_ler_rer: MerkleProofWire,
    pub proof_ger_l1root: MerkleProofWire,
    pub l1_leaf: L1InfoTreeLeafWire,
}

#[repr(C, align(1))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct ImportedBridgeExitWire {
    pub global_index_leaf: [u8; 4],
    pub global_index_rollup: [u8; 4],
    pub bridge_exit: BridgeExitWire,
    pub claim_type: u8,
    pub claim_data: ClaimDataBytes,
    pub _padding: [u8; 7],
}

#[repr(C, align(1))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct SmtNonInclusionProofWire {
    pub num_siblings: u8,
    pub _padding: [u8; 3],
    pub siblings: [Hash32; 64],
}

#[repr(C, align(1))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct BalanceMerkleProofWire {
    pub siblings: Hash32x192,
}

#[repr(C, align(1))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct BalanceProofEntryWire {
    pub token_info: TokenInfoWire,
    pub balance: U256Bytes,
}

#[repr(C, align(1))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct MultisigSignatureEntryWire {
    pub signer_index: [u8; 4],
    pub signature: Bytes65,
    pub _padding: [u8; 3],
}

pub struct ZeroCopyComponents {
    pub header_bytes: Vec<u8>,
    pub bridge_exits_bytes: Vec<u8>,
    pub imported_bridge_exits_bytes: Vec<u8>,
    pub nullifier_paths_bytes: Vec<u8>,
    pub balances_proofs_bytes: Vec<u8>,
    pub balance_merkle_paths_bytes: Vec<u8>,
    pub multisig_signatures_bytes: Vec<u8>,
    pub multisig_expected_signers_bytes: Vec<u8>,
}

#[repr(C, align(1))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct MultiBatchHeaderOffsets {
    pub bridge_exits_offset: [u8; 4],
    pub imported_bridge_exits_offset: [u8; 4],
    pub nullifier_paths_offset: [u8; 4],
    pub balances_proofs_offset: [u8; 4],
    pub balance_merkle_paths_offset: [u8; 4],
    pub multisig_signatures_offset: [u8; 4],
    pub multisig_expected_signers_offset: [u8; 4],
    pub total_len: [u8; 4],
}

#[derive(Debug)]
pub struct MultiBatchHeaderRef<'a> {
    pub origin_network: NetworkId,
    pub height: u64,
    pub prev_pessimistic_root: Digest,
    pub l1_info_root: Digest,
    pub certificate_id: Digest,
    pub bridge_exits: &'a [BridgeExitWire],
    pub imported_bridge_exits: &'a [ImportedBridgeExitWire],
    pub nullifier_paths: &'a [SmtNonInclusionProofWire],
    pub balances_proofs: &'a [BalanceProofEntryWire],
    pub balance_merkle_paths: &'a [BalanceMerkleProofWire],
    pub aggchain_data: AggchainData,
}

const_assert_eq!(mem::size_of::<MultiBatchHeaderWire>(), 296);
const_assert_eq!(mem::size_of::<BridgeExitWire>(), 116);
const_assert_eq!(mem::size_of::<TokenInfoWire>(), 24);
const_assert_eq!(mem::size_of::<MerkleProofWire>(), 1056);
const_assert_eq!(mem::size_of::<L1InfoTreeLeafWire>(), 140);
const_assert_eq!(mem::size_of::<ClaimFromMainnetWire>(), 2252);
const_assert_eq!(mem::size_of::<ClaimFromRollupWire>(), 3308);
const_assert_eq!(mem::size_of::<ImportedBridgeExitWire>(), 3716);
const_assert_eq!(mem::size_of::<SmtNonInclusionProofWire>(), 2052);
const_assert_eq!(mem::size_of::<BalanceMerkleProofWire>(), 6144);
const_assert_eq!(mem::size_of::<BalanceProofEntryWire>(), 56);
const_assert_eq!(mem::size_of::<MultisigSignatureEntryWire>(), 72);
const_assert_eq!(mem::size_of::<MultiBatchHeaderOffsets>(), 32);

impl MultiBatchHeader {
    pub fn to_zero_copy_components(&self) -> Result<ZeroCopyComponents, ZeroCopyError> {
        let mut header = MultiBatchHeaderWire::zeroed();
        header.bridge_exits_count = u32_to_le_bytes(self.bridge_exits.len())?;
        header.imported_bridge_exits_count = u32_to_le_bytes(self.imported_bridge_exits.len())?;
        header.balances_proofs_count = u32_to_le_bytes(self.balances_proofs.len())?;
        header.height = self.height.to_le_bytes();
        header.origin_network = self.origin_network.to_u32().to_le_bytes();
        header.prev_pessimistic_root = self.prev_pessimistic_root.0;
        header.l1_info_root = self.l1_info_root.0;
        header.certificate_id = self.certificate_id.0;

        let mut multisig_signatures_bytes = Vec::new();
        let mut multisig_expected_signers_bytes = Vec::new();
        let mut multisig_signatures_count = 0u32;
        let mut multisig_expected_signers_count = 0u32;
        let mut multisig_threshold = 0u64;

        match &self.aggchain_data {
            AggchainData::LegacyEcdsa { signer, signature } => {
                header.aggchain_proof_type = AGGCHAIN_PROOF_TYPE_LEGACY_ECDSA;
                header.ecdsa_signer = signer.into_array();
                header.ecdsa_signature = Bytes65(signature.as_bytes());
            }
            AggchainData::MultisigOnly(multisig) => {
                header.aggchain_proof_type = AGGCHAIN_PROOF_TYPE_MULTISIG_ONLY;
                let components = multisig_to_components(multisig)?;
                multisig_signatures_bytes = components.signatures_bytes;
                multisig_expected_signers_bytes = components.expected_signers_bytes;
                multisig_signatures_count = components.signatures_count;
                multisig_expected_signers_count = components.expected_signers_count;
                multisig_threshold = components.threshold;
            }
            AggchainData::MultisigAndAggchainProof {
                multisig,
                aggchain_proof,
            } => {
                header.aggchain_proof_type = AGGCHAIN_PROOF_TYPE_MULTISIG_AND_AGGCHAIN;
                let components = multisig_to_components(multisig)?;
                multisig_signatures_bytes = components.signatures_bytes;
                multisig_expected_signers_bytes = components.expected_signers_bytes;
                multisig_signatures_count = components.signatures_count;
                multisig_expected_signers_count = components.expected_signers_count;
                multisig_threshold = components.threshold;

                header.generic_params = aggchain_proof.aggchain_params.0;
                header.generic_vkey = vkey_to_bytes(aggchain_proof.aggchain_vkey);
            }
        }

        header.multisig_signatures_count = multisig_signatures_count.to_le_bytes();
        header.multisig_expected_signers_count = multisig_expected_signers_count.to_le_bytes();
        header.multisig_threshold = multisig_threshold.to_le_bytes();

        let header_bytes = bytemuck::bytes_of(&header).to_vec();

        let bridge_exits_bytes = wire_vec_bytes(self.bridge_exits.iter().map(BridgeExitWire::from));

        let mut imported_bridge_exits_bytes = Vec::with_capacity(
            self.imported_bridge_exits.len() * mem::size_of::<ImportedBridgeExitWire>(),
        );
        let mut nullifier_paths_bytes = Vec::with_capacity(
            self.imported_bridge_exits.len() * mem::size_of::<SmtNonInclusionProofWire>(),
        );
        for (imported, path) in &self.imported_bridge_exits {
            let wire = ImportedBridgeExitWire::try_from(imported)?;
            imported_bridge_exits_bytes.extend_from_slice(bytemuck::bytes_of(&wire));
            let wire_path = SmtNonInclusionProofWire::try_from(path)?;
            nullifier_paths_bytes.extend_from_slice(bytemuck::bytes_of(&wire_path));
        }

        let mut balances_proofs_bytes = Vec::with_capacity(
            self.balances_proofs.len() * mem::size_of::<BalanceProofEntryWire>(),
        );
        let mut balance_merkle_paths_bytes = Vec::with_capacity(
            self.balances_proofs.len() * mem::size_of::<BalanceMerkleProofWire>(),
        );
        for (token, (balance, path)) in &self.balances_proofs {
            let entry = BalanceProofEntryWire {
                token_info: (*token).into(),
                balance: (*balance).to_be_bytes::<32>(),
            };
            balances_proofs_bytes.extend_from_slice(bytemuck::bytes_of(&entry));
            let path_wire = BalanceMerkleProofWire::from(path);
            balance_merkle_paths_bytes.extend_from_slice(bytemuck::bytes_of(&path_wire));
        }

        Ok(ZeroCopyComponents {
            header_bytes,
            bridge_exits_bytes,
            imported_bridge_exits_bytes,
            nullifier_paths_bytes,
            balances_proofs_bytes,
            balance_merkle_paths_bytes,
            multisig_signatures_bytes,
            multisig_expected_signers_bytes,
        })
    }

    pub fn from_zero_copy_components<'a>(
        header_bytes: &'a [u8],
        bridge_exits_bytes: &'a [u8],
        imported_bridge_exits_bytes: &'a [u8],
        nullifier_paths_bytes: &'a [u8],
        balances_proofs_bytes: &'a [u8],
        balance_merkle_paths_bytes: &'a [u8],
        multisig_signatures_bytes: &'a [u8],
        multisig_expected_signers_bytes: &'a [u8],
    ) -> Result<MultiBatchHeaderRef<'a>, ZeroCopyError> {
        if header_bytes.len() != mem::size_of::<MultiBatchHeaderWire>() {
            return Err(ZeroCopyError::InvalidSize {
                expected: mem::size_of::<MultiBatchHeaderWire>(),
                actual: header_bytes.len(),
            });
        }

        let header = bytemuck::try_from_bytes::<MultiBatchHeaderWire>(header_bytes)
            .map_err(|_| ZeroCopyError::BytemuckCast)?;

        let bridge_exits_count = u32::from_le_bytes(header.bridge_exits_count) as usize;
        let imported_bridge_exits_count =
            u32::from_le_bytes(header.imported_bridge_exits_count) as usize;
        let balances_proofs_count = u32::from_le_bytes(header.balances_proofs_count) as usize;
        let multisig_signatures_count =
            u32::from_le_bytes(header.multisig_signatures_count) as usize;
        let multisig_expected_signers_count =
            u32::from_le_bytes(header.multisig_expected_signers_count) as usize;

        let bridge_exits_bytes = normalize_component_bytes(bridge_exits_bytes, bridge_exits_count)?;
        let bridge_exits = cast_slice::<BridgeExitWire>(bridge_exits_bytes)?;
        ensure_count("bridge_exits", bridge_exits_count, bridge_exits.len())?;

        let imported_bridge_exits_bytes =
            normalize_component_bytes(imported_bridge_exits_bytes, imported_bridge_exits_count)?;
        let imported_bridge_exits = cast_slice::<ImportedBridgeExitWire>(imported_bridge_exits_bytes)?;
        ensure_count(
            "imported_bridge_exits",
            imported_bridge_exits_count,
            imported_bridge_exits.len(),
        )?;

        let nullifier_paths_bytes =
            normalize_component_bytes(nullifier_paths_bytes, imported_bridge_exits_count)?;
        let nullifier_paths = cast_slice::<SmtNonInclusionProofWire>(nullifier_paths_bytes)?;
        ensure_count(
            "nullifier_paths",
            imported_bridge_exits_count,
            nullifier_paths.len(),
        )?;

        for path in nullifier_paths {
            if path.num_siblings > 64 {
                return Err(ZeroCopyError::InvalidSiblingsCount {
                    value: path.num_siblings,
                });
            }
        }

        let balances_proofs_bytes =
            normalize_component_bytes(balances_proofs_bytes, balances_proofs_count)?;
        let balances_proofs = cast_slice::<BalanceProofEntryWire>(balances_proofs_bytes)?;
        ensure_count(
            "balances_proofs",
            balances_proofs_count,
            balances_proofs.len(),
        )?;

        let balance_merkle_paths_bytes =
            normalize_component_bytes(balance_merkle_paths_bytes, balances_proofs_count)?;
        let balance_merkle_paths =
            cast_slice::<BalanceMerkleProofWire>(balance_merkle_paths_bytes)?;
        ensure_count(
            "balance_merkle_paths",
            balances_proofs_count,
            balance_merkle_paths.len(),
        )?;

        let multisig_signatures_bytes =
            normalize_component_bytes(multisig_signatures_bytes, multisig_signatures_count)?;
        let multisig_signatures =
            cast_slice::<MultisigSignatureEntryWire>(multisig_signatures_bytes)?;
        ensure_count(
            "multisig_signatures",
            multisig_signatures_count,
            multisig_signatures.len(),
        )?;

        let multisig_expected_signers_bytes = normalize_component_bytes(
            multisig_expected_signers_bytes,
            multisig_expected_signers_count,
        )?;
        let multisig_expected_signers =
            cast_slice::<[u8; 20]>(multisig_expected_signers_bytes)?;
        ensure_count(
            "multisig_expected_signers",
            multisig_expected_signers_count,
            multisig_expected_signers.len(),
        )?;

        for exit in bridge_exits {
            validate_leaf_type(exit.leaf_type)?;
        }
        for exit in imported_bridge_exits {
            validate_claim_type(exit.claim_type)?;
        }

        let aggchain_data = aggchain_data_from_wire(
            header,
            multisig_signatures,
            multisig_expected_signers,
        )?;

        let origin_network = NetworkId::new(u32::from_le_bytes(header.origin_network));
        let prev_pessimistic_root = Digest(header.prev_pessimistic_root);
        let l1_info_root = Digest(header.l1_info_root);
        let certificate_id = Digest(header.certificate_id);
        let height = u64::from_le_bytes(header.height);

        Ok(MultiBatchHeaderRef {
            origin_network,
            height,
            prev_pessimistic_root,
            l1_info_root,
            certificate_id,
            bridge_exits,
            imported_bridge_exits,
            nullifier_paths,
            balances_proofs,
            balance_merkle_paths,
            aggchain_data,
        })
    }

    pub fn to_zero_copy_packed_bytes(&self) -> Result<Vec<u8>, ZeroCopyError> {
        let components = self.to_zero_copy_components()?;
        let header_len = mem::size_of::<MultiBatchHeaderWire>();
        let offsets_len = mem::size_of::<MultiBatchHeaderOffsets>();
        let payload_start = header_len + offsets_len;

        let mut offsets = MultiBatchHeaderOffsets::zeroed();
        let mut cursor = 0usize;

        offsets.bridge_exits_offset = u32_to_le_bytes(cursor)?;
        cursor = cursor
            .checked_add(components.bridge_exits_bytes.len())
            .ok_or(ZeroCopyError::BytemuckCast)?;
        offsets.imported_bridge_exits_offset = u32_to_le_bytes(cursor)?;
        cursor = cursor
            .checked_add(components.imported_bridge_exits_bytes.len())
            .ok_or(ZeroCopyError::BytemuckCast)?;
        offsets.nullifier_paths_offset = u32_to_le_bytes(cursor)?;
        cursor = cursor
            .checked_add(components.nullifier_paths_bytes.len())
            .ok_or(ZeroCopyError::BytemuckCast)?;
        offsets.balances_proofs_offset = u32_to_le_bytes(cursor)?;
        cursor = cursor
            .checked_add(components.balances_proofs_bytes.len())
            .ok_or(ZeroCopyError::BytemuckCast)?;
        offsets.balance_merkle_paths_offset = u32_to_le_bytes(cursor)?;
        cursor = cursor
            .checked_add(components.balance_merkle_paths_bytes.len())
            .ok_or(ZeroCopyError::BytemuckCast)?;
        offsets.multisig_signatures_offset = u32_to_le_bytes(cursor)?;
        cursor = cursor
            .checked_add(components.multisig_signatures_bytes.len())
            .ok_or(ZeroCopyError::BytemuckCast)?;
        offsets.multisig_expected_signers_offset = u32_to_le_bytes(cursor)?;
        cursor = cursor
            .checked_add(components.multisig_expected_signers_bytes.len())
            .ok_or(ZeroCopyError::BytemuckCast)?;
        offsets.total_len = u32_to_le_bytes(cursor)?;

        let mut out = Vec::with_capacity(payload_start + cursor);
        out.extend_from_slice(&components.header_bytes);
        out.extend_from_slice(bytemuck::bytes_of(&offsets));
        out.extend_from_slice(&components.bridge_exits_bytes);
        out.extend_from_slice(&components.imported_bridge_exits_bytes);
        out.extend_from_slice(&components.nullifier_paths_bytes);
        out.extend_from_slice(&components.balances_proofs_bytes);
        out.extend_from_slice(&components.balance_merkle_paths_bytes);
        out.extend_from_slice(&components.multisig_signatures_bytes);
        out.extend_from_slice(&components.multisig_expected_signers_bytes);

        Ok(out)
    }

    pub fn from_zero_copy_packed_bytes<'a>(
        bytes: &'a [u8],
    ) -> Result<MultiBatchHeaderRef<'a>, ZeroCopyError> {
        let header_len = mem::size_of::<MultiBatchHeaderWire>();
        let offsets_len = mem::size_of::<MultiBatchHeaderOffsets>();
        let payload_start = header_len + offsets_len;

        if bytes.len() < payload_start {
            return Err(ZeroCopyError::InvalidSize {
                expected: payload_start,
                actual: bytes.len(),
            });
        }

        let header_bytes = &bytes[..header_len];
        let offsets_bytes = &bytes[header_len..payload_start];
        let offsets = bytemuck::try_from_bytes::<MultiBatchHeaderOffsets>(offsets_bytes)
            .map_err(|_| ZeroCopyError::BytemuckCast)?;

        let payload = &bytes[payload_start..];
        let total_len = u32::from_le_bytes(offsets.total_len) as usize;
        if total_len != payload.len() {
            return Err(ZeroCopyError::InvalidSize {
                expected: total_len,
                actual: payload.len(),
            });
        }

        let offsets_list = [
            ("bridge_exits_offset", offsets.bridge_exits_offset),
            ("imported_bridge_exits_offset", offsets.imported_bridge_exits_offset),
            ("nullifier_paths_offset", offsets.nullifier_paths_offset),
            ("balances_proofs_offset", offsets.balances_proofs_offset),
            ("balance_merkle_paths_offset", offsets.balance_merkle_paths_offset),
            ("multisig_signatures_offset", offsets.multisig_signatures_offset),
            (
                "multisig_expected_signers_offset",
                offsets.multisig_expected_signers_offset,
            ),
            ("total_len", offsets.total_len),
        ];

        let mut last = 0usize;
        let mut values = Vec::with_capacity(offsets_list.len());
        for (field, value) in offsets_list {
            let offset = u32::from_le_bytes(value) as usize;
            if offset < last || offset > total_len {
                return Err(ZeroCopyError::InvalidIndex {
                    field,
                    index: offset as u32,
                });
            }
            values.push(offset);
            last = offset;
        }

        let bridge_exits_bytes = &payload[values[0]..values[1]];
        let imported_bridge_exits_bytes = &payload[values[1]..values[2]];
        let nullifier_paths_bytes = &payload[values[2]..values[3]];
        let balances_proofs_bytes = &payload[values[3]..values[4]];
        let balance_merkle_paths_bytes = &payload[values[4]..values[5]];
        let multisig_signatures_bytes = &payload[values[5]..values[6]];
        let multisig_expected_signers_bytes = &payload[values[6]..values[7]];

        Self::from_zero_copy_components(
            header_bytes,
            bridge_exits_bytes,
            imported_bridge_exits_bytes,
            nullifier_paths_bytes,
            balances_proofs_bytes,
            balance_merkle_paths_bytes,
            multisig_signatures_bytes,
            multisig_expected_signers_bytes,
        )
    }
}

impl<'a> MultiBatchHeaderRef<'a> {
    pub fn to_owned(&self) -> Result<MultiBatchHeader, ZeroCopyError> {
        let bridge_exits: Vec<BridgeExit> = self
            .bridge_exits
            .iter()
            .map(BridgeExit::try_from)
            .collect::<Result<_, _>>()?;

        let mut imported_bridge_exits = Vec::with_capacity(self.imported_bridge_exits.len());
        for (wire_exit, wire_path) in self
            .imported_bridge_exits
            .iter()
            .zip(self.nullifier_paths.iter())
        {
            let exit = ImportedBridgeExit::try_from(wire_exit)?;
            let path = NullifierPath::from(wire_path);
            imported_bridge_exits.push((exit, path));
        }

        let mut balances_proofs = Vec::with_capacity(self.balances_proofs.len());
        for (entry, path) in self
            .balances_proofs
            .iter()
            .zip(self.balance_merkle_paths.iter())
        {
            let token = TokenInfo::from(&entry.token_info);
            let balance = U256::from_be_bytes(entry.balance);
            let merkle_path = LocalBalancePath::from(path);
            balances_proofs.push((token, (balance, merkle_path)));
        }

        Ok(MultiBatchHeader {
            origin_network: self.origin_network,
            height: self.height,
            prev_pessimistic_root: self.prev_pessimistic_root,
            bridge_exits,
            imported_bridge_exits,
            l1_info_root: self.l1_info_root,
            balances_proofs,
            aggchain_data: self.aggchain_data.clone(),
            certificate_id: self.certificate_id,
        })
    }
}

fn u32_to_le_bytes(value: usize) -> Result<[u8; 4], ZeroCopyError> {
    let value = u32::try_from(value).map_err(|_| ZeroCopyError::BytemuckCast)?;
    Ok(value.to_le_bytes())
}

fn validate_leaf_type(value: u8) -> Result<(), ZeroCopyError> {
    match value {
        0 | 1 => Ok(()),
        _ => Err(ZeroCopyError::InvalidDiscriminant {
            field: "leaf_type",
            value,
        }),
    }
}

fn validate_claim_type(value: u8) -> Result<(), ZeroCopyError> {
    match value {
        CLAIM_TYPE_MAINNET | CLAIM_TYPE_ROLLUP => Ok(()),
        _ => Err(ZeroCopyError::InvalidDiscriminant {
            field: "claim_type",
            value,
        }),
    }
}

fn ensure_count(field: &'static str, expected: usize, actual: usize) -> Result<(), ZeroCopyError> {
    if expected != actual {
        return Err(ZeroCopyError::CountMismatch {
            field,
            header: expected as u32,
            actual,
        });
    }
    Ok(())
}

fn normalize_component_bytes<'a>(
    bytes: &'a [u8],
    expected_count: usize,
) -> Result<&'a [u8], ZeroCopyError> {
    if expected_count == 0 {
        if bytes.is_empty() {
            return Ok(bytes);
        }
        // SP1 read_vec panics on empty input, so allow a 1-byte sentinel.
        if bytes.len() == 1 && bytes[0] == 0 {
            return Ok(&[]);
        }
        return Err(ZeroCopyError::InvalidSize {
            expected: 0,
            actual: bytes.len(),
        });
    }

    Ok(bytes)
}

fn cast_slice<T: Pod>(bytes: &[u8]) -> Result<&[T], ZeroCopyError> {
    bytemuck::try_cast_slice(bytes).map_err(|_| ZeroCopyError::BytemuckCast)
}

fn wire_vec_bytes<I, T>(iter: I) -> Vec<u8>
where
    I: IntoIterator<Item = T>,
    T: Pod,
{
    let mut bytes = Vec::new();
    for item in iter {
        bytes.extend_from_slice(bytemuck::bytes_of(&item));
    }
    bytes
}

impl From<&BridgeExit> for BridgeExitWire {
    fn from(exit: &BridgeExit) -> Self {
        let (metadata_hash, has_metadata) = match exit.metadata {
            Some(digest) => (digest.0, 1u8),
            None => ([0u8; 32], 0u8),
        };

        Self {
            origin_network: exit.token_info.origin_network.to_le_bytes(),
            dest_network: exit.dest_network.to_le_bytes(),
            origin_token_address: exit.token_info.origin_token_address.into_array(),
            dest_address: exit.dest_address.into_array(),
            amount: exit.amount.to_be_bytes::<32>(),
            metadata_hash,
            leaf_type: exit.leaf_type as u8,
            has_metadata,
            _padding: [0; 2],
        }
    }
}

impl TryFrom<&BridgeExitWire> for BridgeExit {
    type Error = ZeroCopyError;

    fn try_from(wire: &BridgeExitWire) -> Result<Self, Self::Error> {
        validate_leaf_type(wire.leaf_type)?;
        let leaf_type = if wire.leaf_type == 0 {
            LeafType::Transfer
        } else {
            LeafType::Message
        };

        let metadata = if wire.has_metadata == 0 {
            None
        } else {
            Some(Digest(wire.metadata_hash))
        };

        Ok(BridgeExit {
            leaf_type,
            token_info: TokenInfo::from(&wire.token_info()),
            dest_network: NetworkId::new(u32::from_le_bytes(wire.dest_network)),
            dest_address: Address::from(wire.dest_address),
            amount: U256::from_be_bytes(wire.amount),
            metadata,
        })
    }
}

impl BridgeExitWire {
    fn token_info(&self) -> TokenInfoWire {
        TokenInfoWire {
            origin_network: self.origin_network,
            origin_token_address: self.origin_token_address,
        }
    }
}

impl From<TokenInfo> for TokenInfoWire {
    fn from(info: TokenInfo) -> Self {
        Self {
            origin_network: info.origin_network.to_le_bytes(),
            origin_token_address: info.origin_token_address.into_array(),
        }
    }
}

impl From<&TokenInfoWire> for TokenInfo {
    fn from(wire: &TokenInfoWire) -> Self {
        TokenInfo {
            origin_network: NetworkId::new(u32::from_le_bytes(wire.origin_network)),
            origin_token_address: Address::from(wire.origin_token_address),
        }
    }
}

impl From<&MerkleProof> for MerkleProofWire {
    fn from(proof: &MerkleProof) -> Self {
        Self {
            siblings: proof.proof.siblings.map(|d| d.0),
            root: proof.root.0,
        }
    }
}

impl From<&MerkleProofWire> for MerkleProof {
    fn from(wire: &MerkleProofWire) -> Self {
        MerkleProof {
            proof: LETMerkleProof {
                siblings: wire.siblings.map(Digest),
            },
            root: Digest(wire.root),
        }
    }
}

impl From<&L1InfoTreeLeaf> for L1InfoTreeLeafWire {
    fn from(leaf: &L1InfoTreeLeaf) -> Self {
        Self {
            l1_info_tree_index: leaf.l1_info_tree_index.to_le_bytes(),
            rer: leaf.rer.0,
            mer: leaf.mer.0,
            global_exit_root: leaf.inner.global_exit_root.0,
            block_hash: leaf.inner.block_hash.0,
            timestamp: leaf.inner.timestamp.to_le_bytes(),
        }
    }
}

impl From<&L1InfoTreeLeafWire> for L1InfoTreeLeaf {
    fn from(wire: &L1InfoTreeLeafWire) -> Self {
        L1InfoTreeLeaf {
            l1_info_tree_index: u32::from_le_bytes(wire.l1_info_tree_index),
            rer: Digest(wire.rer),
            mer: Digest(wire.mer),
            inner: L1InfoTreeLeafInner {
                global_exit_root: Digest(wire.global_exit_root),
                block_hash: Digest(wire.block_hash),
                timestamp: u64::from_le_bytes(wire.timestamp),
            },
        }
    }
}

impl From<&ClaimFromMainnet> for ClaimFromMainnetWire {
    fn from(claim: &ClaimFromMainnet) -> Self {
        Self {
            proof_leaf_mer: MerkleProofWire::from(&claim.proof_leaf_mer),
            proof_ger_l1root: MerkleProofWire::from(&claim.proof_ger_l1root),
            l1_leaf: L1InfoTreeLeafWire::from(&claim.l1_leaf),
        }
    }
}

impl From<&ClaimFromMainnetWire> for ClaimFromMainnet {
    fn from(wire: &ClaimFromMainnetWire) -> Self {
        ClaimFromMainnet {
            proof_leaf_mer: MerkleProof::from(&wire.proof_leaf_mer),
            proof_ger_l1root: MerkleProof::from(&wire.proof_ger_l1root),
            l1_leaf: L1InfoTreeLeaf::from(&wire.l1_leaf),
        }
    }
}

impl From<&ClaimFromRollup> for ClaimFromRollupWire {
    fn from(claim: &ClaimFromRollup) -> Self {
        Self {
            proof_leaf_ler: MerkleProofWire::from(&claim.proof_leaf_ler),
            proof_ler_rer: MerkleProofWire::from(&claim.proof_ler_rer),
            proof_ger_l1root: MerkleProofWire::from(&claim.proof_ger_l1root),
            l1_leaf: L1InfoTreeLeafWire::from(&claim.l1_leaf),
        }
    }
}

impl From<&ClaimFromRollupWire> for ClaimFromRollup {
    fn from(wire: &ClaimFromRollupWire) -> Self {
        ClaimFromRollup {
            proof_leaf_ler: MerkleProof::from(&wire.proof_leaf_ler),
            proof_ler_rer: MerkleProof::from(&wire.proof_ler_rer),
            proof_ger_l1root: MerkleProof::from(&wire.proof_ger_l1root),
            l1_leaf: L1InfoTreeLeaf::from(&wire.l1_leaf),
        }
    }
}

impl TryFrom<&ImportedBridgeExit> for ImportedBridgeExitWire {
    type Error = ZeroCopyError;

    fn try_from(exit: &ImportedBridgeExit) -> Result<Self, Self::Error> {
        let (claim_type, claim_data) = claim_to_bytes(&exit.claim_data)?;
        let rollup_index = exit.global_index.rollup_index().map(u32::from).unwrap_or(0);

        Ok(Self {
            global_index_leaf: exit.global_index.leaf_index().to_le_bytes(),
            global_index_rollup: rollup_index.to_le_bytes(),
            bridge_exit: BridgeExitWire::from(&exit.bridge_exit),
            claim_type,
            claim_data,
            _padding: [0; 7],
        })
    }
}

impl TryFrom<&ImportedBridgeExitWire> for ImportedBridgeExit {
    type Error = ZeroCopyError;

    fn try_from(wire: &ImportedBridgeExitWire) -> Result<Self, Self::Error> {
        validate_claim_type(wire.claim_type)?;

        let leaf_index = u32::from_le_bytes(wire.global_index_leaf);
        let rollup_index = u32::from_le_bytes(wire.global_index_rollup);

        let global_index = match wire.claim_type {
            CLAIM_TYPE_MAINNET => {
                if rollup_index != 0 {
                    return Err(ZeroCopyError::InvalidIndex {
                        field: "global_index_rollup",
                        index: rollup_index,
                    });
                }
                GlobalIndex::new(NetworkId::ETH_L1, leaf_index)
            }
            CLAIM_TYPE_ROLLUP => GlobalIndex::new(NetworkId::new(rollup_index), leaf_index),
            _ => {
                return Err(ZeroCopyError::InvalidDiscriminant {
                    field: "claim_type",
                    value: wire.claim_type,
                })
            }
        };

        let claim_data = claim_from_bytes(wire.claim_type, &wire.claim_data)?;
        let bridge_exit = BridgeExit::try_from(&wire.bridge_exit)?;

        Ok(ImportedBridgeExit {
            bridge_exit,
            claim_data,
            global_index,
        })
    }
}

impl TryFrom<&NullifierPath> for SmtNonInclusionProofWire {
    type Error = ZeroCopyError;

    fn try_from(path: &NullifierPath) -> Result<Self, Self::Error> {
        if path.siblings.len() > 64 {
            return Err(ZeroCopyError::InvalidSiblingsCount {
                value: path.siblings.len() as u8,
            });
        }

        let mut siblings = [[0u8; 32]; 64];
        for (idx, digest) in path.siblings.iter().enumerate() {
            siblings[idx] = digest.0;
        }

        Ok(Self {
            num_siblings: path.siblings.len() as u8,
            _padding: [0; 3],
            siblings,
        })
    }
}

impl From<&SmtNonInclusionProofWire> for NullifierPath {
    fn from(wire: &SmtNonInclusionProofWire) -> Self {
        let count = wire.num_siblings as usize;
        let siblings = wire.siblings[..count]
            .iter()
            .map(|bytes| Digest(*bytes))
            .collect();

        NullifierPath { siblings }
    }
}

impl From<&LocalBalancePath> for BalanceMerkleProofWire {
    fn from(path: &LocalBalancePath) -> Self {
        Self {
            siblings: Hash32x192(path.siblings.map(|d| d.0)),
        }
    }
}

impl From<&BalanceMerkleProofWire> for LocalBalancePath {
    fn from(wire: &BalanceMerkleProofWire) -> Self {
        LocalBalancePath {
            siblings: wire.siblings.0.map(Digest),
        }
    }
}

fn claim_to_bytes(claim: &Claim) -> Result<(u8, ClaimDataBytes), ZeroCopyError> {
    let mut claim_data = ClaimDataBytes([0u8; 3584]);
    match claim {
        Claim::Mainnet(mainnet) => {
            let wire = ClaimFromMainnetWire::from(mainnet.as_ref());
            let bytes = bytemuck::bytes_of(&wire);
            claim_data.0[..bytes.len()].copy_from_slice(bytes);
            Ok((CLAIM_TYPE_MAINNET, claim_data))
        }
        Claim::Rollup(rollup) => {
            let wire = ClaimFromRollupWire::from(rollup.as_ref());
            let bytes = bytemuck::bytes_of(&wire);
            claim_data.0[..bytes.len()].copy_from_slice(bytes);
            Ok((CLAIM_TYPE_ROLLUP, claim_data))
        }
    }
}

fn claim_from_bytes(claim_type: u8, data: &ClaimDataBytes) -> Result<Claim, ZeroCopyError> {
    match claim_type {
        CLAIM_TYPE_MAINNET => {
            let size = mem::size_of::<ClaimFromMainnetWire>();
            let wire = bytemuck::try_from_bytes::<ClaimFromMainnetWire>(&data.0[..size])
                .map_err(|_| ZeroCopyError::BytemuckCast)?;
            Ok(Claim::Mainnet(Box::new(ClaimFromMainnet::from(wire))))
        }
        CLAIM_TYPE_ROLLUP => {
            let size = mem::size_of::<ClaimFromRollupWire>();
            let wire = bytemuck::try_from_bytes::<ClaimFromRollupWire>(&data.0[..size])
                .map_err(|_| ZeroCopyError::BytemuckCast)?;
            Ok(Claim::Rollup(Box::new(ClaimFromRollup::from(wire))))
        }
        _ => Err(ZeroCopyError::InvalidDiscriminant {
            field: "claim_type",
            value: claim_type,
        }),
    }
}

struct MultisigComponents {
    signatures_bytes: Vec<u8>,
    expected_signers_bytes: Vec<u8>,
    signatures_count: u32,
    expected_signers_count: u32,
    threshold: u64,
}

fn multisig_to_components(multisig: &MultiSignature) -> Result<MultisigComponents, ZeroCopyError> {
    let mut signatures = Vec::new();
    for (idx, signature) in multisig.signatures.iter().enumerate() {
        let Some(signature) = signature else {
            continue;
        };
        let entry = MultisigSignatureEntryWire {
            signer_index: (idx as u32).to_le_bytes(),
            signature: Bytes65(signature.as_bytes()),
            _padding: [0; 3],
        };
        signatures.extend_from_slice(bytemuck::bytes_of(&entry));
    }

    let mut expected_signers_bytes = Vec::with_capacity(multisig.expected_signers.len() * 20);
    for signer in &multisig.expected_signers {
        expected_signers_bytes.extend_from_slice(&signer.into_array());
    }

    let signatures_count = u32::try_from(signatures.len() / mem::size_of::<MultisigSignatureEntryWire>())
        .map_err(|_| ZeroCopyError::BytemuckCast)?;
    let expected_signers_count = u32::try_from(multisig.expected_signers.len())
        .map_err(|_| ZeroCopyError::BytemuckCast)?;

    Ok(MultisigComponents {
        signatures_bytes: signatures,
        expected_signers_bytes,
        signatures_count,
        expected_signers_count,
        threshold: multisig.threshold as u64,
    })
}

fn aggchain_data_from_wire(
    header: &MultiBatchHeaderWire,
    signatures: &[MultisigSignatureEntryWire],
    expected_signers: &[[u8; 20]],
) -> Result<AggchainData, ZeroCopyError> {
    let aggchain_type = header.aggchain_proof_type;
    if aggchain_type != AGGCHAIN_PROOF_TYPE_MULTISIG_ONLY
        && aggchain_type != AGGCHAIN_PROOF_TYPE_MULTISIG_AND_AGGCHAIN
    {
        if !signatures.is_empty() || !expected_signers.is_empty() {
            return Err(ZeroCopyError::CountMismatch {
                field: "multisig_components",
                header: 0,
                actual: signatures.len(),
            });
        }
    }

    match aggchain_type {
        AGGCHAIN_PROOF_TYPE_LEGACY_ECDSA => {
            let signature = Signature::try_from(header.ecdsa_signature.0.as_slice())
                .map_err(|_| ZeroCopyError::BytemuckCast)?;
            let signer = Address::from(header.ecdsa_signer);
            Ok(AggchainData::LegacyEcdsa { signer, signature })
        }
        AGGCHAIN_PROOF_TYPE_MULTISIG_ONLY => {
            let multisig = multisig_from_wire(header, signatures, expected_signers)?;
            Ok(AggchainData::MultisigOnly(multisig))
        }
        AGGCHAIN_PROOF_TYPE_MULTISIG_AND_AGGCHAIN => {
            let multisig = multisig_from_wire(header, signatures, expected_signers)?;
            let aggchain_proof = AggchainProof {
                aggchain_params: Digest(header.generic_params),
                aggchain_vkey: bytes_to_vkey(header.generic_vkey),
            };
            Ok(AggchainData::MultisigAndAggchainProof {
                multisig,
                aggchain_proof,
            })
        }
        _ => Err(ZeroCopyError::InvalidDiscriminant {
            field: "aggchain_proof_type",
            value: aggchain_type,
        }),
    }
}

fn multisig_from_wire(
    header: &MultiBatchHeaderWire,
    signatures: &[MultisigSignatureEntryWire],
    expected_signers: &[[u8; 20]],
) -> Result<MultiSignature, ZeroCopyError> {
    let threshold = u64::from_le_bytes(header.multisig_threshold);
    let threshold = usize::try_from(threshold).map_err(|_| ZeroCopyError::BytemuckCast)?;

    let mut signatures_vec = vec![None; expected_signers.len()];
    for entry in signatures {
        let idx = u32::from_le_bytes(entry.signer_index);
        let idx_usize = usize::try_from(idx).map_err(|_| ZeroCopyError::BytemuckCast)?;
        if idx_usize >= signatures_vec.len() {
            return Err(ZeroCopyError::InvalidIndex {
                field: "multisig_signer_index",
                index: idx,
            });
        }
        if signatures_vec[idx_usize].is_some() {
            return Err(ZeroCopyError::InvalidIndex {
                field: "multisig_signer_index",
                index: idx,
            });
        }
        let signature = Signature::try_from(entry.signature.0.as_slice())
            .map_err(|_| ZeroCopyError::BytemuckCast)?;
        signatures_vec[idx_usize] = Some(signature);
    }

    let expected_signers: Vec<Address> = expected_signers
        .iter()
        .map(|bytes| Address::from(*bytes))
        .collect();

    Ok(MultiSignature {
        signatures: signatures_vec,
        expected_signers,
        threshold,
    })
}

fn vkey_to_bytes(vkey: Vkey) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (idx, word) in vkey.iter().enumerate() {
        let bytes = word.to_le_bytes();
        let start = idx * 4;
        out[start..start + 4].copy_from_slice(&bytes);
    }
    out
}

fn bytes_to_vkey(bytes: [u8; 32]) -> Vkey {
    let mut out = [0u32; 8];
    for i in 0..8 {
        let start = i * 4;
        out[i] = u32::from_le_bytes(bytes[start..start + 4].try_into().unwrap());
    }
    out
}

#[cfg(test)]
mod tests {
    use agglayer_primitives::U256;
    use unified_bridge::{BridgeExit, NetworkId, TokenInfo};

    use super::*;

    fn sample_header() -> MultiBatchHeader {
        MultiBatchHeader {
            origin_network: NetworkId::new(1),
            height: 42,
            prev_pessimistic_root: Digest([1u8; 32]),
            bridge_exits: vec![BridgeExit {
                leaf_type: LeafType::Transfer,
                token_info: TokenInfo {
                    origin_network: NetworkId::new(1),
                    origin_token_address: Address::new([2u8; 20]),
                },
                dest_network: NetworkId::new(2),
                dest_address: Address::new([3u8; 20]),
                amount: U256::from_be_bytes([4u8; 32]),
                metadata: None,
            }],
            imported_bridge_exits: Vec::new(),
            l1_info_root: Digest([5u8; 32]),
            balances_proofs: Vec::new(),
            aggchain_data: AggchainData::LegacyEcdsa {
                signer: Address::new([6u8; 20]),
                signature: {
                    let mut sig_bytes = [7u8; 65];
                    sig_bytes[64] = 27;
                    Signature::try_from(sig_bytes.as_slice()).unwrap()
                },
            },
            certificate_id: Digest([8u8; 32]),
        }
    }

    #[test]
    fn roundtrip_minimal() {
        let header = sample_header();
        let components = header.to_zero_copy_components().unwrap();
        let ref_header = MultiBatchHeader::from_zero_copy_components(
            &components.header_bytes,
            &components.bridge_exits_bytes,
            &components.imported_bridge_exits_bytes,
            &components.nullifier_paths_bytes,
            &components.balances_proofs_bytes,
            &components.balance_merkle_paths_bytes,
            &components.multisig_signatures_bytes,
            &components.multisig_expected_signers_bytes,
        )
        .unwrap();
        let owned = ref_header.to_owned().unwrap();

        assert_eq!(owned.origin_network, header.origin_network);
        assert_eq!(owned.height, header.height);
        assert_eq!(owned.prev_pessimistic_root, header.prev_pessimistic_root);
        assert_eq!(owned.l1_info_root, header.l1_info_root);
        assert_eq!(owned.certificate_id, header.certificate_id);
        assert_eq!(owned.bridge_exits.len(), 1);
        assert_eq!(owned.bridge_exits[0].dest_network, NetworkId::new(2));
    }

    #[test]
    fn roundtrip_packed() {
        let header = sample_header();
        let packed = header.to_zero_copy_packed_bytes().unwrap();
        let ref_header = MultiBatchHeader::from_zero_copy_packed_bytes(&packed).unwrap();
        let owned = ref_header.to_owned().unwrap();

        assert_eq!(owned.origin_network, header.origin_network);
        assert_eq!(owned.height, header.height);
        assert_eq!(owned.prev_pessimistic_root, header.prev_pessimistic_root);
        assert_eq!(owned.l1_info_root, header.l1_info_root);
        assert_eq!(owned.certificate_id, header.certificate_id);
        assert_eq!(owned.bridge_exits.len(), 1);
        assert_eq!(owned.bridge_exits[0].dest_network, NetworkId::new(2));
    }

    #[test]
    fn invalid_leaf_type() {
        let header = sample_header();
        let mut components = header.to_zero_copy_components().unwrap();
        let mut wire = bytemuck::pod_read_unaligned::<BridgeExitWire>(
            &components.bridge_exits_bytes[..mem::size_of::<BridgeExitWire>()],
        );
        wire.leaf_type = 9;
        components.bridge_exits_bytes[..mem::size_of::<BridgeExitWire>()]
            .copy_from_slice(bytemuck::bytes_of(&wire));

        let err = MultiBatchHeader::from_zero_copy_components(
            &components.header_bytes,
            &components.bridge_exits_bytes,
            &components.imported_bridge_exits_bytes,
            &components.nullifier_paths_bytes,
            &components.balances_proofs_bytes,
            &components.balance_merkle_paths_bytes,
            &components.multisig_signatures_bytes,
            &components.multisig_expected_signers_bytes,
        )
        .unwrap_err();

        assert!(matches!(err, ZeroCopyError::InvalidDiscriminant { .. }));
    }
}
