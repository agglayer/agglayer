//! Definitions of the certificate storage format with backwards compatibility.
//!
//! Currently, we have two versions of certificate storage format. The first
//! byte determines the storage format version.
//!
//! In version 0, where backwards compatibility is required, the first byte
//! happens to be the highest byte of network ID. This effectively limits the
//! range of network IDs in v0 storage to [0, 2^24-1]. The current network IDs
//! fall into this range, so the highest byte (with value 0) also acts as the
//! version tag.
//!
//! In subsequent versions, we just have the version byte followed by a
//! straightforward encoding of the certificate, restoring the full range of
//! network IDs.
//!
//! In the unlikely scenario where it turns out we need more than 256 storage
//! format versions, another byte can be allocated to specify a "sub-version" in
//! one of the future versions.

use std::borrow::Cow;

use agglayer_sp1::ProofExt as _;
use agglayer_interop_types_v13 as legacy_interop_types;
use agglayer_tries::roots::LocalExitRoot;
use agglayer_types::{
    aggchain_proof::{AggchainData, AggchainProof, MultisigPayload, Proof, SP1StarkWithContext},
    primitives::{Digest, SignatureError},
    Address, Certificate, Height, Metadata, NetworkId, Signature, U256,
};
use pessimistic_proof::unified_bridge::{
    AggchainProofPublicValues, BridgeExit, Claim, ClaimFromMainnet, ClaimFromRollup, GlobalIndex,
    ImportedBridgeExit, L1InfoTreeLeaf, L1InfoTreeLeafInner, LeafType, MerkleProof, TokenInfo,
};
use prost::Message as _;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    schema::{bincode_codec, CodecError},
    types::{generated::agglayer::storage::v0 as proto, proof::ProofConversionError},
};

