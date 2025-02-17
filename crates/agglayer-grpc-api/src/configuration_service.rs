use agglayer_grpc_server::node::v1::configuration_service_server::ConfigurationService;
use agglayer_grpc_types::node::v1::{GetEpochConfigurationRequest, GetEpochConfigurationResponse};

use crate::Server;

#[tonic::async_trait]
impl ConfigurationService for Server {
    async fn get_epoch_configuration(
        &self,
        _request: tonic::Request<GetEpochConfigurationRequest>,
    ) -> Result<tonic::Response<GetEpochConfigurationResponse>, tonic::Status> {
        let response = GetEpochConfigurationResponse::default();
        Ok(tonic::Response::new(response))
    }
}
