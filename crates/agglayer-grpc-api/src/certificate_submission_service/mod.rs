use std::sync::Arc;

use agglayer_contracts::L1TransactionFetcher;
use agglayer_contracts::RollupContract;
use agglayer_grpc_server::node::v1::certificate_submission_service_server::CertificateSubmissionService;
use agglayer_grpc_types::compat::v1::Error;
use agglayer_grpc_types::node::v1::{SubmitCertificateRequest, SubmitCertificateResponse};
use agglayer_rpc::AgglayerService;
use agglayer_storage::stores::{
    DebugReader, DebugWriter, PendingCertificateReader, PendingCertificateWriter, StateReader,
    StateWriter,
};
use error::CertificateSubmissionErrorWrapper;
use tracing::instrument;

const SUBMIT_CERTIFICATE_METHOD_PATH: &str =
    "agglayer-node.grpc-api.v1.certificate-submission-service.submit_certificate";

pub struct CertificateSubmissionServer<L1Rpc, PendingStore, StateStore, DebugStore> {
    pub(crate) service: Arc<AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore>>,
}

mod error;

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

        let certificate_id = self
            .service
            .send_certificate(certificate)
            .await
            .map_err(|error| {
                CertificateSubmissionErrorWrapper::new(error, SUBMIT_CERTIFICATE_METHOD_PATH)
            })?;

        Ok(tonic::Response::new(SubmitCertificateResponse {
            certificate_id: Some(certificate_id.into()),
        }))
    }
}
