use agglayer_types::{
    CertificateHeader, CertificateId, CertificateIndex, CertificateStatus, Digest, EpochNumber,
    Height, Metadata, NetworkId,
};
use bincode::Options as _;
use serde::Deserialize;

use crate::columns::{default_bincode_options, CodecError};

/// The pre-0.3 certificate format (`v0`).
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
struct CertificateHeaderV0 {
    network_id: NetworkId,
    height: Height,
    epoch_number: Option<EpochNumber>,
    certificate_index: Option<CertificateIndex>,
    certificate_id: CertificateId,
    prev_local_exit_root: Digest,
    new_local_exit_root: Digest,
    metadata: Metadata,
    status: CertificateStatus,
    settlement_tx_hash: Option<Digest>,
}

impl From<CertificateHeaderV0> for CertificateHeader {
    fn from(certificate: CertificateHeaderV0) -> Self {
        let CertificateHeaderV0 {
            network_id,
            height,
            epoch_number,
            certificate_index,
            certificate_id,
            prev_local_exit_root,
            new_local_exit_root,
            metadata,
            status,
            settlement_tx_hash,
        } = certificate;

        CertificateHeader {
            network_id,
            height,
            epoch_number,
            certificate_index,
            certificate_id,
            prev_local_exit_root,
            new_local_exit_root,
            metadata,
            status,
            settlement_tx_hash,
        }
    }
}

fn decode<T: for<'de> Deserialize<'de> + Into<CertificateHeader>>(
    bytes: &[u8],
) -> Result<CertificateHeader, CodecError> {
    Ok(default_bincode_options().deserialize::<T>(bytes)?.into())
}

impl crate::columns::Codec for CertificateHeader {
    fn encode(&self) -> Result<Vec<u8>, CodecError> {
        todo!()
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        match bytes.first().copied() {
            None => Err(CodecError::CertificateEmpty),
            Some(0) => decode::<CertificateHeaderV0>(bytes),
            Some(version) => Err(CodecError::BadCertificateVersion { version }),
        }
    }
}
