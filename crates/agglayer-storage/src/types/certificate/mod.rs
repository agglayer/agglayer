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

use agglayer_tries::roots::LocalExitRoot;
use agglayer_types::{
    aggchain_proof::{AggchainData, AggchainProof, MultisigPayload, Proof},
    primitives::Digest,
    Certificate, Height, Metadata, NetworkId, Signature,
};
use pessimistic_proof::unified_bridge::{
    AggchainProofPublicValues, BridgeExit, ImportedBridgeExit,
};
use serde::{Deserialize, Serialize};

use crate::columns::{bincode_codec, CodecError};

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

            AggchainData::MultisigOnly(multisig) => AggchainDataV1::MultisigOnly {
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
            AggchainDataV1::MultisigOnly { multisig } => {
                Self::MultisigOnly(MultisigPayload(multisig.into_owned()))
            }
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

/// Type specifying the current certificate encoding format.
type CurrentCertificate<'a> = CertificateV1<'a>;

fn decode<T: for<'de> Deserialize<'de> + Into<Certificate>>(
    bytes: &[u8],
) -> Result<Certificate, CodecError> {
    Ok(bincode_codec().deserialize::<T>(bytes)?.into())
}

impl crate::columns::Codec for Certificate {
    fn encode(&self) -> Result<Vec<u8>, CodecError> {
        // TODO get rid of the clones <https://github.com/agglayer/agglayer/issues/618>
        Ok(bincode_codec().serialize(&CurrentCertificate::from(self))?)
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        match bytes.first().copied() {
            None => Err(CodecError::CertificateEmpty),
            Some(0) => decode::<CertificateV0>(bytes),
            Some(1) => decode::<CertificateV1>(bytes),
            Some(version) => Err(CodecError::BadCertificateVersion { version }),
        }
    }
}

#[cfg(test)]
mod tests;
