use std::sync::Arc;

use agglayer_grpc_server::node::v1::node_state_service_server::NodeStateService;
use agglayer_grpc_types::{
    compat::v1::Error,
    node::v1::{
        GetCertificateHeaderErrorKind, GetCertificateHeaderRequest, GetCertificateHeaderResponse,
        GetLatestCertificateHeaderErrorKind, GetLatestCertificateHeaderRequest,
        GetLatestCertificateHeaderResponse, GetNetworkInfoErrorKind, GetNetworkInfoRequest,
        GetNetworkInfoResponse, LatestCertificateRequestType,
    },
};
use agglayer_rpc::AgglayerService;
use agglayer_storage::stores::{
    DebugReader, EpochStoreReader, NetworkInfoReader, PendingCertificateReader, StateReader,
};
use tonic_types::{ErrorDetails, StatusExt as _};
use tracing::error;

const GET_CERTIFICATE_HEADER_METHOD_PATH: &str =
    "agglayer-node.grpc-api.v1.node-state-service.get_certificate_header";
const GET_LATEST_CERTIFICATE_HEADER_METHOD_PATH: &str =
    "agglayer-node.grpc-api.v1.node-state-service.get_latest_certificate_header";
const GET_NETWORK_INFO_METHOD_PATH: &str =
    "agglayer-node.grpc-api.v1.node-state-service.get_network_info";

pub struct NodeStateServer<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore> {
    pub(crate) service:
        Arc<AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore>>,
}

#[tonic::async_trait]
impl<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore> NodeStateService
    for NodeStateServer<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore>
