use std::sync::Arc;

use agglayer_grpc_server::node::v1::node_state_service_server::NodeStateService;
use agglayer_grpc_types::{
    compat::v1::Error,
    node::v1::{
        GetCertificateHeaderErrorKind, GetCertificateHeaderRequest, GetCertificateHeaderResponse,
        GetLatestCertificateHeaderErrorKind, GetLatestCertificateHeaderRequest,
        GetLatestCertificateHeaderResponse, GetNetworkStatusErrorKind, GetNetworkStatusRequest,
        GetNetworkStatusResponse, LatestCertificateRequestType,
    },
};
use agglayer_rpc::AgglayerService;
use agglayer_storage::stores::{DebugReader, PendingCertificateReader, StateReader};
use tonic_types::{ErrorDetails, StatusExt as _};
use tracing::error;

const GET_CERTIFICATE_HEADER_METHOD_PATH: &str =
    "agglayer-node.grpc-api.v1.node-state-service.get_certificate_header";
const GET_LATEST_CERTIFICATE_HEADER_METHOD_PATH: &str =
    "agglayer-node.grpc-api.v1.node-state-service.get_latest_certificate_header";
const GET_NETWORK_STATUS_METHOD_PATH: &str =
    "agglayer-node.grpc-api.v1.node-state-service.get_network_status";

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

    #[tracing::instrument(level = "debug", skip(self, request), fields(request_id = tracing::field::Empty))]
    async fn get_latest_certificate_header(
        &self,
        request: tonic::Request<GetLatestCertificateHeaderRequest>,
    ) -> Result<tonic::Response<GetLatestCertificateHeaderResponse>, tonic::Status> {
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

    #[tracing::instrument(level = "debug", skip(self, request), fields(request_id = tracing::field::Empty))]
    async fn get_network_status(
        &self,
        request: tonic::Request<GetNetworkStatusRequest>,
    ) -> Result<tonic::Response<GetNetworkStatusResponse>, tonic::Status> {
        let request_id = uuid::Uuid::new_v4().to_string();
        tracing::Span::current().record("request_id", &request_id);
        let request = request.into_inner();

        let network_id = request.network_id.into();

        //TODO FINISH IMPLEMENTATION

        // Gather network status information
        let settled_certificate = self
            .service
            .get_latest_settled_certificate_header(network_id)
            .map_err(|error| {
                error!(?error, "Failed to get latest settled certificate");
                tonic::Status::with_error_details(
                    tonic::Code::NotFound,
                    "Failed to get latest settled certificate",
                    ErrorDetails::with_error_info(
                        GetNetworkStatusErrorKind::MissingLatestSettledCertificate.as_str_name(),
                        GET_NETWORK_STATUS_METHOD_PATH,
                        [],
                    ),
                )
            })?;

        let pending_certificate = self
            .service
            .get_latest_pending_certificate_header(network_id)
            .map_err(|error| {
                error!(?error, "Failed to get latest pending certificate");
                tonic::Status::with_error_details(
                    tonic::Code::NotFound,
                    "Failed to get latest pending certificate",
                    ErrorDetails::with_error_info(
                        GetNetworkStatusErrorKind::MissingLatestPendingCertificate.as_str_name(),
                        GET_NETWORK_STATUS_METHOD_PATH,
                        [],
                    ),
                )
            })?;

        // TODO: Fetch additional needed data.

        // Build NetworkStatus response
        let network_status = agglayer_grpc_types::node::types::v1::NetworkStatus {
            network_status: "default".to_string(), // Default status
            network_type: "default".to_string(),   // Default type
            network_id: request.network_id,
            settled_height: settled_certificate
                .as_ref()
                .map(|cert| cert.height.as_u64())
                .unwrap_or(0),
            settled_certificate_id: settled_certificate
                .as_ref()
                .map(|cert| cert.certificate_id.into()),
            settled_pp_root: None, // Would need additional data source
            settled_ler: None,     // Would need additional data source
            settled_bridge_global_index: None, // Would need additional data source
            settled_claim_global_index: None, // Would need additional data source
            latest_pending_height: pending_certificate
                .as_ref()
                .map(|cert| cert.height.as_u64())
                .unwrap_or(0),
            latest_pending_status: pending_certificate
                .as_ref()
                .map(|cert| format!("{:?}", cert.status))
                .unwrap_or_else(|| "None".to_string()),
            latest_pending_error: "".to_string(), // Would need additional error tracking
            latest_epoch_with_settlement: 0,      // Would need epoch tracking
        };

        Ok(tonic::Response::new(GetNetworkStatusResponse {
            network_status: Some(network_status),
        }))
    }
}