#[derive(Debug, thiserror::Error)]
pub enum CertificateConversionError {
    #[error("missing field `{0}`")]
    MissingField(&'static str),

    #[error("invalid data for `{field}`: {reason}")]
    InvalidData { field: &'static str, reason: String },

    #[error("invalid signature in `{field}`: {source}")]
    Signature {
        field: &'static str,
        #[source]
        source: SignatureError,
    },

    #[error("{0}")]
    Proof(#[from] ProofConversionError),
}

macro_rules! required_field {
    ($from:expr, $field:ident) => {
        $from
            .$field
            .ok_or(CertificateConversionError::MissingField(stringify!($field)))
    };
    ($value:expr, $field:expr $(,)?) => {
        $value.ok_or(CertificateConversionError::MissingField($field))
    };
}

fn expect_bytes<const N: usize>(
    bytes: &[u8],
    field: &'static str,
) -> Result<[u8; N], CertificateConversionError> {
    bytes
        .try_into()
        .map_err(|_| CertificateConversionError::InvalidData {
            field,
            reason: format!("expected {N} bytes, got {}", bytes.len()),
        })
}

fn parse_signature(
    value: proto::FixedBytes65,
    field: &'static str,
) -> Result<Signature, CertificateConversionError> {
    Signature::try_from(value.value.as_ref())
        .map_err(|source| CertificateConversionError::Signature { field, source })
}

impl From<Signature> for proto::FixedBytes65 {
    fn from(value: Signature) -> Self {
        Self {
            value: value.as_bytes().to_vec().into(),
        }
    }
}

impl TryFrom<proto::FixedBytes20> for Address {
    type Error = CertificateConversionError;

    fn try_from(value: proto::FixedBytes20) -> Result<Self, Self::Error> {
        Ok(Self::from(expect_bytes::<20>(
            value.value.as_ref(),
            "fixed_bytes20",
        )?))
    }
}

impl From<Address> for proto::FixedBytes20 {
    fn from(value: Address) -> Self {
        Self {
            value: value.as_slice().to_vec().into(),
        }
    }
}

impl TryFrom<proto::FixedBytes32> for Digest {
    type Error = CertificateConversionError;

    fn try_from(value: proto::FixedBytes32) -> Result<Self, Self::Error> {
        Ok(Self::from(expect_bytes::<32>(
            value.value.as_ref(),
            "fixed_bytes32",
        )?))
    }
}

impl From<Digest> for proto::FixedBytes32 {
    fn from(value: Digest) -> Self {
        Self {
            value: value.0.to_vec().into(),
        }
    }
}

impl TryFrom<proto::FixedBytes32> for LocalExitRoot {
    type Error = CertificateConversionError;

    fn try_from(value: proto::FixedBytes32) -> Result<Self, Self::Error> {
        Ok(Self::from(Digest::try_from(value)?))
    }
}

impl From<LocalExitRoot> for proto::FixedBytes32 {
    fn from(value: LocalExitRoot) -> Self {
        Digest::from(value).into()
    }
}

impl TryFrom<proto::FixedBytes32> for Metadata {
    type Error = CertificateConversionError;

    fn try_from(value: proto::FixedBytes32) -> Result<Self, Self::Error> {
        Ok(Self::new(Digest::try_from(value)?))
    }
}

impl From<Metadata> for proto::FixedBytes32 {
    fn from(value: Metadata) -> Self {
        (*value).into()
    }
}

impl TryFrom<proto::FixedBytes32> for U256 {
    type Error = CertificateConversionError;

    fn try_from(value: proto::FixedBytes32) -> Result<Self, Self::Error> {
        Ok(Self::from_be_bytes(expect_bytes::<32>(
            value.value.as_ref(),
            "fixed_bytes32",
        )?))
    }
}

impl From<U256> for proto::FixedBytes32 {
    fn from(value: U256) -> Self {
        Self {
            value: value.to_be_bytes::<32>().to_vec().into(),
        }
    }
}

impl TryFrom<proto::TokenInfo> for TokenInfo {
    type Error = CertificateConversionError;

    fn try_from(value: proto::TokenInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            origin_network: NetworkId::new(value.origin_network),
            origin_token_address: required_field!(value, origin_token_address)?.try_into()?,
        })
    }
}

impl From<TokenInfo> for proto::TokenInfo {
    fn from(value: TokenInfo) -> Self {
        Self {
            origin_network: value.origin_network.to_u32(),
            origin_token_address: Some(value.origin_token_address.into()),
        }
    }
}

impl TryFrom<proto::BridgeExit> for BridgeExit {
    type Error = CertificateConversionError;

    fn try_from(value: proto::BridgeExit) -> Result<Self, Self::Error> {
        Ok(Self {
            leaf_type: match value.leaf_type {
                x if x == proto::LeafType::Transfer as i32 => LeafType::Transfer,
                x if x == proto::LeafType::Message as i32 => LeafType::Message,
                other => {
                    return Err(CertificateConversionError::InvalidData {
                        field: "leaf_type",
                        reason: format!("unsupported leaf type `{other}`"),
                    });
                }
            },
            token_info: required_field!(value, token_info)?.try_into()?,
            dest_network: NetworkId::new(value.dest_network),
            dest_address: required_field!(value, dest_address)?.try_into()?,
            amount: required_field!(value, amount)?.try_into()?,
            metadata: value.metadata.map(TryInto::try_into).transpose()?,
        })
    }
}

impl From<BridgeExit> for proto::BridgeExit {
    fn from(value: BridgeExit) -> Self {
        Self {
            leaf_type: match value.leaf_type {
                LeafType::Transfer => proto::LeafType::Transfer as i32,
                LeafType::Message => proto::LeafType::Message as i32,
            },
            token_info: Some(value.token_info.into()),
            dest_network: value.dest_network.to_u32(),
            dest_address: Some(value.dest_address.into()),
            amount: Some(value.amount.into()),
            metadata: value.metadata.map(Into::into),
        }
    }
}

impl TryFrom<proto::MerkleProof> for MerkleProof {
    type Error = CertificateConversionError;