where
    PendingStore: PendingCertificateReader + 'static,
    StateStore: NetworkInfoReader + StateReader + 'static,
    DebugStore: DebugReader + 'static,
    L1Rpc: Send + Sync + 'static,
    EpochsStore: EpochStoreReader + 'static,
{
    #[tracing::instrument(level = "debug", skip(self, request), fields(request_id = tracing::field::Empty, client_info = tracing::field::Empty))]
    async fn get_certificate_header(
        &self,
        request: tonic::Request<GetCertificateHeaderRequest>,
    ) -> Result<tonic::Response<GetCertificateHeaderResponse>, tonic::Status> {
        let client_info = crate::client_info_from_metadata(request.metadata());
        tracing::Span::current().record("client_info", &client_info);

        let request_id = uuid::Uuid::new_v4().to_string();
        tracing::Span::current().record("request_id", &request_id);
        let request = request.into_inner();

        let certificate_id: agglayer_types::CertificateId = request
            .certificate_id
            .ok_or_else(|| Error::missing_field("certificate_id"))
            .and_then(|c| c.try_into())
            .map_err(|error| {
                tonic::Status::with_error_details(
                    tonic::Code::InvalidArgument,
                    "Invalid certificate ID",
                    ErrorDetails::with_error_info(
                        GetCertificateHeaderErrorKind::from(error.kind()).as_str_name(),
                        GET_CERTIFICATE_HEADER_METHOD_PATH,
                        [("error".into(), format!("{error:?}"))],
                    ),
                )
            })?;

        match self.service.fetch_certificate_header(certificate_id) {
            Err(agglayer_rpc::CertificateRetrievalError::Storage(error)) => {
                error!(?error, "returning internal storage error to RPC");
                Err(tonic::Status::internal("Internal storage error"))
            }
            Err(agglayer_rpc::CertificateRetrievalError::NotFound { .. }) => {
                Err(tonic::Status::with_error_details(
                    tonic::Code::NotFound,
                    "Certificate not found",
                    ErrorDetails::with_error_info(
                        GetCertificateHeaderErrorKind::NotFound.as_str_name(),
                        GET_CERTIFICATE_HEADER_METHOD_PATH,
                        [],
                    ),
                ))
            }
            Ok(header) => Ok(tonic::Response::new(GetCertificateHeaderResponse {
                certificate_header: Some(header.into()),
            })),
        }
    }

    #[tracing::instrument(level = "debug", skip(self, request), fields(request_id = tracing::field::Empty,  client_info = tracing::field::Empty))]
    async fn get_latest_certificate_header(
        &self,
        request: tonic::Request<GetLatestCertificateHeaderRequest>,
    ) -> Result<tonic::Response<GetLatestCertificateHeaderResponse>, tonic::Status> {
        let client_info = crate::client_info_from_metadata(request.metadata());
        tracing::Span::current().record("client_info", &client_info);

        let request_id = uuid::Uuid::new_v4().to_string();
        tracing::Span::current().record("request_id", &request_id);

        let request = request.into_inner();
        let network_id = request.network_id;

        let result = match request.r#type() {
            LatestCertificateRequestType::Pending => self
                .service
                .get_latest_pending_certificate_header(network_id.into()),
            LatestCertificateRequestType::Settled => self
                .service
                .get_latest_settled_certificate_header(network_id.into()),
            LatestCertificateRequestType::Unspecified => {
                let error =
                    Error::invalid_data("invalid request type".to_owned()).inside_field("type");
                let kind = GetLatestCertificateHeaderErrorKind::from(error.kind());
                return Err(tonic::Status::with_error_details(
                    tonic::Code::InvalidArgument,
                    kind,
                    ErrorDetails::with_error_info(
                        kind.as_str_name(),
                        GET_LATEST_CERTIFICATE_HEADER_METHOD_PATH,
                        [("error".into(), format!("{error:?}"))],
                    ),
                ));
            }
        };

        let header = result.map_err(|error| match error {
            agglayer_rpc::CertificateRetrievalError::Storage(error) => {
                error!(?error, "returning internal storage error to RPC");
                tonic::Status::internal("Internal storage error")
            }
            agglayer_rpc::CertificateRetrievalError::NotFound { .. } => {
                tonic::Status::with_error_details(
                    tonic::Code::NotFound,
                    "Certificate not found",
                    ErrorDetails::with_error_info(
                        GetLatestCertificateHeaderErrorKind::NotFound.as_str_name(),
                        GET_LATEST_CERTIFICATE_HEADER_METHOD_PATH,
                        [],
                    ),
                )
            }
        })?;

        Ok(tonic::Response::new(GetLatestCertificateHeaderResponse {
            certificate_header: header.map(Into::into),
        }))
    }

    #[tracing::instrument(level = "debug", skip(self, request), fields(request_id = tracing::field::Empty, client_info = tracing::field::Empty))]
    async fn get_network_info(
        &self,
        request: tonic::Request<GetNetworkInfoRequest>,
    ) -> Result<tonic::Response<GetNetworkInfoResponse>, tonic::Status> {
        let client_info = crate::client_info_from_metadata(request.metadata());
        tracing::Span::current().record("client_info", &client_info);

        let request_id = uuid::Uuid::new_v4().to_string();
        tracing::Span::current().record("request_id", &request_id);

        let network_info: agglayer_grpc_types::node::types::v1::NetworkInfo = self
            .service
            .get_network_info(request.into_inner().network_id.into())
            .map_err(|error| {
                error!(?error, "Failed to get network state");
                match error {
                    agglayer_rpc::GetNetworkInfoError::UnknownNetworkType { .. } => {
                        tonic::Status::with_error_details(
                            tonic::Code::NotFound,
                            "Network type could not be determined",
                            ErrorDetails::with_error_info(
                                GetNetworkInfoErrorKind::UnknownNetworkType.as_str_name(),
                                GET_NETWORK_INFO_METHOD_PATH,
                                [],
                            ),
                        )
                    }
                    agglayer_rpc::GetNetworkInfoError::InternalError { source, .. } => {
                        tonic::Status::with_error_details(
                            tonic::Code::Internal,
                            "Internal error",
                            ErrorDetails::with_error_info(
                                GetNetworkInfoErrorKind::InternalError.as_str_name(),
                                GET_NETWORK_INFO_METHOD_PATH,
                                [("error".into(), format!("{source:?}"))],
                            ),
                        )
                    }
                }
            })?
            .into();

        Ok(tonic::Response::new(GetNetworkInfoResponse {
            network_info: Some(network_info),
        }))
    }
}
