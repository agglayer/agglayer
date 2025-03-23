use agglayer_types::{CertificateHeader, CertificateStatus, CertificateStatusError};

use crate::node::v1;

impl From<CertificateStatusError> for v1::types::CertificateStatusError {
    fn from(value: CertificateStatusError) -> Self {
        v1::types::CertificateStatusError {
            // Display value with the whole stack trace
            message: format!("{value:?}").into_bytes().into(),
        }
    }
}

impl From<CertificateHeader> for v1::types::CertificateHeader {
    fn from(value: CertificateHeader) -> Self {
        let (status, error) = match value.status {
            CertificateStatus::Pending => (v1::types::CertificateStatus::Pending, None),
            CertificateStatus::Proven => (v1::types::CertificateStatus::Proven, None),
            CertificateStatus::Candidate => (v1::types::CertificateStatus::Candidate, None),
            CertificateStatus::InError { error } => {
                (v1::types::CertificateStatus::InError, Some(error.into()))
            }
            CertificateStatus::Settled => (v1::types::CertificateStatus::Settled, None),
        };
        v1::types::CertificateHeader {
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
