use std::collections::HashMap;

use agglayer_grpc_types::node::v1::CertificateSubmissionErrorKind;
use agglayer_rpc::CertificateSubmissionError;
use ethers::providers::Middleware;
use tonic_types::ErrorDetails;
use tonic_types::StatusExt as _;

pub(crate) struct CertificateSubmissionErrorWrapper<Rpc: Middleware> {
    inner: CertificateSubmissionError<Rpc>,
    context: &'static str,
}

impl<Rpc> CertificateSubmissionErrorWrapper<Rpc>
where
    Rpc: Middleware,
{
    pub(crate) fn new(inner: CertificateSubmissionError<Rpc>, context: &'static str) -> Self {
        Self { inner, context }
    }
}

impl<Rpc> From<CertificateSubmissionErrorWrapper<Rpc>> for tonic::Status
where
    Rpc: Middleware,
{
    fn from(error: CertificateSubmissionErrorWrapper<Rpc>) -> Self {
        match error.inner {
            agglayer_rpc::CertificateSubmissionError::Storage(_) => {
                tonic::Status::internal("Internal storage error")
            }

            agglayer_rpc::CertificateSubmissionError::OrchestratorNotResponsive => {
                tonic::Status::internal("Orchestrator not responsive")
            }

            agglayer_rpc::CertificateSubmissionError::SignatureError(
                signature_verification_error,
            ) => tonic::Status::with_error_details(
                tonic::Code::InvalidArgument,
                "Signature verification error",
                ErrorDetails::with_error_info(
                    CertificateSubmissionErrorKind::SignatureVerification.as_str_name(),
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
                        CertificateSubmissionErrorKind::UnableToReplacePendingCertificate
                            .as_str_name(),
                        error.context,
                        details,
                    ),
                )
            }
        }
    }
}
