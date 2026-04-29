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

use agglayer_interop_types_v13 as legacy_interop_types;
use agglayer_sp1::ProofExt as _;
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

#[path = "proto.rs"]
mod proto_conversions;

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
    value: proto::Signature,
    field: &'static str,
) -> Result<Signature, CertificateConversionError> {
    Signature::try_from(value.value.as_ref())
        .map_err(|source| CertificateConversionError::Signature { field, source })
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
