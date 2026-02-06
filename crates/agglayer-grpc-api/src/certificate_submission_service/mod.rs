use std::sync::Arc;

use agglayer_contracts::{AggchainContract, L1TransactionFetcher, RollupContract};
use agglayer_grpc_server::node::v1::certificate_submission_service_server::CertificateSubmissionService;
use agglayer_grpc_types::node::v1::{
    SubmitCertificateErrorKind, SubmitCertificateRequest, SubmitCertificateResponse,
};
use agglayer_rpc::AgglayerService;
use agglayer_storage::stores::{
    DebugReader, DebugWriter, EpochStoreReader, PendingCertificateReader, PendingCertificateWriter,
    StateReader, StateWriter,
};
use error::CertificateSubmissionErrorWrapper;
use tonic_types::{ErrorDetails, StatusExt};
use tracing::instrument;

const SUBMIT_CERTIFICATE_METHOD_PATH: &str =
    "agglayer-node.grpc-api.v1.certificate-submission-service.submit_certificate";

pub struct CertificateSubmissionServer<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore> {
    pub(crate) service:
        Arc<AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore>>,
}

mod error;

#[tonic::async_trait]
impl<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore> CertificateSubmissionService
    for CertificateSubmissionServer<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore>
where
    PendingStore: PendingCertificateReader + PendingCertificateWriter + 'static,
    StateStore: StateReader + StateWriter + 'static,
    DebugStore: DebugReader + DebugWriter + 'static,
    L1Rpc: RollupContract + AggchainContract + L1TransactionFetcher + Send + Sync + 'static,
    EpochsStore: EpochStoreReader + 'static,
{
    #[instrument(skip(self, request), level = "debug", fields(
        certificate_id = tracing::field::Empty,
        client = crate::client_info_from_metadata(request.metadata())
    ))]
    async fn submit_certificate(
        &self,
        request: tonic::Request<SubmitCertificateRequest>,
    ) -> Result<tonic::Response<SubmitCertificateResponse>, tonic::Status> {
        let certificate: agglayer_types::Certificate = match request.into_inner().certificate {
            Some(certificate) => certificate.try_into().map_err(
                |error: agglayer_grpc_types::compat::v1::Error| {
                    tonic::Status::with_error_details(
                        tonic::Code::InvalidArgument,
                        "Invalid certificate",
                        ErrorDetails::with_error_info(
                            SubmitCertificateErrorKind::from(error.kind()),
                            SUBMIT_CERTIFICATE_METHOD_PATH,
                            [("error".into(), format!("{error:?}"))],
                        ),
                    )
                },
            )?,
            None => return Err(tonic::Status::invalid_argument("Missing certificate")),
        };

        let certificate_id = self
            .service
            .send_certificate(certificate)
            .await
            .map_err(|error| {
                CertificateSubmissionErrorWrapper::new(error, SUBMIT_CERTIFICATE_METHOD_PATH)
            })?;

        tracing::Span::current().record("certificate_id", certificate_id.to_string());

        Ok(tonic::Response::new(SubmitCertificateResponse {
            certificate_id: Some(certificate_id.into()),
        }))
    }
}
