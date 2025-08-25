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
use agglayer_interop::grpc::v1::FixedBytes32;
use agglayer_rpc::AgglayerService;
use agglayer_storage::stores::{DebugReader, PendingCertificateReader, StateReader};
use tonic_types::{ErrorDetails, StatusExt as _};
use tracing::{error, warn};

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

        // Gather network status information
        let latest_settled_certificate = self
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

        let latest_pending_certificate = self
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

        // Determine network type from the latest available certificate
        let network_type = match self
            .service
            .get_latest_available_certificate_for_network(network_id)
        {
            Ok(Some(certificate)) => {
                // Determine network type based on aggchain_data variant
                match certificate.aggchain_data {
                    agglayer_types::aggchain_proof::AggchainData::ECDSA { .. } => "ECDSA",
                    agglayer_types::aggchain_proof::AggchainData::Generic { .. } => "Generic",
                }
            }
            Ok(None) => {
                // No certificate found, use default/unknown type
                warn!(
                    "No certificate found for network {}, using default network type",
                    network_id
                );
                "Unknown"
            }
            Err(error) => {
                // Error retrieving certificate, log and use default
                warn!(
                    ?error,
                    "Failed to get certificate for network {}, using default network type",
                    network_id
                );
                "Unknown"
            }
        };

        // TODO: Define network status. Could represent the healthiness of the network
        // in regard to the agglayer-node. We could have multiple kind of status
        // that could represent a network sending too many unprovable certs,
        // or even a network that didn't settle for N epochs and such (optional).
        let network_status = "TBD";

        // Extract settled certificate data
        let (settled_height, settled_cert_id, _settled_epoch) = latest_settled_certificate
            .as_ref()
            .map(|cert| {
                (
                    cert.height.as_u64(),
                    Some(cert.certificate_id.into()),
                    cert.epoch_number,
                )
            })
            .unwrap_or((0, None, None));

        // Get pending certificate error if exists
        let pending_error = latest_pending_certificate
            .as_ref()
            .and_then(|cert| match &cert.status {
                agglayer_types::CertificateStatus::InError { error } => Some(error.to_string()),
                _ => None,
            })
            .unwrap_or_default();

        // Get epoch with latest settlement from settled certificate header
        let latest_epoch_with_settlement = latest_settled_certificate
            .as_ref()
            .and_then(|cert| cert.epoch_number)
            .map(|epoch| epoch.as_u64())
            .unwrap_or(0);

        // Extract settled_pp_root from the settled certificate's proof public values
        let settled_pp_root = latest_settled_certificate.as_ref().and_then(|cert| {
            // Get the proof for the settled certificate
            self.service
                .get_proof(cert.certificate_id)
                .inspect_err(|error| {
                    error!(
                        ?error,
                        "get network status: failed to get proof for settled certificate"
                    );
                })
                .ok()
                .flatten()
                .and_then(|proof| {
                    // Deserialize the proof's public values to get PessimisticProofOutput
                    let agglayer_types::Proof::SP1(sp1_proof) = proof;
                    pessimistic_proof::PessimisticProofOutput::bincode_codec()
                        .deserialize::<pessimistic_proof::PessimisticProofOutput>(
                            sp1_proof.public_values.as_slice(),
                        )
                        .inspect_err(|error| {
                            error!(
                                ?error,
                                "get network status: failed to deserialize pessimistic proof \
                                 output"
                            );
                        })
                        .ok()
                        .map(|output| output.new_pessimistic_root)
                })
        });

        let network_status = agglayer_grpc_types::node::types::v1::NetworkStatus {
            network_status: network_status.to_string(),
            network_type: network_type.to_string(),
            network_id: request.network_id,
            settled_height,
            settled_certificate_id: settled_cert_id,
            // Extract actual data from settled certificate when available
            settled_pp_root: settled_pp_root.map(FixedBytes32::from),
            settled_ler: latest_settled_certificate
                .as_ref()
                .map(|cert| FixedBytes32::from(cert.new_local_exit_root)),
            // For global indices, we'll need to implement additional storage lookups
            // Setting as None for now since they require more complex data retrieval
            settled_let_leaf_count: 0,
            settled_claim: None,
            latest_pending_height: latest_pending_certificate
                .as_ref()
                .map(|cert| cert.height.as_u64())
                .unwrap_or(0),
            latest_pending_status: latest_pending_certificate
                .as_ref()
                .map(|cert| format!("{}", cert.status))
                .unwrap_or_else(|| "Unknown".to_string()),
            latest_pending_error: pending_error,
            latest_epoch_with_settlement,
        };

        Ok(tonic::Response::new(GetNetworkStatusResponse {
            network_status: Some(network_status),
        }))
    }
}
