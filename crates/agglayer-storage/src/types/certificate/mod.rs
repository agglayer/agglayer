//! Definitions of the certificate storage format.
//!
//! ## Two distinct certificate codecs
//!
//! The crate exposes two value types for certificate-bearing column families,
//! each with its own [`crate::schema::Codec`]:
//!
//! * [`Certificate`] — protobuf encoding only. Used by the proto-backed runtime
//!   column families (e.g. `CertificatePerIndexProtoColumn`,
//!   `DebugCertificatesProtoColumn`, `PendingQueueProtoColumn`). New writes go
//!   here.
//!
//! * [`LegacyCertificate`] — transitional newtype wrapper used by the legacy
//!   column families. Encoding emits `v1` bincode (runtime code does not write
//!   to legacy CFs after the proto migration). Decoding accepts *either* legacy
//!   bincode (`v0`/`v1`) *or* protobuf, because the legacy CFs historically
//!   received both: bincode rows pre-#1519 and proto rows between #1519 (proto
//!   codec switch) and this PR (proto CF split).
//!
//! Splitting the codecs is deliberate: the proto CF is byte-unambiguous and
//! its codec is strictly proto. Format sniffing is restricted to the
//! transitional `LegacyCertificate` type, whose CF is read-only after
//! migration and slated for removal in a follow-up.
//!
//! ## Legacy bincode format history
//!
//! For historical context, the legacy bincode payloads carry a leading byte
//! that distinguishes the two pre-proto formats:
//!
//! * `v0`: the first byte happens to be the highest byte of network ID. This
//!   effectively limits the range of network IDs in v0 storage to `[0,
//!   2^24-1]`. All network IDs that ever made it into v0 storage fell into this
//!   range, so the highest byte (with value 0) also acts as the version tag.
//!
//! * `v1`: a leading version byte (`1`) followed by a straightforward encoding
//!   of the certificate, restoring the full range of network IDs.

use std::borrow::Cow;

use agglayer_tries::roots::LocalExitRoot;
use agglayer_types::{
    aggchain_proof::{AggchainData, AggchainProof, MultisigPayload, Proof},
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
#[cfg_attr(feature = "testutils", derive(Eq, PartialEq))]
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

impl From<CertificateV1<'_>> for Certificate {
    fn from(certificate: CertificateV1) -> Self {
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

        Certificate {
            network_id,
            height,
            prev_local_exit_root,
            new_local_exit_root,
            bridge_exits: bridge_exits.into_owned(),
            imported_bridge_exits: imported_bridge_exits.into_owned(),
            metadata,
            aggchain_data: aggchain_data.into(),
            custom_chain_data: custom_chain_data.into_owned(),
            l1_info_tree_leaf_count,
        }
    }
}

impl<'a> From<&'a Certificate> for CertificateV1<'a> {
    fn from(certificate: &'a Certificate) -> Self {
        let Certificate {
            network_id,
            height,
            prev_local_exit_root,
            new_local_exit_root,
            bridge_exits,
            imported_bridge_exits,
            metadata,
            aggchain_data,
            custom_chain_data,
            l1_info_tree_leaf_count,
        } = certificate;

        CertificateV1 {
            version: VersionTag,
            network_id: *network_id,
            height: *height,
            prev_local_exit_root: *prev_local_exit_root,
            new_local_exit_root: *new_local_exit_root,
            bridge_exits: bridge_exits.into(),
            imported_bridge_exits: imported_bridge_exits.into(),
            aggchain_data: aggchain_data.into(),
            metadata: *metadata,
            custom_chain_data: custom_chain_data.into(),
            l1_info_tree_leaf_count: *l1_info_tree_leaf_count,
        }
    }
}

// Duplicated from `agglayer-types` since we need slightly different serde
// impls.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "testutils", derive(Eq, PartialEq))]
#[allow(clippy::upper_case_acronyms)]
pub enum AggchainDataV1<'a> {
    ECDSA {
        signature: Signature,
    },

    GenericNoSignature {
        proof: Cow<'a, Proof>,
        aggchain_params: Digest,
    },

    GenericWithSignature {
        proof: Cow<'a, Proof>,
        aggchain_params: Digest,
        signature: Cow<'a, Box<Signature>>,
    },

    GenericWithPublicValues {
        proof: Cow<'a, Proof>,
        aggchain_params: Digest,
        signature: Option<Box<Signature>>,
        public_values: Cow<'a, Box<AggchainProofPublicValues>>,
    },

    MultisigOnly {
        multisig: Cow<'a, [Option<Signature>]>,
    },

    MultisigAndAggchainProof {
        multisig: Cow<'a, [Option<Signature>]>,
        proof: Cow<'a, Proof>,
        aggchain_params: Digest,
        public_values: Option<Cow<'a, Box<AggchainProofPublicValues>>>,
    },
}