    fn try_from(value: proto::MerkleProof) -> Result<Self, Self::Error> {
        if value.siblings.len() != 32 {
            return Err(CertificateConversionError::InvalidData {
                field: "siblings",
                reason: format!("expected 32 siblings, got {}", value.siblings.len()),
            });
        }

        let siblings: Vec<Digest> = value
            .siblings
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()?;

        Ok(Self::new(
            required_field!(value, root)?.try_into()?,
            siblings.try_into().unwrap(),
        ))
    }
}

impl From<MerkleProof> for proto::MerkleProof {
    fn from(value: MerkleProof) -> Self {
        Self {
            root: Some(value.root.into()),
            siblings: value.siblings().iter().copied().map(Into::into).collect(),
        }
    }
}

impl TryFrom<proto::L1InfoTreeLeaf> for L1InfoTreeLeafInner {
    type Error = CertificateConversionError;

    fn try_from(value: proto::L1InfoTreeLeaf) -> Result<Self, Self::Error> {
        Ok(Self {
            global_exit_root: required_field!(value, global_exit_root)?.try_into()?,
            block_hash: required_field!(value, block_hash)?.try_into()?,
            timestamp: value.timestamp,
        })
    }
}

impl From<L1InfoTreeLeafInner> for proto::L1InfoTreeLeaf {
    fn from(value: L1InfoTreeLeafInner) -> Self {
        Self {
            global_exit_root: Some(value.global_exit_root.into()),
            block_hash: Some(value.block_hash.into()),
            timestamp: value.timestamp,
        }
    }
}

impl TryFrom<proto::L1InfoTreeLeafWithContext> for L1InfoTreeLeaf {
    type Error = CertificateConversionError;

    fn try_from(value: proto::L1InfoTreeLeafWithContext) -> Result<Self, Self::Error> {
        Ok(Self {
            l1_info_tree_index: value.l1_info_tree_index,
            rer: required_field!(value, rer)?.try_into()?,
            mer: required_field!(value, mer)?.try_into()?,
            inner: required_field!(value, inner)?.try_into()?,
        })
    }
}

impl From<L1InfoTreeLeaf> for proto::L1InfoTreeLeafWithContext {
    fn from(value: L1InfoTreeLeaf) -> Self {
        Self {
            l1_info_tree_index: value.l1_info_tree_index,
            rer: Some(value.rer.into()),
            mer: Some(value.mer.into()),
            inner: Some(value.inner.into()),
        }
    }
}

impl TryFrom<proto::ClaimFromMainnet> for ClaimFromMainnet {
    type Error = CertificateConversionError;

    fn try_from(value: proto::ClaimFromMainnet) -> Result<Self, Self::Error> {
        Ok(Self {
            proof_leaf_mer: required_field!(value, proof_leaf_mer)?.try_into()?,
            proof_ger_l1root: required_field!(value, proof_ger_l1root)?.try_into()?,
            l1_leaf: required_field!(value, l1_leaf)?.try_into()?,
        })
    }
}

impl From<ClaimFromMainnet> for proto::ClaimFromMainnet {
    fn from(value: ClaimFromMainnet) -> Self {
        Self {
            proof_leaf_mer: Some(value.proof_leaf_mer.into()),
            proof_ger_l1root: Some(value.proof_ger_l1root.into()),
            l1_leaf: Some(value.l1_leaf.into()),
        }
    }
}

impl TryFrom<proto::ClaimFromRollup> for ClaimFromRollup {
    type Error = CertificateConversionError;

    fn try_from(value: proto::ClaimFromRollup) -> Result<Self, Self::Error> {
        Ok(Self {
            proof_leaf_ler: required_field!(value, proof_leaf_ler)?.try_into()?,
            proof_ler_rer: required_field!(value, proof_ler_rer)?.try_into()?,
            proof_ger_l1root: required_field!(value, proof_ger_l1root)?.try_into()?,
            l1_leaf: required_field!(value, l1_leaf)?.try_into()?,
        })
    }
}

impl From<ClaimFromRollup> for proto::ClaimFromRollup {
    fn from(value: ClaimFromRollup) -> Self {
        Self {
            proof_leaf_ler: Some(value.proof_leaf_ler.into()),
            proof_ler_rer: Some(value.proof_ler_rer.into()),
            proof_ger_l1root: Some(value.proof_ger_l1root.into()),
            l1_leaf: Some(value.l1_leaf.into()),
        }
    }
}

impl TryFrom<proto::imported_bridge_exit::Claim> for Claim {
    type Error = CertificateConversionError;

