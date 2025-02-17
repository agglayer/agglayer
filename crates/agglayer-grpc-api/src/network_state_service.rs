use agglayer_grpc_server::node::v1::network_state_service_server::NetworkStateService;
use agglayer_grpc_types::node::v1::{
    GetLatestKnownCertificateHeaderRequest, GetLatestKnownCertificateHeaderResponse,
    GetLatestPendingCertificateHeaderRequest, GetLatestPendingCertificateHeaderResponse,
    GetLatestSettledCertificateHeaderRequest, GetLatestSettledCertificateHeaderResponse,
};

pub struct NetworkStateServer {}

#[tonic::async_trait]
impl NetworkStateService for NetworkStateServer {
    async fn get_latest_known_certificate_header(
        &self,
        request: tonic::Request<GetLatestKnownCertificateHeaderRequest>,
    ) -> Result<tonic::Response<GetLatestKnownCertificateHeaderResponse>, tonic::Status> {
        let _request = request.into_inner();
        let response = GetLatestKnownCertificateHeaderResponse::default();
        Ok(tonic::Response::new(response))
    }

    async fn get_latest_settled_certificate_header(
        &self,
        request: tonic::Request<GetLatestSettledCertificateHeaderRequest>,
    ) -> Result<tonic::Response<GetLatestSettledCertificateHeaderResponse>, tonic::Status> {
        let _request = request.into_inner();
        let response = GetLatestSettledCertificateHeaderResponse::default();
        Ok(tonic::Response::new(response))
    }

    async fn get_latest_pending_certificate_header(
        &self,
        request: tonic::Request<GetLatestPendingCertificateHeaderRequest>,
    ) -> Result<tonic::Response<GetLatestPendingCertificateHeaderResponse>, tonic::Status> {
        let _request = request.into_inner();
        let response = GetLatestPendingCertificateHeaderResponse::default();
        Ok(tonic::Response::new(response))
    }
}
