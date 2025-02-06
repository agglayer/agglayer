/// Definitions of the certificate storage format with backwards compatibility.
///
/// Currently, we have two versions of certificate storage format.
/// All begin with `network_id: 32`, followed by a 64-bit field which encodes
/// certificate height and/or format version. In the V0 format, this field
/// corresponds to certificate height, providing backwards compatibility.
///
/// If the field is < [TAG_BEGIN], it is interpreted as height in storage format V0.
/// If the field is >= [TAG_BEGIN], it is interpreted as an identifier of the storage
/// format version. For versions > 0, the format is encoded in the former height field
/// as `TAG_BEGIN + FORMAT_VERSION`. In this case, the height has to be stored elsewhere.

use agglayer_types::{Certificate, Digest, Height, Metadata, NetworkId, Signature};
use bincode::Options;
use pessimistic_proof::{
    aggchain_proof::{AggchainProof, AggchainProofSP1},
    bridge_exit::BridgeExit,
    imported_bridge_exit::ImportedBridgeExit,
};
use serde::{Deserialize, Serialize};

use crate::columns::default_bincode_options;

const TAG_BEGIN: u64 = 1u64 << 63;

/// A height field restricted to values < [TAG_BEGIN].
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
#[serde(try_from = "Height")]
struct V0Height(Height);

impl V0Height {
    const ALLOWED_RANGE: std::ops::Range<Height> = 0..TAG_BEGIN;

    const fn as_height(&self) -> Height {
        self.0
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid certificate height ({0})")]
struct InvalidHeight(Height);

impl TryFrom<Height> for V0Height {
    type Error = InvalidHeight;

    fn try_from(height: Height) -> Result<Self, Self::Error> {
        Some(Self(height))
            .filter(|this| Self::ALLOWED_RANGE.contains(&this.0))
            .ok_or(InvalidHeight(height))
    }
}

/// A unit type representing a format version.
///
/// Encodes as a u64 [Self::TAG]. Decodes successfully only if a u64 value matches [Self::TAG].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(try_from = "Height", into = "u64")]
struct VersionTag<const VERSION: u64>;

impl<const V: u64> VersionTag<V> {
    const VERSION: u64 = V;
    const TAG: u64 = TAG_BEGIN + Self::VERSION;
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid version tag ({0})")]
struct InvalidVersionTag(u64);

impl<const V: u64> TryFrom<Height> for VersionTag<V> {
    type Error = InvalidVersionTag;

    fn try_from(height: u64) -> Result<Self, Self::Error> {
        (height == Self::TAG)
            .then_some(Self)
            .ok_or(InvalidVersionTag(height))
    }
}

impl<const V: u64> From<VersionTag<V>> for u64 {
    fn from(VersionTag: VersionTag<V>) -> Self {
        <VersionTag<V>>::TAG
    }
}

/// The pre-0.3 certificate format.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
struct CertificateV0 {
    network_id: NetworkId,
    height: V0Height,
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
            network_id,
            height: height.as_height(),
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
    network_id: NetworkId,
    version: VersionTag<1>,
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
            network_id,
            version: VersionTag,
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
            network_id: *network_id,
            version: VersionTag,
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

impl crate::columns::Codec for Certificate {
    fn encode(&self) -> Result<Vec<u8>, crate::columns::CodecError> {
        Ok(default_bincode_options().serialize(&CurrentCertificate::from(self))?)
    }

    fn decode(bytes: &[u8]) -> Result<Self, crate::columns::CodecError> {
        let certificate = default_bincode_options()
            .deserialize(bytes)
            .map(CertificateV1::into)
            .or_else(|_| {
                default_bincode_options()
                    .deserialize(bytes)
                    .map(CertificateV0::into)
            })?;
        Ok(certificate)
    }
}

#[cfg(test)]
mod tests;