    fn try_from(value: proto::imported_bridge_exit::Claim) -> Result<Self, Self::Error> {
        Ok(match value {
            proto::imported_bridge_exit::Claim::Mainnet(value) => {
                Claim::Mainnet(Box::new(value.try_into()?))
            }
            proto::imported_bridge_exit::Claim::Rollup(value) => {
                Claim::Rollup(Box::new(value.try_into()?))
            }
        })
    }
}

impl From<Claim> for proto::imported_bridge_exit::Claim {
    fn from(value: Claim) -> Self {
        match value {
            Claim::Mainnet(value) => Self::Mainnet((*value).into()),
            Claim::Rollup(value) => Self::Rollup((*value).into()),
        }
    }
}

impl TryFrom<proto::FixedBytes32> for GlobalIndex {
    type Error = CertificateConversionError;

    fn try_from(value: proto::FixedBytes32) -> Result<Self, Self::Error> {
        GlobalIndex::from_u256(value.try_into()?).map_err(|err| {
            CertificateConversionError::InvalidData {
                field: "global_index",
                reason: err.to_string(),
            }
        })
    }
}

impl From<GlobalIndex> for proto::FixedBytes32 {
    fn from(value: GlobalIndex) -> Self {
        value.into_u256().into()
    }
}

impl TryFrom<proto::ImportedBridgeExit> for ImportedBridgeExit {
    type Error = CertificateConversionError;

    fn try_from(value: proto::ImportedBridgeExit) -> Result<Self, Self::Error> {
        Ok(Self {
            bridge_exit: required_field!(value, bridge_exit)?.try_into()?,
            claim_data: required_field!(value, claim)?.try_into()?,
            global_index: required_field!(value, global_index)?.try_into()?,
        })
    }
}

impl From<ImportedBridgeExit> for proto::ImportedBridgeExit {
    fn from(value: ImportedBridgeExit) -> Self {
        Self {
            bridge_exit: Some(value.bridge_exit.into()),
            global_index: Some(value.global_index.into()),
            claim: Some(value.claim_data.into()),
        }
    }
}

impl TryFrom<proto::AggchainProofPublicValues> for AggchainProofPublicValues {
    type Error = CertificateConversionError;

    fn try_from(value: proto::AggchainProofPublicValues) -> Result<Self, Self::Error> {
        Ok(Self {
            prev_local_exit_root: required_field!(value, prev_local_exit_root)?.try_into()?,
            new_local_exit_root: required_field!(value, new_local_exit_root)?.try_into()?,
            l1_info_root: required_field!(value, l1_info_root)?.try_into()?,
            origin_network: NetworkId::new(value.origin_network),
            commit_imported_bridge_exits: required_field!(
                value.commit_imported_bridge_exits,
                "commit_imported_bridge_exits",
            )?
            .try_into()?,
            aggchain_params: required_field!(value, aggchain_params)?.try_into()?,
        })
    }
}

impl From<AggchainProofPublicValues> for proto::AggchainProofPublicValues {
    fn from(value: AggchainProofPublicValues) -> Self {
        Self {
            prev_local_exit_root: Some(value.prev_local_exit_root.into()),
            new_local_exit_root: Some(value.new_local_exit_root.into()),
            l1_info_root: Some(value.l1_info_root.into()),
            origin_network: value.origin_network.to_u32(),
            commit_imported_bridge_exits: Some(value.commit_imported_bridge_exits.into()),
            aggchain_params: Some(value.aggchain_params.into()),
        }
    }
}

impl TryFrom<proto::MultisigPayload> for MultisigPayload {
    type Error = CertificateConversionError;

