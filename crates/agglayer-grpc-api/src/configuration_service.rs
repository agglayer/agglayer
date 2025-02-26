use std::sync::Arc;

use agglayer_grpc_server::node::v1::configuration_service_server::ConfigurationService;
use agglayer_grpc_types::node::v1::{
    ConfigurationErrorKind, GetEpochConfigurationRequest, GetEpochConfigurationResponse,
};
use agglayer_rpc::AgglayerService;
use tonic_types::{ErrorDetails, StatusExt as _};

pub(crate) const SERVICE_PATH: &str = "agglayer-node.grpc-api.v1.configuration-service";

pub struct ConfigurationServer<L1Rpc, PendingStore, StateStore, DebugStore> {
    pub(crate) service: Arc<AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore>>,
}

#[tonic::async_trait]
impl<L1Rpc, PendingStore, StateStore, DebugStore> ConfigurationService
    for ConfigurationServer<L1Rpc, PendingStore, StateStore, DebugStore>
where
    DebugStore: Send + Sync + 'static,
    L1Rpc: Send + Sync + 'static,
    PendingStore: Send + Sync + 'static,
    StateStore: Send + Sync + 'static,
{
    async fn get_epoch_configuration(
        &self,
        _request: tonic::Request<GetEpochConfigurationRequest>,
    ) -> Result<tonic::Response<GetEpochConfigurationResponse>, tonic::Status> {
        self.service
            .get_epoch_configuration()
            .ok_or_else(|| {
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

                tonic::Status::with_error_details(
                    tonic::Code::InvalidArgument,
                    "Unexpected clock configuration.",
                    error_details,
                )
            })
            .map(|epoch_configuration| {
                let response = GetEpochConfigurationResponse {
                    epoch_configuration: Some(epoch_configuration.into()),
                };

                tonic::Response::new(response)
            })
    }
}
