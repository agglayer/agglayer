//! Definitions of the certificate storage format with backwards compatibility.
//!
//! Currently, we have two versions of certificate storage format. The first byte determines the
//! storage format version.
//!
//! In version 0, where backwards compatibility is required, the first byte happens to be the
//! highest byte of network ID. This effectively limits the range of network IDs in v0 storage
//! to [0, 2^24-1]. The current network IDs fall into this range, so the highest byte (with value 0)
//! also acts as the version tag.
//!
//! In subsequent versions, we just have the version byte followed by a straightforward encoding of
//! the certificate, restoring the full range of network IDs.
//!
//! In the unlikely scenario where it turns out we need more than 256 storage format versions,
//! another byte can be allocated to specify a "sub-version" in one of the future versions.

use std::borrow::Cow;

use agglayer_types::{Certificate, Digest, Height, Metadata, NetworkId, Signature};
use bincode::Options;
use pessimistic_proof::{
    aggchain_proof::{AggchainProof, AggchainProofSP1},
    bridge_exit::BridgeExit,
    imported_bridge_exit::ImportedBridgeExit,
};
use serde::{Deserialize, Serialize};

use crate::columns::{default_bincode_options, CodecError};

/// A unit type serializing to a constant byte representing the storage version.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
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
/// In v0, the first byte of network ID was reserved to specify the storage format version.
#[derive(Debug, Clone, Copy, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
struct NetworkIdV0([u8; 3]);

impl From<NetworkIdV0> for NetworkId {
    fn from(NetworkIdV0([b2, b1, b0]): NetworkIdV0) -> NetworkId {
        u32::from_be_bytes([0, b2, b1, b0]).into()
    }
}

/// The pre-0.3 certificate format (`v0`).
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
struct CertificateV0 {
    version: VersionTag<0>,
    network_id: NetworkIdV0,
    height: Height,
    prev_local_exit_root: Digest,
    new_local_exit_root: Digest,
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
            aggchain_proof: AggchainProof::ECDSA { signature },
            metadata,
        }
    }
}

/// The new certificate format as stored in the database (`v1`).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CertificateV1<'a> {
    version: VersionTag<1>,
    network_id: NetworkId,
    height: Height,
    prev_local_exit_root: Digest,
    new_local_exit_root: Digest,
    bridge_exits: Cow<'a, [BridgeExit]>,
    imported_bridge_exits: Cow<'a, [ImportedBridgeExit]>,
    aggchain_proof: AggchainProofV1<'a>,
    metadata: Metadata,
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
            aggchain_proof,
            metadata,
        } = certificate;

        Certificate {
            network_id,
            height,
            prev_local_exit_root,
            new_local_exit_root,
            bridge_exits: bridge_exits.into_owned(),
            imported_bridge_exits: imported_bridge_exits.into_owned(),
            metadata,
            aggchain_proof: aggchain_proof.into(),
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
            aggchain_proof,
        } = certificate;

        CertificateV1 {
            version: VersionTag,
            network_id: *network_id,
            height: *height,
            prev_local_exit_root: *prev_local_exit_root,
            new_local_exit_root: *new_local_exit_root,
            bridge_exits: bridge_exits.into(),
            imported_bridge_exits: imported_bridge_exits.into(),
            aggchain_proof: aggchain_proof.into(),
            metadata: *metadata,
        }
    }
}

// Duplicated from `agglayer-types` since we need slightly different serde impls.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum AggchainProofV1<'a> {
    ECDSA {
        signature: Signature,
    },
    SP1 {
        aggchain_proof: Cow<'a, AggchainProofSP1>,
    },
}

impl<'a> From<&'a AggchainProof> for AggchainProofV1<'a> {
    fn from(proof: &'a AggchainProof) -> Self {
        match proof {
            AggchainProof::ECDSA { signature } => Self::ECDSA {
                signature: *signature,
            },
            AggchainProof::SP1 { aggchain_proof } => Self::SP1 {
                aggchain_proof: Cow::Borrowed(aggchain_proof),
            },
        }
    }
}

impl From<AggchainProofV1<'_>> for AggchainProof {
    fn from(proof: AggchainProofV1) -> Self {
        match proof {
            AggchainProofV1::ECDSA { signature } => Self::ECDSA { signature },
            AggchainProofV1::SP1 { aggchain_proof } => Self::SP1 {
                aggchain_proof: aggchain_proof.into_owned(),
            },
        }
    }
}

/// Type specifying the current certificate encoding format.
type CurrentCertificate<'a> = CertificateV1<'a>;

fn decode<T: for<'de> Deserialize<'de> + Into<Certificate>>(
    bytes: &[u8],
) -> Result<Certificate, CodecError> {
    Ok(default_bincode_options().deserialize::<T>(bytes)?.into())
}

impl crate::columns::Codec for Certificate {
    fn encode(&self) -> Result<Vec<u8>, CodecError> {
        // TODO get rid of the clones <https://github.com/agglayer/agglayer/issues/618>
        Ok(default_bincode_options().serialize(&CurrentCertificate::from(self))?)
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