    fn try_from(value: proto::MultisigPayload) -> Result<Self, Self::Error> {
        Ok(Self(
            value
                .signatures
                .into_iter()
                .map(|entry| {
                    entry
                        .signature
                        .map(|sig| parse_signature(sig, "multisig.signature"))
                        .transpose()
                })
                .collect::<Result<_, _>>()?,
        ))
    }
}

impl From<MultisigPayload> for proto::MultisigPayload {
    fn from(value: MultisigPayload) -> Self {
        Self {
            signatures: value
                .0
                .into_iter()
                .map(|signature| proto::MultisigEntry {
                    signature: signature.map(Into::into),
                })
                .collect(),
        }
    }
}

impl TryFrom<proto::AggchainData> for AggchainData {
    type Error = CertificateConversionError;

    fn try_from(value: proto::AggchainData) -> Result<Self, Self::Error> {
        Ok(match required_field!(value, data)? {
            proto::aggchain_data::Data::Ecdsa(signature) => Self::ECDSA {
                signature: parse_signature(signature, "aggchain_data.ecdsa")?,
            },
            proto::aggchain_data::Data::Generic(generic) => Self::Generic {
                proof: Proof::try_from(required_field!(
                    generic.proof,
                    "aggchain_data.generic.proof"
                )?)?,
                aggchain_params: required_field!(
                    generic.aggchain_params,
                    "aggchain_data.generic.aggchain_params",
                )?
                .try_into()?,
                signature: generic
                    .signature
                    .map(|signature| {
                        parse_signature(signature, "aggchain_data.generic.signature").map(Box::new)
                    })
                    .transpose()?,
                public_values: generic
                    .public_values
                    .map(TryInto::try_into)
                    .transpose()?
                    .map(Box::new),
            },
            proto::aggchain_data::Data::MultisigOnly(multisig_only) => Self::MultisigOnly {
                multisig: required_field!(
                    multisig_only.multisig,
                    "aggchain_data.multisig_only.multisig"
                )?
                .try_into()?,
            },
            proto::aggchain_data::Data::MultisigAndAggchainProof(multisig_and_proof) => {
                Self::MultisigAndAggchainProof {
                    multisig: required_field!(
                        multisig_and_proof.multisig,
                        "aggchain_data.multisig_and_aggchain_proof.multisig",
                    )?
                    .try_into()?,
                    aggchain_proof: AggchainProof {
                        proof: Proof::try_from(required_field!(
                            multisig_and_proof.proof,
                            "aggchain_data.multisig_and_aggchain_proof.proof",
                        )?)?,
                        aggchain_params: required_field!(
                            multisig_and_proof.aggchain_params,
                            "aggchain_data.multisig_and_aggchain_proof.aggchain_params",
                        )?
                        .try_into()?,
                        public_values: multisig_and_proof
                            .public_values
                            .map(TryInto::try_into)
                            .transpose()?
                            .map(Box::new),
                    },
                }
            }
        })
    }
}

impl TryFrom<&AggchainData> for proto::AggchainData {
    type Error = CertificateConversionError;

    fn try_from(value: &AggchainData) -> Result<Self, Self::Error> {
        Ok(Self {
            data: Some(match value {
                AggchainData::ECDSA { signature } => {
                    proto::aggchain_data::Data::Ecdsa((*signature).into())
                }
                AggchainData::Generic {
                    proof,
                    aggchain_params,
                    signature,
                    public_values,
                } => proto::aggchain_data::Data::Generic(proto::Generic {
                    proof: Some(proto::Proof::try_from(proof)?),
                    aggchain_params: Some((*aggchain_params).into()),
                    signature: signature.as_ref().map(|signature| (**signature).into()),
                    public_values: public_values.as_ref().map(|value| (**value).clone().into()),
                }),
                AggchainData::MultisigOnly { multisig } => {
                    proto::aggchain_data::Data::MultisigOnly(proto::MultisigOnly {
                        multisig: Some(multisig.clone().into()),
                    })
                }
                AggchainData::MultisigAndAggchainProof {
                    multisig,
                    aggchain_proof,
                } => proto::aggchain_data::Data::MultisigAndAggchainProof(
                    proto::MultisigAndAggchainProof {
                        multisig: Some(multisig.clone().into()),
                        proof: Some(proto::Proof::try_from(&aggchain_proof.proof)?),
                        aggchain_params: Some(aggchain_proof.aggchain_params.into()),
                        public_values: aggchain_proof
                            .public_values
                            .as_ref()
                            .map(|value| (**value).clone().into()),
                    },
                ),
            }),
        })
    }
}

impl TryFrom<proto::Certificate> for Certificate {
    type Error = CertificateConversionError;

