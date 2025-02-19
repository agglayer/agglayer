use agglayer_grpc_server::node::v1::network_state_service_server::NetworkStateService;
use agglayer_grpc_types::node::v1::{
    GetLatestCertificateHeaderRequest, GetLatestCertificateHeaderResponse,
};

use crate::Server;

#[tonic::async_trait]
impl NetworkStateService for Server {
    async fn get_latest_certificate_header(
        &self,
        request: tonic::Request<GetLatestCertificateHeaderRequest>,
    ) -> Result<tonic::Response<GetLatestCertificateHeaderResponse>, tonic::Status> {
        let _request = request.into_inner();
        let response = GetLatestCertificateHeaderResponse::default();
        Ok(tonic::Response::new(response))
    }
}
