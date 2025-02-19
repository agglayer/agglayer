use agglayer_types::{CertificateHeader, CertificateStatus, CertificateStatusError, NetworkId};

use super::Error;
use crate::protocol::types::v1;

impl TryFrom<v1::CertificateStatusError> for CertificateStatusError {
    type Error = Error;

    fn try_from(value: v1::CertificateStatusError) -> Result<Self, Self::Error> {
        Ok(CertificateStatusError::Message(
            String::from_utf8_lossy(&value.message).into(),
        ))
    }
}

impl From<CertificateStatusError> for v1::CertificateStatusError {
    fn from(value: CertificateStatusError) -> Self {
        v1::CertificateStatusError {
            // Display value with the whole stack trace
            message: format!("{value:?}").into_bytes().into(),
        }
    }
}

impl TryFrom<v1::CertificateHeader> for CertificateHeader {
    type Error = Error;

    fn try_from(value: v1::CertificateHeader) -> Result<Self, Self::Error> {
        let status = match value.status() {
            v1::CertificateStatus::Pending => CertificateStatus::Pending,
            v1::CertificateStatus::Proven => CertificateStatus::Proven,
            v1::CertificateStatus::Candidate => CertificateStatus::Candidate,
            v1::CertificateStatus::InError => CertificateStatus::InError {
                error: required_field!(value, error),
            },
            v1::CertificateStatus::Settled => CertificateStatus::Settled,
            _ => return Err(Error::InvalidCertificateStatus(value.status)),
        };
        Ok(CertificateHeader {
            network_id: NetworkId::new(value.network_id),
            height: value.height,
            epoch_number: value.epoch_number,
            certificate_index: value.certificate_index,
            certificate_id: required_field!(value, certificate_id),
            prev_local_exit_root: required_field!(value, prev_local_exit_root),
            new_local_exit_root: required_field!(value, new_local_exit_root),
            metadata: required_field!(value, metadata),
            status,
            settlement_tx_hash: value
                .settlement_tx_hash
                .map(TryInto::try_into)
                .transpose()
                .map_err(|e| Error::ParsingField("settlement_tx_hash", Box::new(e)))?,
        })
    }
}

impl From<CertificateHeader> for v1::CertificateHeader {
    fn from(value: CertificateHeader) -> Self {
        let (status, error) = match value.status {
            CertificateStatus::Pending => (v1::CertificateStatus::Pending, None),
            CertificateStatus::Proven => (v1::CertificateStatus::Proven, None),
            CertificateStatus::Candidate => (v1::CertificateStatus::Candidate, None),
            CertificateStatus::InError { error } => {
                (v1::CertificateStatus::InError, Some(error.into()))
            }
            CertificateStatus::Settled => (v1::CertificateStatus::Settled, None),
        };
        v1::CertificateHeader {
            network_id: value.network_id.into(),
            height: value.height,
            epoch_number: value.epoch_number,
            certificate_index: value.certificate_index,
            certificate_id: Some(value.certificate_id.into()),
            prev_local_exit_root: Some(value.prev_local_exit_root.into()),
            new_local_exit_root: Some(value.new_local_exit_root.into()),
            metadata: Some(value.metadata.into()),
            status: status.into(),
            error,
            settlement_tx_hash: value.settlement_tx_hash.map(Into::into),
        }
    }
}
