use agglayer_grpc_server::node::v1::configuration_service_server::ConfigurationService;
use agglayer_grpc_types::node::v1::{
    ConfigurationErrorKind, GetEpochConfigurationRequest, GetEpochConfigurationResponse,
};
use tonic_types::{ErrorDetails, StatusExt as _};

const SERVICE_PATH: &str = "agglayer-node.grpc-api.v1.configuration-service";

pub struct ConfigurationServer {}

#[tonic::async_trait]
impl ConfigurationService for ConfigurationServer {
    async fn get_epoch_configuration(
        &self,
        _request: tonic::Request<GetEpochConfigurationRequest>,
    ) -> Result<tonic::Response<GetEpochConfigurationResponse>, tonic::Status> {
        let mut error_details = ErrorDetails::new();
        let context = format!("{}.{}", SERVICE_PATH, "get-epoch-configuration");

        error_details.set_error_info(
            ConfigurationErrorKind::UnexpectedClockConfiguration.as_str_name(),
            context,
            [],
        );

        error_details.set_localized_message(
            "en-US",
            "AggLayer isn't configured with a BlockClock configuration, thus no \
             EpochConfiguration is available",
        );

        let status = tonic::Status::with_error_details(
            tonic::Code::InvalidArgument,
            "Unexpected clock configuration.",
            error_details,
        );

        Err(status)
    }
}
