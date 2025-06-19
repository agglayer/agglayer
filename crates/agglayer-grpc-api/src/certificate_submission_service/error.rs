use std::collections::HashMap;

use agglayer_grpc_types::node::v1::SubmitCertificateErrorKind;
use agglayer_rpc::CertificateSubmissionError;
use tonic_types::{ErrorDetails, StatusExt as _};
use tracing::{error, warn};

pub(crate) struct CertificateSubmissionErrorWrapper {
    inner: CertificateSubmissionError,
    context: &'static str,
}

impl CertificateSubmissionErrorWrapper {
    pub(crate) fn new(inner: CertificateSubmissionError, context: &'static str) -> Self {
        Self { inner, context }
    }
}

impl From<CertificateSubmissionErrorWrapper> for tonic::Status {
    fn from(error: CertificateSubmissionErrorWrapper) -> Self {
        match error.inner {
            agglayer_rpc::CertificateSubmissionError::Storage(error) => {
                error!(?error, "returning internal storage error to RPC");
                tonic::Status::internal("Internal storage error")
            }

            agglayer_rpc::CertificateSubmissionError::OrchestratorNotResponsive => {
                warn!("returning orchestrator not responsive to RPC");
                tonic::Status::internal("Orchestrator not responsive")
            }

            agglayer_rpc::CertificateSubmissionError::SignatureError(
                signature_verification_error,
            ) => tonic::Status::with_error_details(
                tonic::Code::InvalidArgument,
                "Signature verification error",
                ErrorDetails::with_error_info(
                    SubmitCertificateErrorKind::SignatureVerification.as_str_name(),
                    error.context,
                    [("error".into(), format!("{signature_verification_error:?}"))],
                ),
            ),

            agglayer_rpc::CertificateSubmissionError::UnableToReplacePendingCertificate {
                reason,
                height,
                network_id,
                stored_certificate_id,
                replacement_certificate_id,
                source,
            } => {
                let mut details: HashMap<String, String> = vec![
                    ("reason".into(), reason),
                    ("height".into(), height.to_string()),
                    ("network_id".into(), network_id.to_string()),
                    (
                        "stored_certificate_id".into(),
                        stored_certificate_id.to_string(),
                    ),
                    (
                        "replacement_certificate_id".into(),
                        replacement_certificate_id.to_string(),
                    ),
                ]
                .into_iter()
                .collect();

                if let Some(source) = source {
                    details.insert("error".into(), format!("{source:?}"));
                }

                tonic::Status::with_error_details(
                    tonic::Code::InvalidArgument,
                    "Unable to replace pending certificate",
                    ErrorDetails::with_error_info(
                        SubmitCertificateErrorKind::UnableToReplacePendingCertificate.as_str_name(),
                        error.context,
                        details,
                    ),
                )
            }
        }
    }
}