    fn try_from(value: proto::Certificate) -> Result<Self, Self::Error> {
        Ok(Self {
            network_id: NetworkId::new(value.network_id),
            height: Height::new(value.height),
            prev_local_exit_root: required_field!(value, prev_local_exit_root)?.try_into()?,
            new_local_exit_root: required_field!(value, new_local_exit_root)?.try_into()?,
            bridge_exits: value
                .bridge_exits
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            imported_bridge_exits: value
                .imported_bridge_exits
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            metadata: required_field!(value, metadata)?.try_into()?,
            aggchain_data: required_field!(value, aggchain_data)?.try_into()?,
            custom_chain_data: value.custom_chain_data.to_vec(),
            l1_info_tree_leaf_count: value.l1_info_tree_leaf_count,
        })
    }
}

impl TryFrom<&Certificate> for proto::Certificate {
    type Error = CertificateConversionError;

    fn try_from(value: &Certificate) -> Result<Self, Self::Error> {
        Ok(Self {
            network_id: value.network_id.to_u32(),
            height: value.height.as_u64(),
            prev_local_exit_root: Some(value.prev_local_exit_root.into()),
            new_local_exit_root: Some(value.new_local_exit_root.into()),
            bridge_exits: value.bridge_exits.iter().cloned().map(Into::into).collect(),
            imported_bridge_exits: value
                .imported_bridge_exits
                .iter()
                .cloned()
                .map(Into::into)
                .collect(),
            metadata: Some(value.metadata.into()),
            aggchain_data: Some((&value.aggchain_data).try_into()?),
            custom_chain_data: value.custom_chain_data.clone().into(),
            l1_info_tree_leaf_count: value.l1_info_tree_leaf_count,
        })
    }
}

/// A unit type serializing to a constant byte representing the storage version.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, Eq, PartialEq)]
#[serde(try_from = "u8", into = "u8")]
struct VersionTag<const VERSION: u8>;

impl<const VERSION: u8> TryFrom<u8> for VersionTag<VERSION> {
    type Error = CodecError;
    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        (byte == VERSION)
            .then_some(Self)
            .ok_or(CodecError::BadCertificateVersion { version: byte })
    }
}

impl<const VERSION: u8> From<VersionTag<VERSION>> for u8 {
    fn from(VersionTag: VersionTag<VERSION>) -> Self {
        VERSION
    }
}

/// A three-byte network ID used in v0.
///
/// In v0, the first byte of network ID was reserved to specify the storage
/// format version.
#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq)]
#[cfg_attr(feature = "testutils", derive(Serialize))]
struct NetworkIdV0([u8; 3]);

impl From<NetworkIdV0> for NetworkId {
    fn from(NetworkIdV0([b2, b1, b0]): NetworkIdV0) -> NetworkId {
        u32::from_be_bytes([0, b2, b1, b0]).into()
    }
}

/// The pre-0.3 certificate format (`v0`).
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "testutils", derive(Serialize, Eq, PartialEq))]
struct CertificateV0 {
    version: VersionTag<0>,
    network_id: NetworkIdV0,
    height: Height,
    prev_local_exit_root: LocalExitRoot,
    new_local_exit_root: LocalExitRoot,
    bridge_exits: Vec<BridgeExit>,
    imported_bridge_exits: Vec<ImportedBridgeExit>,
    signature: Signature,
    metadata: Metadata,
}

