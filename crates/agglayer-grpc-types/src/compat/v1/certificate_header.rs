use agglayer_types::{CertificateHeader, CertificateStatus, CertificateStatusError};

use crate::protocol::types::v1;

impl From<CertificateStatusError> for v1::CertificateStatusError {
    fn from(value: CertificateStatusError) -> Self {
        v1::CertificateStatusError {
            // Display value with the whole stack trace
            message: format!("{value:?}").into_bytes().into(),
        }
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
