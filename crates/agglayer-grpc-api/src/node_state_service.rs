use std::sync::Arc;

use agglayer_grpc_server::node::v1::node_state_service_server::NodeStateService;
use agglayer_grpc_types::node::v1::{
    GetCertificateHeaderErrorKind, GetCertificateHeaderRequest, GetCertificateHeaderResponse,
    GetLatestCertificateHeaderErrorKind, GetLatestCertificateHeaderRequest,
    GetLatestCertificateHeaderResponse, LatestCertificateRequestType,
};
use agglayer_rpc::AgglayerService;
use agglayer_storage::stores::{DebugReader, PendingCertificateReader, StateReader};
use axum::body::Bytes;
use tonic_types::{ErrorDetails, StatusExt as _};
use tracing::warn;

const GET_CERTIFICATE_HEADER_METHOD_PATH: &str =
    "agglayer-node.grpc-api.v1.node-state-service.get_certificate_header";
const GET_LATEST_CERTIFICATE_HEADER_METHOD_PATH: &str =
    "agglayer-node.grpc-api.v1.node-state-service.get_latest_certificate_header";

pub struct NodeStateServer<L1Rpc, PendingStore, StateStore, DebugStore> {
    pub(crate) service: Arc<AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore>>,
}

#[tonic::async_trait]
impl<L1Rpc, PendingStore, StateStore, DebugStore> NodeStateService
    for NodeStateServer<L1Rpc, PendingStore, StateStore, DebugStore>
where
    PendingStore: PendingCertificateReader + 'static,
    StateStore: StateReader + 'static,
    DebugStore: DebugReader + 'static,
    L1Rpc: Send + Sync + 'static,
{
    #[tracing::instrument(level = "debug", skip(self, request), fields(request_id = tracing::field::Empty))]
    async fn get_certificate_header(
        &self,
        request: tonic::Request<GetCertificateHeaderRequest>,
    ) -> Result<tonic::Response<GetCertificateHeaderResponse>, tonic::Status> {
        let request_id = uuid::Uuid::new_v4().to_string();
        tracing::Span::current().record("request_id", &request_id);
        let request = request.into_inner();

        let certificate_id: agglayer_types::CertificateId = match request.certificate_id {
            None => {
                let error_kind = GetCertificateHeaderErrorKind::MissingCertificateId;
                let mut error_details = ErrorDetails::with_error_info(
                    error_kind.as_str_name(),
                    GET_CERTIFICATE_HEADER_METHOD_PATH,
                    [],
                );

                error_details.add_bad_request_violation("certificate_id", error_kind);

                Err(error_details)
            }
            Some(value) => {
                value
                    .try_into()
                    .map_err(|error: agglayer_grpc_types::compat::v1::Error| {
                        let mut error_details = ErrorDetails::with_error_info(
                            GetCertificateHeaderErrorKind::from(&error).as_str_name(),
                            GET_CERTIFICATE_HEADER_METHOD_PATH,
                            [("error".to_string(), format!("{error:?}"))],
                        );

                        error_details.set_bad_request(Vec::from(&error));

                        error_details
                    })
            }
        }
        .map_err(|error_details| {
            tonic::Status::with_error_details(
                tonic::Code::InvalidArgument,
                "Invalid certificate ID",
                error_details,
            )
        })?;

        match self.service.fetch_certificate_header(certificate_id) {
            Err(agglayer_rpc::CertificateRetrievalError::Storage(error)) => {
                Err(tonic::Status::with_error_details(
                    tonic::Code::Internal,
                    "Internal error",
                    ErrorDetails::with_error_info(
                        GetCertificateHeaderErrorKind::Internal.as_str_name(),
                        GET_CERTIFICATE_HEADER_METHOD_PATH,
                        [("error_message".into(), error.to_string())],
                    ),
                ))
            }
            Err(agglayer_rpc::CertificateRetrievalError::NotFound { .. }) => {
                Err(tonic::Status::with_details(
                    tonic::Code::NotFound,
                    "Certificate not found",
                    Bytes::new(),
                ))
            }
            Ok(header) => Ok(tonic::Response::new(GetCertificateHeaderResponse {
                certificate_header: Some(header.into()),
            })),
        }
    }

    #[tracing::instrument(level = "debug", skip(self, request), fields(request_id = tracing::field::Empty))]
    async fn get_latest_certificate_header(
        &self,
        request: tonic::Request<GetLatestCertificateHeaderRequest>,
    ) -> Result<tonic::Response<GetLatestCertificateHeaderResponse>, tonic::Status> {
        let request_id = uuid::Uuid::new_v4().to_string();
        tracing::Span::current().record("request_id", &request_id);

        let request = request.into_inner();
        let network_id = request.network_id;

        let request_type =
            LatestCertificateRequestType::try_from(request.r#type).map_err(|_| {
                let kind = GetLatestCertificateHeaderErrorKind::InvalidRequestType;
                let mut error_details = ErrorDetails::with_error_info(
                    kind.as_str_name(),
                    GET_LATEST_CERTIFICATE_HEADER_METHOD_PATH,
                    [],
                );

                error_details.add_bad_request_violation("type", kind);

                tonic::Status::with_error_details(tonic::Code::InvalidArgument, kind, error_details)
            })?;

        let result = match request_type {
            LatestCertificateRequestType::Pending => self
                .service
                .get_latest_pending_certificate_header(network_id.into()),
            LatestCertificateRequestType::Settled => self
                .service
                .get_latest_settled_certificate_header(network_id.into()),
            _ => {
                warn!(
                    "Invalid request type should have been handled earlier: {:?}",
                    request_type
                );

                return Err(tonic::Status::new(
                    tonic::Code::InvalidArgument,
                    "Invalid request type",
                ));
            }
        };

        let header = result.map_err(|error| match error {
            agglayer_rpc::CertificateRetrievalError::Storage(error) => {
                tonic::Status::with_error_details(
                    tonic::Code::Internal,
                    "Internal error",
                    ErrorDetails::with_error_info(
                        GetCertificateHeaderErrorKind::Internal.as_str_name(),
                        GET_LATEST_CERTIFICATE_HEADER_METHOD_PATH,
                        [("error".into(), format!("{error:?}"))],
                    ),
                )
            }
            agglayer_rpc::CertificateRetrievalError::NotFound { .. } => {
                tonic::Status::new(tonic::Code::NotFound, "Certificate not found")
            }
        })?;

        Ok(tonic::Response::new(GetLatestCertificateHeaderResponse {
            certificate_header: header.map(Into::into),
        }))
    }
}