impl From<CertificateV0> for Certificate {
    fn from(certificate: CertificateV0) -> Self {
        let CertificateV0 {
            version: VersionTag,
            network_id,
            height,
            prev_local_exit_root,
            new_local_exit_root,
            bridge_exits,
            imported_bridge_exits,
            signature,
            metadata,
        } = certificate;

        Certificate {
            network_id: network_id.into(),
            height,
            prev_local_exit_root,
            new_local_exit_root,
            bridge_exits,
            imported_bridge_exits,
            aggchain_data: AggchainData::ECDSA { signature },
            metadata,
            custom_chain_data: vec![],
            l1_info_tree_leaf_count: None,
        }
    }
}

/// The new certificate format as stored in the database (`v1`).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CertificateV1<'a> {
    version: VersionTag<1>,
    network_id: NetworkId,
    height: Height,
    prev_local_exit_root: LocalExitRoot,
    new_local_exit_root: LocalExitRoot,
    bridge_exits: Cow<'a, [BridgeExit]>,
    imported_bridge_exits: Cow<'a, [ImportedBridgeExit]>,
    aggchain_data: AggchainDataV1<'a>,
    metadata: Metadata,
    custom_chain_data: Cow<'a, [u8]>,
    l1_info_tree_leaf_count: Option<u32>,
}

impl TryFrom<CertificateV1<'_>> for Certificate {
    type Error = CodecError;

    fn try_from(certificate: CertificateV1) -> Result<Self, Self::Error> {
        let CertificateV1 {
            version: VersionTag,
            network_id,
            height,
            prev_local_exit_root,
            new_local_exit_root,
            bridge_exits,
            imported_bridge_exits,
            aggchain_data,
            metadata,
            custom_chain_data,
            l1_info_tree_leaf_count,
        } = certificate;

        Ok(Certificate {
            network_id,
            height,
            prev_local_exit_root,
            new_local_exit_root,
            bridge_exits: bridge_exits.into_owned(),
            imported_bridge_exits: imported_bridge_exits.into_owned(),
            metadata,
            aggchain_data: aggchain_data.try_into()?,
            custom_chain_data: custom_chain_data.into_owned(),
            l1_info_tree_leaf_count,
        })
    }
}

// Historical storage-v1 rows embed aggchain proof types from
// `agglayer-interop-types` 0.13.x, before the SP1 v6 proof upgrade.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum AggchainDataV1<'a> {
    ECDSA {
        signature: Signature,
    },

    GenericNoSignature {
        proof: Cow<'a, legacy_interop_types::aggchain_proof::Proof>,
        aggchain_params: Digest,
    },

    GenericWithSignature {
        proof: Cow<'a, legacy_interop_types::aggchain_proof::Proof>,
        aggchain_params: Digest,
        signature: Cow<'a, Box<Signature>>,
    },

    GenericWithPublicValues {
        proof: Cow<'a, legacy_interop_types::aggchain_proof::Proof>,
        aggchain_params: Digest,
        signature: Option<Box<Signature>>,
        public_values: Cow<'a, Box<AggchainProofPublicValues>>,
    },

    MultisigOnly {
        multisig: Cow<'a, [Option<Signature>]>,
    },

    MultisigAndAggchainProof {
        multisig: Cow<'a, [Option<Signature>]>,
        proof: Cow<'a, legacy_interop_types::aggchain_proof::Proof>,
        aggchain_params: Digest,
        public_values: Option<Cow<'a, Box<AggchainProofPublicValues>>>,
    },
}

fn legacy_proof_into_current(
    proof: legacy_interop_types::aggchain_proof::Proof,
) -> Result<Proof, CodecError> {
    let proof = match proof {
        legacy_interop_types::aggchain_proof::Proof::SP1Stark(proof) => {
            Proof::SP1Stark(SP1StarkWithContext {
                version: proof.version,
                proof: agglayer_types::bincode::sp1v4().serialize(proof.proof.as_ref())?,
                vkey: agglayer_types::bincode::sp1v4().serialize(&proof.vkey)?,
            })
        }
    };

    proof
        .ensure_readable(&agglayer_sp1::AcceptancePolicy::DEFAULT)
        .map_err(|err| CodecError::InvalidEnumVariant(err.to_string()))?;

    Ok(proof)
}

