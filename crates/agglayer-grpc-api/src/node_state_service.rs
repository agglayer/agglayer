use agglayer_grpc_server::node::v1::node_state_service_server::NodeStateService;
use agglayer_grpc_types::node::v1::{
    GetCertificateHeaderRequest, GetCertificateHeaderResponse, GetLatestCertificateHeaderRequest,
    GetLatestCertificateHeaderResponse,
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

    async fn get_latest_certificate_header(
        &self,
        request: tonic::Request<GetLatestCertificateHeaderRequest>,
    ) -> Result<tonic::Response<GetLatestCertificateHeaderResponse>, tonic::Status> {
        let _request = request.into_inner();
        let response = GetLatestCertificateHeaderResponse::default();
        Ok(tonic::Response::new(response))
    }
}
