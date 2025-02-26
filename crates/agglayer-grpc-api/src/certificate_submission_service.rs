use std::collections::HashMap;
use std::sync::Arc;

use agglayer_contracts::L1TransactionFetcher;
use agglayer_contracts::RollupContract;
use agglayer_grpc_server::node::v1::certificate_submission_service_server::CertificateSubmissionService;
use agglayer_grpc_types::compat::v1::Error;
use agglayer_grpc_types::node::v1::{
    CertificateSubmissionErrorKind, SubmitCertificateRequest, SubmitCertificateResponse,
};
use agglayer_rpc::AgglayerService;
use agglayer_storage::stores::{
    DebugReader, DebugWriter, PendingCertificateReader, PendingCertificateWriter, StateReader,
    StateWriter,
};
use tonic_types::ErrorDetails;
use tonic_types::StatusExt as _;
use tracing::instrument;

const SERVICE_PATH: &str = "agglayer-node.grpc-api.v1.certificate-submission-service";

pub struct CertificateSubmissionServer<L1Rpc, PendingStore, StateStore, DebugStore> {
    pub(crate) service: Arc<AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore>>,
}

#[tonic::async_trait]
impl<L1Rpc, PendingStore, StateStore, DebugStore> CertificateSubmissionService
    for CertificateSubmissionServer<L1Rpc, PendingStore, StateStore, DebugStore>
where
    PendingStore: PendingCertificateReader + PendingCertificateWriter + 'static,
    StateStore: StateReader + StateWriter + 'static,
    DebugStore: DebugReader + DebugWriter + 'static,
    L1Rpc: RollupContract + L1TransactionFetcher + Send + Sync + 'static,
{
    #[instrument(skip(self), level = "debug", fields(certificate_id = tracing::field::Empty))]
    async fn submit_certificate(
        &self,
        request: tonic::Request<SubmitCertificateRequest>,
    ) -> Result<tonic::Response<SubmitCertificateResponse>, tonic::Status> {
        let context = format!("{}.{}", SERVICE_PATH, "submit-certificate");
        let certificate: agglayer_types::Certificate = match request.into_inner().certificate {
            Some(certificate) => certificate.try_into().map_err(|error| {
                // TODO: We can't define which field failed
                match &error {
                    Error::WrongBytesLength { .. } => {}
                    Error::WrongVectorLength { .. } => {}
                    Error::MissingField(_field) => {}
                    Error::InvalidLeafType(_leaf_type) => {}
                    Error::InvalidCertificateStatus(_status) => {}
                    Error::ParsingField(_field, _source) => {}
                    Error::ParsingSignature(_source) => {}
                    Error::DeserializingProof(_source) => {}
                    Error::SerializingProof(_source) => {}
                }

                tonic::Status::invalid_argument(format!("Invalid certificate: {}", error))
            })?,
            None => return Err(tonic::Status::invalid_argument("Missing certificate")),
        };

        let certificate_id = certificate.hash();
        let certificate_id = self
            .service
            .send_certificate(certificate)
            .await
            .map_err(|error| match error {
                agglayer_rpc::CertificateSubmissionError::Storage(_) => {
                    tonic::Status::internal("Internal storage error")
                }
                agglayer_rpc::CertificateSubmissionError::OrchestratorNotResponsive => {
                    tonic::Status::internal("Orchestrator not responsive")
                }
                agglayer_rpc::CertificateSubmissionError::SignatureError(
                    signature_verification_error,
                ) => {
                    let mut error_details = ErrorDetails::new();

                    error_details.set_error_info(
                        CertificateSubmissionErrorKind::SignatureVerification.as_str_name(),
                        &context,
                        [
                            ("certificate_id".into(), certificate_id.to_string()),
                            ("source".into(), signature_verification_error.to_string()),
                        ],
                    );

                    tonic::Status::with_error_details(
                        tonic::Code::InvalidArgument,
                        "Signature verification error",
                        error_details,
                    )
                }
                agglayer_rpc::CertificateSubmissionError::UnableToReplacePendingCertificate {
                    reason,
                    height,
                    network_id,
                    stored_certificate_id,
                    replacement_certificate_id,
                    source,
                } => {
                    let mut error_details = ErrorDetails::new();

                    let mut details: HashMap<String, String> = vec![
                        ("reason".into(), reason.to_string()),
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
                        details.insert("source".into(), source.to_string());
                    }

                    error_details.set_error_info(
                        CertificateSubmissionErrorKind::UnableToReplacePendingCertificate
                            .as_str_name(),
                        &context,
                        details,
                    );

                    tonic::Status::with_error_details(
                        tonic::Code::InvalidArgument,
                        "Unable to replace pending certificate",
                        error_details,
                    )
                }
            })?;

        Ok(tonic::Response::new(SubmitCertificateResponse {
            certificate_id: Some(certificate_id.into()),
        }))
    }
}
