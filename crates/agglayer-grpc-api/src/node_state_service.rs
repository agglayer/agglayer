use std::sync::Arc;

use agglayer_grpc_server::node::v1::node_state_service_server::NodeStateService;
use agglayer_grpc_types::{
    node::v1::{
        GetCertificateHeaderErrorKind, GetCertificateHeaderRequest, GetCertificateHeaderResponse,
        GetLatestCertificateHeaderRequest, GetLatestCertificateHeaderResponse,
        LatestCertificateRequestType,
    },
    protocol::types::v1::{CertificateHeader, CertificateStatus, FixedBytes32},
};
use agglayer_rpc::AgglayerService;
use agglayer_storage::stores::{DebugReader, PendingCertificateReader, StateReader};
use agglayer_types::Digest;
use axum::body::Bytes;
use tonic_types::{ErrorDetails, StatusExt as _};

#[allow(unused)]
const SERVICE_PATH: &str = "agglayer-node.grpc-api.v1.node-state-service";

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
        let context = format!("{}.{}", SERVICE_PATH, "get-certificate-header");

        let certificate_id = request
            .certificate_id
            .ok_or(GetCertificateHeaderErrorKind::MissingCertificateId)
            .and_then(|value| {
                value
                    .value
                    .ok_or(GetCertificateHeaderErrorKind::MissingCertificateId)
            })
            .and_then(|value| {
                Digest::try_from(&value.value[..])
                    .map_err(|_| GetCertificateHeaderErrorKind::MalformedCertificateId)
            })
            .map_err(|error| {
                let mut error_details = ErrorDetails::new();

                let error_message = match error {
                    GetCertificateHeaderErrorKind::MalformedCertificateId => {
                        "Certificate ID is malformed"
                    }
                    GetCertificateHeaderErrorKind::MissingCertificateId => {
                        "Certificate ID is missing"
                    }
                    _ => "Unknown error",
                };

                error_details.set_error_info(error.as_str_name(), &context, []);
                error_details.add_bad_request_violation("certificate_id", error_message);

                tonic::Status::with_error_details(
                    tonic::Code::InvalidArgument,
                    error_message,
                    error_details,
                )
            })?;

        let response = self
            .service
            .get_certificate_header(certificate_id)
            .map_err(|error| {
                let mut error_details = ErrorDetails::new();
                match error {
                    agglayer_rpc::CertificateRetrievalError::Storage(error) => {
                        error_details.set_error_info(
                            GetCertificateHeaderErrorKind::Internal.as_str_name(),
                            &context,
                            [("error_message".into(), error.to_string())],
                        );
                        error_details.set_request_info(&request_id, format!("{:?}", error));
                        tonic::Status::with_error_details(
                            tonic::Code::Internal,
                            "Internal error",
                            error_details,
                        )
                    }
                    agglayer_rpc::CertificateRetrievalError::NotFound { .. } => {
                        tonic::Status::with_details(
                            tonic::Code::NotFound,
                            "Certificate not found",
                            Bytes::new(),
                        )
                    }
                }
            })
            .map(|header| GetCertificateHeaderResponse {
                certificate_header: Some(CertificateHeader {
                    network_id: *header.network_id,
                    height: header.height,
                    epoch_number: header.epoch_number,
                    certificate_index: header.certificate_index,
                    certificate_id: Some(agglayer_grpc_types::protocol::types::v1::CertificateId {
                        value: Some(FixedBytes32 {
                            value: Bytes::copy_from_slice(&*header.certificate_id),
                        }),
                    }),
                    prev_local_exit_root: None,
                    new_local_exit_root: None,
                    metadata: None,
                    status: CertificateStatus::Unspecified.into(),
                    error: None,
                    settlement_tx_hash: None,
                }),
            })?;

        Ok(tonic::Response::new(response))
    }

    #[tracing::instrument(level = "debug", skip(self, request), fields(request_id = tracing::field::Empty))]
    async fn get_latest_certificate_header(
        &self,
        request: tonic::Request<GetLatestCertificateHeaderRequest>,
    ) -> Result<tonic::Response<GetLatestCertificateHeaderResponse>, tonic::Status> {
        let request_id = uuid::Uuid::new_v4().to_string();
        tracing::Span::current().record("request_id", &request_id);

        let request = request.into_inner();
        let context = format!("{}.{}", SERVICE_PATH, "get-latest-known-certificate-header");
        let network_id = request.network_id;

        let request_type =
            LatestCertificateRequestType::try_from(request.r#type).map_err(|_| {
                let mut error_details = ErrorDetails::new();
                error_details.set_error_info(
                    GetCertificateHeaderErrorKind::InvalidRequestType.as_str_name(),
                    &context,
                    [],
                );
                error_details.set_request_info(&request_id, "Invalid request type");

                tonic::Status::with_error_details(
                    tonic::Code::InvalidArgument,
                    "Invalid request type",
                    error_details,
                )
            })?;

        let result = match request_type {
            LatestCertificateRequestType::Unspecified => {
                let mut error_details = ErrorDetails::new();
                error_details.set_error_info(
                    GetCertificateHeaderErrorKind::InvalidRequestType.as_str_name(),
                    &context,
                    [],
                );

                error_details.set_request_info(&request_id, "Invalid request type");

                return Err(tonic::Status::with_error_details(
                    tonic::Code::InvalidArgument,
                    "Invalid request type",
                    error_details,
                ));
            }
            LatestCertificateRequestType::Pending => self
                .service
                .get_latest_pending_certificate_header(network_id.into()),
            LatestCertificateRequestType::Settled => self
                .service
                .get_latest_settled_certificate_header(network_id.into()),
        };

        let response = result
            .map_err(|error| {
                let mut error_details = ErrorDetails::new();
                match error {
                    agglayer_rpc::CertificateRetrievalError::Storage(error) => {
                        error_details.set_error_info(
                            GetCertificateHeaderErrorKind::Internal.as_str_name(),
                            &context,
                            [("error_message".into(), error.to_string())],
                        );
                        error_details.set_request_info(&request_id, format!("{:?}", error));
                        tonic::Status::with_error_details(
                            tonic::Code::Internal,
                            "Internal error",
                            error_details,
                        )
                    }
                    agglayer_rpc::CertificateRetrievalError::NotFound { .. } => {
                        tonic::Status::with_details(
                            tonic::Code::NotFound,
                            "Certificate not found",
                            Bytes::new(),
                        )
                    }
                }
            })
            .map(|header| GetLatestCertificateHeaderResponse {
                certificate_header: header.map(|header| CertificateHeader {
                    network_id: *header.network_id,
                    height: header.height,
                    epoch_number: header.epoch_number,
                    certificate_index: header.certificate_index,
                    certificate_id: None,
                    prev_local_exit_root: None,
                    new_local_exit_root: None,
                    metadata: None,
                    status: CertificateStatus::Unspecified.into(),
                    error: None,
                    settlement_tx_hash: None,
                }),
            })?;

        Ok(tonic::Response::new(response))
    }
}