impl TryFrom<AggchainDataV1<'_>> for AggchainData {
    type Error = CodecError;

    fn try_from(proof: AggchainDataV1) -> Result<Self, Self::Error> {
        Ok(match proof {
            AggchainDataV1::ECDSA { signature } => Self::ECDSA { signature },
            AggchainDataV1::GenericNoSignature {
                proof,
                aggchain_params,
            } => Self::Generic {
                proof: legacy_proof_into_current(proof.into_owned())?,
                aggchain_params,
                signature: None,
                public_values: None,
            },
            AggchainDataV1::GenericWithSignature {
                proof,
                aggchain_params,
                signature,
            } => Self::Generic {
                proof: legacy_proof_into_current(proof.into_owned())?,
                aggchain_params,
                signature: Some(signature.into_owned()),
                public_values: None,
            },
            AggchainDataV1::GenericWithPublicValues {
                proof,
                aggchain_params,
                signature,
                public_values,
            } => Self::Generic {
                proof: legacy_proof_into_current(proof.into_owned())?,
                aggchain_params,
                signature,
                public_values: Some(public_values.into_owned()),
            },
            AggchainDataV1::MultisigOnly { multisig } => Self::MultisigOnly {
                multisig: MultisigPayload(multisig.into_owned()),
            },
            AggchainDataV1::MultisigAndAggchainProof {
                multisig,
                proof,
                aggchain_params,
                public_values,
            } => Self::MultisigAndAggchainProof {
                multisig: MultisigPayload(multisig.into_owned()),
                aggchain_proof: AggchainProof {
                    proof: legacy_proof_into_current(proof.into_owned())?,
                    aggchain_params,
                    public_values: public_values.map(|pv| pv.into_owned()),
                },
            },
        })
    }
}

fn panic_bincode_error() -> agglayer_types::bincode::Error {
    Box::new(agglayer_types::bincode::ErrorKind::Custom(String::from(
        "panic during deserialization",
    )))
}

/// Wrap `bincode` deserialization in `catch_unwind` so a panic from a
/// malformed row surfaces as `CodecError::Serialization` instead of
/// unwinding the node process.
///
/// The closure captures `&[u8]` and a `bincode::Options` value, both of
/// which are `UnwindSafe`; no `AssertUnwindSafe` is needed. Clippy's
/// `catch_unwind` lint is not triggered here, confirming the
/// `UnwindSafe` bound is satisfied by inference.
fn deserialize_bincode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, CodecError> {
    std::panic::catch_unwind(|| bincode_codec().deserialize::<T>(bytes))
        .map_err(|_| CodecError::Serialization(panic_bincode_error()))?
        .map_err(CodecError::Serialization)
}

fn decode<T>(bytes: &[u8]) -> Result<Certificate, CodecError>
where
    T: DeserializeOwned,
    Certificate: TryFrom<T, Error = CodecError>,
{
    Certificate::try_from(deserialize_bincode::<T>(bytes)?)
}

fn decode_v0(bytes: &[u8]) -> Result<Certificate, CodecError> {
    Ok(deserialize_bincode::<CertificateV0>(bytes)?.into())
}

impl crate::schema::Codec for Certificate {
    fn encode_into<W: std::io::Write>(&self, mut writer: W) -> Result<(), CodecError> {
        let proto = proto::Certificate::try_from(self)
            .map_err(|error| CodecError::Conversion(error.to_string()))?;
        let mut buf = prost::bytes::BytesMut::with_capacity(proto.encoded_len());
        proto.encode(&mut buf)?;
        writer.write_all(&buf)?;

        Ok(())
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        match bytes.first().copied() {
            None => Err(CodecError::CertificateEmpty),
            Some(0) => decode_v0(bytes),
            Some(1) => decode::<CertificateV1>(bytes),
            Some(_) => {
                let proto = proto::Certificate::decode(bytes)?;
                Certificate::try_from(proto)
                    .map_err(|error| CodecError::Conversion(error.to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests;
