use std::{convert::Infallible, sync::Arc};

use agglayer_config::Config;
use agglayer_contracts::{AggchainContract, L1TransactionFetcher, RollupContract};
use agglayer_grpc_server::node::v1::{
    certificate_submission_service_server::CertificateSubmissionServiceServer,
    configuration_service_server::ConfigurationServiceServer,
    node_state_service_server::NodeStateServiceServer,
};
use agglayer_storage::stores::{
    DebugReader, DebugWriter, EpochStoreReader, PendingCertificateReader, PendingCertificateWriter,
    StateReader, StateWriter,
};
use certificate_submission_service::CertificateSubmissionServer;
use configuration_service::ConfigurationServer;
use http::{Request, Response};
use node_state_service::NodeStateServer;
use tonic::{
    body::{boxed, BoxBody},
    codec::CompressionEncoding,
    server::NamedService,
};
use tower::{Service, ServiceExt as _};

mod certificate_submission_service;
mod configuration_service;
mod node_state_service;

#[derive(Default)]
pub struct Server {}

#[derive(Default)]
pub struct ServerBuilder {
    pub(crate) router: axum::Router,
    reflection_fds: Vec<&'static [u8]>,
}

impl ServerBuilder {
    fn add_rpc_service<S>(mut self, rpc_service: S) -> Self
    where
        S: Service<Request<BoxBody>, Response = Response<BoxBody>, Error = Infallible>
            + NamedService
            + Clone
            + Sync
            + Send
            + 'static,
        S::Future: Send + 'static,
        S::Error: Into<eyre::Error> + Send,
    {
        self.router = self.router.route_service(
            &format!("/{}/{{*rest}}", S::NAME),
            rpc_service.map_request(|r: Request<axum::body::Body>| r.map(boxed)),
        );

        self
    }
    pub fn add_reflection_service(mut self, file_descriptor: &'static [u8]) -> Self {
        self.reflection_fds.push(file_descriptor);

        self
    }

    pub fn build(self) -> Result<axum::Router, tonic_reflection::server::Error> {
        let (reflection_v1, reflection_v1alpha) = self.reflection_fds.iter().fold(
            (
                tonic_reflection::server::Builder::configure(),
                tonic_reflection::server::Builder::configure(),
            ),
            |(v1, v1alpha), descriptor| {
                (
                    v1.register_encoded_file_descriptor_set(descriptor),
                    v1alpha.register_encoded_file_descriptor_set(descriptor),
                )
            },
        );

        Ok(self
            .add_rpc_service(reflection_v1.build_v1()?)
            .add_rpc_service(reflection_v1alpha.build_v1alpha()?)
            .router)
    }
}

impl Server {
    pub fn with_config<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore>(
        config: Arc<Config>,
        rpc_service: Arc<
            agglayer_rpc::AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore, EpochsStore>,
        >,
    ) -> ServerBuilder
    where
        L1Rpc: RollupContract + AggchainContract + L1TransactionFetcher + Send + Sync + 'static,
        PendingStore: PendingCertificateReader + PendingCertificateWriter + 'static,
        StateStore: StateReader + StateWriter + 'static,
        DebugStore: DebugReader + DebugWriter + 'static,
        EpochsStore: EpochStoreReader + 'static,
    {
        let certificate_submission_server = CertificateSubmissionServer {
            service: rpc_service.clone(),
        };
        let certificate_submission_service =
            CertificateSubmissionServiceServer::new(certificate_submission_server)
                .max_decoding_message_size(config.grpc.max_decoding_message_size)
                .max_encoding_message_size(config.grpc.max_encoding_message_size)
                .send_compressed(CompressionEncoding::Zstd)
                .accept_compressed(CompressionEncoding::Zstd);

        let configuration_server = ConfigurationServer {
            service: rpc_service.clone(),
        };
        let configuration_service = ConfigurationServiceServer::new(configuration_server)
            .max_decoding_message_size(config.grpc.max_decoding_message_size)
            .max_encoding_message_size(config.grpc.max_encoding_message_size)
            .send_compressed(CompressionEncoding::Zstd)
            .accept_compressed(CompressionEncoding::Zstd);

        let network_state_server = NodeStateServer {
            service: rpc_service.clone(),
        };
        let network_state_service = NodeStateServiceServer::new(network_state_server)
            .max_decoding_message_size(config.grpc.max_decoding_message_size)
            .max_encoding_message_size(config.grpc.max_encoding_message_size)
            .send_compressed(CompressionEncoding::Zstd)
            .accept_compressed(CompressionEncoding::Zstd);

        ServerBuilder::default()
            .add_rpc_service(certificate_submission_service)
            .add_rpc_service(configuration_service)
            .add_rpc_service(network_state_service)
            .add_reflection_service(agglayer_grpc_types::node::v1::FILE_DESCRIPTOR_SET)
            .add_reflection_service(agglayer_interop::grpc::v1::FILE_DESCRIPTOR_SET)
    }
}

#[cfg(test)]
mod tests;
