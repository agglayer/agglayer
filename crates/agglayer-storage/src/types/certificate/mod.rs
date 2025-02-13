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

/// Magic number to start the encoding for storage format versions >= 1.
///
/// If the first 4 bytes are not this magic value, the byte string
/// is interpreted as version 0 and the first 4 bytes correspond to the `NetworkId`.
const MAGIC: [u8; 4] = 0xffff_ffff_u32.to_be_bytes();

/// The pre-0.3 certificate format.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
struct CertificateV0 {
    network_id: NetworkId,
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

/// Uninhabited type to selectively disable enum arms.
#[derive(Serialize, Deserialize, Clone, Debug)]
enum Impossible {}

/// Defines encoding for certificates version >= 1.
///
/// The enum implicitly introduces the version tag in the encoding.
#[derive(Serialize, Deserialize, Clone, Debug)]
enum CertificateVx {
    // Reserve the tag 0, since version 0 is encoded differently.
    V0(Impossible),

    V1(CertificateV1),
}

impl From<&Certificate> for CertificateVx {
    fn from(certificate: &Certificate) -> Self {
        // Always use the latest version here.
        Self::V1(certificate.into())
    }
}

impl From<CertificateVx> for Certificate {
    fn from(certificate: CertificateVx) -> Self {
        match certificate {
            CertificateVx::V0(impossible) => match impossible {},
            CertificateVx::V1(certificate) => certificate.into(),
        }
    }
}

impl crate::columns::Codec for Certificate {
    fn encode(&self) -> Result<Vec<u8>, CodecError> {
        Ok(default_bincode_options().serialize(&(MAGIC, CertificateVx::from(self)))?)
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        let (magic_or_version, more_bytes) =
            bytes.split_at_checked(MAGIC.len()).ok_or(CodecError::NoMagic)?;

        if magic_or_version == MAGIC.as_slice() {
            Ok(default_bincode_options().deserialize::<CertificateVx>(more_bytes)?.into())
        } else {
            Ok(default_bincode_options().deserialize::<CertificateV0>(bytes)?.into())
        }
    }
}

#[cfg(test)]
mod tests;
