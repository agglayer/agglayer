use std::sync::Arc;

use agglayer_grpc_server::node::v1::configuration_service_server::ConfigurationService;
use agglayer_grpc_types::node::v1::{
    GetEpochConfigurationErrorKind, GetEpochConfigurationRequest, GetEpochConfigurationResponse,
};
use agglayer_rpc::AgglayerService;
use tonic_types::{ErrorDetails, StatusExt as _};

pub(crate) const GET_EPOCH_CONFIGURATION_METHOD_PATH: &str =
    "agglayer-node.grpc-api.v1.configuration-service.get-epoch-configuration";

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
        let epoch_configuration = self.service.get_epoch_configuration().ok_or_else(|| {
            let mut error_details = ErrorDetails::with_error_info(
                GetEpochConfigurationErrorKind::UnexpectedClockConfiguration.as_str_name(),
                GET_EPOCH_CONFIGURATION_METHOD_PATH,
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
        })?;

        Ok(tonic::Response::new(GetEpochConfigurationResponse {
            epoch_configuration: Some(epoch_configuration.into()),
        }))
    }
}