impl<'a> From<&'a AggchainData> for AggchainDataV1<'a> {
    fn from(proof: &'a AggchainData) -> Self {
        match proof {
            AggchainData::ECDSA { signature } => Self::ECDSA {
                signature: *signature,
            },
            AggchainData::Generic {
                proof,
                aggchain_params,
                signature,
                public_values,
            } => {
                let proof = Cow::Borrowed(proof);
                let aggchain_params = *aggchain_params;
                match public_values {
                    Some(pv) => Self::GenericWithPublicValues {
                        proof,
                        aggchain_params,
                        signature: signature.clone(),
                        public_values: Cow::Borrowed(pv),
                    },
                    None => match signature {
                        None => Self::GenericNoSignature {
                            proof,
                            aggchain_params,
                        },
                        Some(signature) => Self::GenericWithSignature {
                            proof,
                            aggchain_params,
                            signature: Cow::Borrowed(signature),
                        },
                    },
                }
            }

            AggchainData::MultisigOnly { multisig } => AggchainDataV1::MultisigOnly {
                multisig: Cow::Borrowed(multisig.0.as_slice()),
            },
            AggchainData::MultisigAndAggchainProof {
                multisig,
                aggchain_proof,
            } => AggchainDataV1::MultisigAndAggchainProof {
                multisig: Cow::Borrowed(multisig.0.as_slice()),
                proof: Cow::Borrowed(&aggchain_proof.proof),
                aggchain_params: aggchain_proof.aggchain_params,
                public_values: aggchain_proof.public_values.as_ref().map(Cow::Borrowed),
            },
        }
    }
}

impl From<AggchainDataV1<'_>> for AggchainData {
    fn from(proof: AggchainDataV1) -> Self {
        match proof {
            AggchainDataV1::ECDSA { signature } => Self::ECDSA { signature },
            AggchainDataV1::GenericNoSignature {
                proof,
                aggchain_params,
            } => Self::Generic {
                proof: proof.into_owned(),
                aggchain_params,
                signature: None,
                public_values: None,
            },
            AggchainDataV1::GenericWithSignature {
                proof,
                aggchain_params,
                signature,
            } => Self::Generic {
                proof: proof.into_owned(),
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
                proof: proof.into_owned(),
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
                    proof: proof.into_owned(),
                    aggchain_params,
                    public_values: public_values.map(|pv| pv.into_owned()),
                },
            },
        }
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

fn decode_legacy<T>(bytes: &[u8]) -> Result<Certificate, CodecError>
where
    T: DeserializeOwned + Into<Certificate>,
{
    Ok(deserialize_bincode::<T>(bytes)?.into())
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
        let proto = proto::Certificate::decode(bytes)?;
        Certificate::try_from(proto).map_err(|error| CodecError::Conversion(error.to_string()))
    }
}

/// Newtype wrapper for a certificate read out of a legacy column family.
///
/// This type exists to give the legacy column families a codec that is
/// distinct from [`Certificate`]'s strictly-proto codec. Decoding here is
/// deliberately permissive about format because the legacy CFs received
/// rows in *both* historical encodings:
///
/// * Bincode `v0`/`v1` rows written before the proto codec switch (#1519).
/// * Proto rows written between #1519 (proto codec became the default encode
///   for `Certificate`) and this PR (which created dedicated proto-backed CFs
///   and moved runtime writes there).
///
/// Decoding tries the bincode path on first byte `0` or `1` and falls back
/// to proto for anything else. Encoding emits `v1` bincode; runtime code
/// does not write to legacy CFs after the proto migration runs, so this
/// path is exercised mainly by tests that round-trip the legacy format.
///
/// **Transitional:** this type and the legacy column families it serves
/// will be removed in a follow-up ticket once the proto migration has been
/// validated in production. Do not extend it; new code should use
/// [`Certificate`] against the proto-backed CFs.
#[derive(Debug, Clone)]
pub struct LegacyCertificate(pub Certificate);

impl From<LegacyCertificate> for Certificate {
    fn from(LegacyCertificate(certificate): LegacyCertificate) -> Self {
        certificate
    }
}

impl crate::schema::Codec for LegacyCertificate {
    fn encode_into<W: std::io::Write>(&self, writer: W) -> Result<(), CodecError> {
        let v1 = CertificateV1::from(&self.0);
        bincode_codec().serialize_into(writer, &v1)?;
        Ok(())
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        let certificate = match bytes.first().copied() {
            None => return Err(CodecError::CertificateEmpty),
            Some(0) => decode_legacy::<CertificateV0>(bytes)?,
            Some(1) => decode_legacy::<CertificateV1>(bytes)?,
            Some(_) => {
                // Post-#1519 / pre-this-PR rows were proto-encoded into the
                // legacy CF before runtime writes moved to the proto CF.
                let proto = proto::Certificate::decode(bytes)?;
                Certificate::try_from(proto)
                    .map_err(|error| CodecError::Conversion(error.to_string()))?
            }
        };

        Ok(Self(certificate))
    }
}

#[cfg(test)]
mod tests;
