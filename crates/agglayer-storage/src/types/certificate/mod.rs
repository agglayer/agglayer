//! Definitions of the certificate storage format with backwards compatibility.
//!
//! Currently, we have three versions of certificate storage format. The first
//! byte determines the storage format version.
//!
//! In version 0, where backwards compatibility is required, the first byte
//! happens to be the highest byte of network ID. This effectively limits the
//! range of network IDs in v0 storage to [0, 2^24-1]. The current network IDs
//! fall into this range, so the highest byte (with value 0) also acts as the
//! version tag.
//!
//! Version 1 stored aggchain proofs using the historical typed schema from
//! `agglayer-interop-types` 0.13.x.
//! Version 2 stores the aggchain proof as a versioned byte envelope.
//!
//! New writes always use v2.

use std::borrow::Cow;

use agglayer_interop_types_v13 as legacy_interop_types;
use agglayer_tries::roots::LocalExitRoot;
use agglayer_types::{
    aggchain_proof::{
        AggchainData, AggchainProof, Proof, ProofError, ProofExt as _, SP1StarkWithContext,
        SP1StarkWithContextExt as _,
    },
    primitives::Digest,
    Certificate, Height, Metadata, NetworkId, Signature,
};
use pessimistic_proof::unified_bridge::{
    AggchainProofPublicValues, BridgeExit, ImportedBridgeExit,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::schema::{bincode_codec, CodecError};

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

impl TryFrom<CertificateV0> for Certificate {
    type Error = CodecError;

    fn try_from(certificate: CertificateV0) -> Result<Self, Self::Error> {
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

        Ok(Certificate {
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
        })
    }
}

/// Historical storage-v1 rows embed aggchain proof types from
/// `agglayer-interop-types` 0.13.x, before the SP1 v6 proof upgrade.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
enum AggchainDataV1<'a> {
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

fn legacy_proof_into_v2(
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
        .sp1()
        .ensure_readable()
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
                proof: legacy_proof_into_v2(proof.into_owned())?,
                aggchain_params,
                signature: None,
                public_values: None,
            },
            AggchainDataV1::GenericWithSignature {
                proof,
                aggchain_params,
                signature,
            } => Self::Generic {
                proof: legacy_proof_into_v2(proof.into_owned())?,
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
                proof: legacy_proof_into_v2(proof.into_owned())?,
                aggchain_params,
                signature,
                public_values: Some(public_values.into_owned()),
            },
            AggchainDataV1::MultisigOnly { multisig } => Self::MultisigOnly {
                multisig: agglayer_types::aggchain_proof::MultisigPayload(multisig.into_owned()),
            },
            AggchainDataV1::MultisigAndAggchainProof {
                multisig,
                proof,
                aggchain_params,
                public_values,
            } => Self::MultisigAndAggchainProof {
                multisig: agglayer_types::aggchain_proof::MultisigPayload(multisig.into_owned()),
                aggchain_proof: AggchainProof {
                    proof: legacy_proof_into_v2(proof.into_owned())?,
                    aggchain_params,
                    public_values: public_values.map(|pv| pv.into_owned()),
                },
            },
        })
    }
}

// Historical storage-v1 certificates decode through the 0.13.x aggchain-data
// wrapper, even though the outer certificate storage version is 1.
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

/// Current certificate storage format (`v2`) using the versioned proof
/// envelope from `agglayer-types`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "testutils", derive(Eq, PartialEq))]
struct CertificateV2<'a> {
    version: VersionTag<2>,
    network_id: NetworkId,
    height: Height,
    prev_local_exit_root: LocalExitRoot,
    new_local_exit_root: LocalExitRoot,
    bridge_exits: Cow<'a, [BridgeExit]>,
    imported_bridge_exits: Cow<'a, [ImportedBridgeExit]>,
    aggchain_data: AggchainDataV2<'a>,
    metadata: Metadata,
    custom_chain_data: Cow<'a, [u8]>,
    l1_info_tree_leaf_count: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "testutils", derive(Eq, PartialEq))]
#[allow(clippy::upper_case_acronyms)]
enum AggchainDataV2<'a> {
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

impl<'a> From<&'a AggchainData> for AggchainDataV2<'a> {
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
            AggchainData::MultisigOnly { multisig } => Self::MultisigOnly {
                multisig: Cow::Borrowed(multisig.0.as_slice()),
            },
            AggchainData::MultisigAndAggchainProof {
                multisig,
                aggchain_proof,
            } => Self::MultisigAndAggchainProof {
                multisig: Cow::Borrowed(multisig.0.as_slice()),
                proof: Cow::Borrowed(&aggchain_proof.proof),
                aggchain_params: aggchain_proof.aggchain_params,
                public_values: aggchain_proof.public_values.as_ref().map(Cow::Borrowed),
            },
        }
    }
}

