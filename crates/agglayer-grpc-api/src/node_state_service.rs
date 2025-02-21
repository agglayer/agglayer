use agglayer_grpc_server::node::v1::node_state_service_server::NodeStateService;
use agglayer_grpc_types::node::v1::{
    GetCertificateHeaderRequest, GetCertificateHeaderResponse,
    GetLatestKnownCertificateHeaderRequest, GetLatestKnownCertificateHeaderResponse,
    GetLatestPendingCertificateHeaderRequest, GetLatestPendingCertificateHeaderResponse,
    GetLatestSettledCertificateHeaderRequest, GetLatestSettledCertificateHeaderResponse,
};

#[allow(unused)]
const SERVICE_PATH: &str = "agglayer-node.grpc-api.v1.node-state-service";

pub struct NodeStateServer {}

#[tonic::async_trait]
impl NodeStateService for NodeStateServer {
    async fn get_certificate_header(
        &self,
        request: tonic::Request<GetCertificateHeaderRequest>,
    ) -> Result<tonic::Response<GetCertificateHeaderResponse>, tonic::Status> {
        let _request = request.into_inner();
        let response = GetCertificateHeaderResponse::default();
        Ok(tonic::Response::new(response))
    }

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
