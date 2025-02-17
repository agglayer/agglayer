use agglayer_grpc_server::node::v1::certificate_submission_service_server::CertificateSubmissionService;
use agglayer_grpc_types::node::v1::{
    ErrorKind, SubmitCertificateRequest, SubmitCertificateResponse,
};
use agglayer_types::Digest;
use tonic_types::{ErrorDetails, StatusExt};
use tracing::instrument;

const SERVICE_PATH: &str = "agglayer-node.grpc-api.v1.configuration-service";

pub struct CertificateSubmissionServer {}

#[tonic::async_trait]
impl CertificateSubmissionService for CertificateSubmissionServer {
    #[instrument(skip(self), level = "debug", fields(certificate_id = tracing::field::Empty))]
    async fn submit_certificate(
        &self,
        request: tonic::Request<SubmitCertificateRequest>,
    ) -> Result<tonic::Response<SubmitCertificateResponse>, tonic::Status> {
        let _certificate = request.into_inner().certificate.unwrap();
        let mut error_details = ErrorDetails::new();
        let context = format!("{}.{}", SERVICE_PATH, "submit-certificate");

        error_details.set_error_info(
            ErrorKind::SignatureVerification.as_str_name(),
            context,
            [
                ("certificate_id".into(), Digest([1; 32]).to_string()),
                ("network_id".into(), 1.to_string()),
            ],
        );

        let status = tonic::Status::with_error_details(
            tonic::Code::InvalidArgument,
            "An invalid certificate was submitted.",
            error_details,
        );

        Err(status)
    }
}