impl TryFrom<AggchainDataV2<'_>> for AggchainData {
    type Error = CodecError;

    fn try_from(proof: AggchainDataV2) -> Result<Self, Self::Error> {
        Ok(match proof {
            AggchainDataV2::ECDSA { signature } => Self::ECDSA { signature },
            AggchainDataV2::GenericNoSignature {
                proof,
                aggchain_params,
            } => Self::Generic {
                proof: {
                    let proof = proof.into_owned();
                    proof
                        .sp1()
                        .ensure_readable()
                        .map_err(|err| CodecError::InvalidEnumVariant(err.to_string()))?;
                    proof
                },
                aggchain_params,
                signature: None,
                public_values: None,
            },
            AggchainDataV2::GenericWithSignature {
                proof,
                aggchain_params,
                signature,
            } => Self::Generic {
                proof: {
                    let proof = proof.into_owned();
                    proof
                        .sp1()
                        .ensure_readable()
                        .map_err(|err| CodecError::InvalidEnumVariant(err.to_string()))?;
                    proof
                },
                aggchain_params,
                signature: Some(signature.into_owned()),
                public_values: None,
            },
            AggchainDataV2::GenericWithPublicValues {
                proof,
                aggchain_params,
                signature,
                public_values,
            } => Self::Generic {
                proof: {
                    let proof = proof.into_owned();
                    proof
                        .sp1()
                        .ensure_readable()
                        .map_err(|err| CodecError::InvalidEnumVariant(err.to_string()))?;
                    proof
                },
                aggchain_params,
                signature,
                public_values: Some(public_values.into_owned()),
            },
            AggchainDataV2::MultisigOnly { multisig } => Self::MultisigOnly {
                multisig: agglayer_types::aggchain_proof::MultisigPayload(multisig.into_owned()),
            },
            AggchainDataV2::MultisigAndAggchainProof {
                multisig,
                proof,
                aggchain_params,
                public_values,
            } => Self::MultisigAndAggchainProof {
                multisig: agglayer_types::aggchain_proof::MultisigPayload(multisig.into_owned()),
                aggchain_proof: AggchainProof {
                    proof: {
                        let proof = proof.into_owned();
                        proof
                            .sp1()
                            .ensure_readable()
                            .map_err(|err| CodecError::InvalidEnumVariant(err.to_string()))?;
                        proof
                    },
                    aggchain_params,
                    public_values: public_values.map(|pv| pv.into_owned()),
                },
            },
        })
    }
}

impl<'a> From<&'a Certificate> for CertificateV2<'a> {
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

        CertificateV2 {
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

impl TryFrom<CertificateV2<'_>> for Certificate {
    type Error = CodecError;

    fn try_from(certificate: CertificateV2) -> Result<Self, Self::Error> {
        let CertificateV2 {
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

/// Type specifying the current certificate encoding format.
type CurrentCertificate<'a> = CertificateV2<'a>;

fn panic_bincode_error() -> agglayer_types::bincode::Error {
    Box::new(agglayer_types::bincode::ErrorKind::Custom(String::from(
        "panic during deserialization",
    )))
}

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

fn ensure_storable(certificate: &Certificate) -> Result<(), CodecError> {
    let result = match &certificate.aggchain_data {
        AggchainData::ECDSA { .. } | AggchainData::MultisigOnly { .. } => Ok(()),
        AggchainData::Generic { proof, .. } => proof.ensure_readable().map(|_| ()),
        AggchainData::MultisigAndAggchainProof { aggchain_proof, .. } => {
            aggchain_proof.proof.ensure_readable().map(|_| ())
        }
    };

    result.map_err(|err| match err {
        ProofError::UnsupportedReadableSp1Version { version } => {
            CodecError::NonWritableCertificate {
                reason: format!(
                    "SP1 proof version `{version}` is not readable in certificate storage v2"
                ),
            }
        }
        other => CodecError::NonWritableCertificate {
            reason: other.to_string(),
        },
    })
}

impl crate::schema::Codec for Certificate {
    fn encode_into<W: std::io::Write>(&self, writer: W) -> Result<(), CodecError> {
        ensure_storable(self)?;
        Ok(bincode_codec().serialize_into(writer, &CurrentCertificate::from(self))?)
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        match bytes.first().copied() {
            None => Err(CodecError::CertificateEmpty),
            Some(0) => decode::<CertificateV0>(bytes),
            Some(1) => decode::<CertificateV1>(bytes),
            Some(2) => decode::<CertificateV2>(bytes),
            Some(version) => Err(CodecError::BadCertificateVersion { version }),
        }
    }
}

#[cfg(test)]
mod tests;
