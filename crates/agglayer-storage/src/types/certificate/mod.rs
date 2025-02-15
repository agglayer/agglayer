//! Definitions of the certificate storage format with backwards compatibility.
//!
//! Currently, we have two versions of certificate storage format.
//! All begin with `network_id: 32`, followed by a 64-bit field which encodes
//! certificate height and/or format version. In the V0 format, this field
//! corresponds to certificate height, providing backwards compatibility.
//!
//! If the field is < [TAG_BEGIN], it is interpreted as height in storage format V0.
//! If the field is >= [TAG_BEGIN], it is interpreted as an identifier of the storage
//! format version. For versions > 0, the format is encoded in the former height field
//! as `TAG_BEGIN + FORMAT_VERSION`. In this case, the height has to be stored elsewhere.

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
            .ok_or(CodecError::BadCertVersion { version: byte })
    }
}

impl<const VERSION: u8> From<VersionTag<VERSION>> for u8 {
    fn from(VersionTag: VersionTag<VERSION>) -> Self {
        VERSION
    }
}

/// In v0, the first byte of network ID was reserved to specify the version.
#[derive(Debug, Clone, Copy, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
struct NetworkIdV0([u8; 3]);

impl From<NetworkIdV0> for NetworkId {
    fn from(NetworkIdV0([b2, b1, b0]): NetworkIdV0) -> NetworkId {
        u32::from_be_bytes([0, b2, b1, b0]).into()
    }
}

/// The pre-0.3 certificate format.
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

/// The new certificate format as stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CertificateV1 {
    version: VersionTag<1>,
    network_id: NetworkId,
    height: Height,
    prev_local_exit_root: Digest,
    new_local_exit_root: Digest,
    bridge_exits: Vec<BridgeExit>,
    imported_bridge_exits: Vec<ImportedBridgeExit>,
    aggchain_proof: AggchainProofV1,
    metadata: Metadata,
}

impl From<CertificateV1> for Certificate {
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
            bridge_exits,
            imported_bridge_exits,
            metadata,
            aggchain_proof: aggchain_proof.into(),
        }
    }
}

impl From<&Certificate> for CertificateV1 {
    fn from(certificate: &Certificate) -> Self {
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
            // TODO get rid of the clones
            bridge_exits: bridge_exits.clone(),
            imported_bridge_exits: imported_bridge_exits.clone(),
            aggchain_proof: aggchain_proof.clone().into(),
            metadata: *metadata,
        }
    }
}

// Duplicated since we need slightly different serde impls.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum AggchainProofV1 {
    ECDSA { signature: Signature },
    SP1 { aggchain_proof: AggchainProofSP1 },
}

impl From<AggchainProof> for AggchainProofV1 {
    fn from(proof: AggchainProof) -> Self {
        match proof {
            AggchainProof::ECDSA { signature } => Self::ECDSA { signature },
            AggchainProof::SP1 { aggchain_proof } => Self::SP1 { aggchain_proof },
        }
    }
}

impl From<AggchainProofV1> for AggchainProof {
    fn from(proof: AggchainProofV1) -> Self {
        match proof {
            AggchainProofV1::ECDSA { signature } => Self::ECDSA { signature },
            AggchainProofV1::SP1 { aggchain_proof } => Self::SP1 { aggchain_proof },
        }
    }
}

type CurrentCertificate = CertificateV1;

fn decode<T: for<'de> Deserialize<'de> + Into<Certificate>>(
    bytes: &[u8],
) -> Result<Certificate, CodecError> {
    Ok(default_bincode_options().deserialize::<T>(bytes)?.into())
}

impl crate::columns::Codec for Certificate {
    fn encode(&self) -> Result<Vec<u8>, CodecError> {
        Ok(default_bincode_options().serialize(&CurrentCertificate::from(self))?)
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        match bytes.first().copied() {
            None => Err(CodecError::CertEmpty),
            Some(0) => decode::<CertificateV0>(bytes),
            Some(1) => decode::<CertificateV1>(bytes),
            Some(version) => Err(CodecError::BadCertVersion { version }),
        }
    }
}

#[cfg(test)]
mod tests;
