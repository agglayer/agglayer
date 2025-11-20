use std::sync::Arc;

use agglayer_grpc_server::node::v1::configuration_service_server::ConfigurationService;
use agglayer_grpc_types::node::v1::{
    GetEpochConfigurationErrorKind, GetEpochConfigurationRequest, GetEpochConfigurationResponse,
};
use agglayer_rpc::AgglayerService;
use tonic_types::{ErrorDetails, StatusExt as _};
use tracing::instrument;

pub(crate) const GET_EPOCH_CONFIGURATION_METHOD_PATH: &str =
    "agglayer-node.grpc-api.v1.configuration-service.get-epoch-configuration";

pub struct ConfigurationServer<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore> {
    pub(crate) service:
        Arc<AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore>>,
}

#[tonic::async_trait]
impl<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore> ConfigurationService
    for ConfigurationServer<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore>
where
    DebugStore: Send + Sync + 'static,
    L1Rpc: Send + Sync + 'static,
    PendingStore: Send + Sync + 'static,
    StateStore: Send + Sync + 'static,
    EpochsStore: Send + Sync + 'static,
{
    #[instrument(skip(self, request), level = "debug", fields(client = tracing::field::Empty))]
    async fn get_epoch_configuration(
        &self,
        request: tonic::Request<GetEpochConfigurationRequest>,
    ) -> Result<tonic::Response<GetEpochConfigurationResponse>, tonic::Status> {
        let client_info = crate::client_info_from_metadata(request.metadata());
        tracing::Span::current().record("client", &client_info);

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
