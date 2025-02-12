use std::sync::Arc;

use agglayer_config::Config;
use agglayer_grpc_server::node::v1::certificate_submission_service_server::CertificateSubmissionServiceServer;
use tonic::codec::CompressionEncoding;

#[derive(Default)]
pub struct Server;

impl Server {
    pub fn start(self, config: Arc<Config>) -> CertificateSubmissionServiceServer<Server> {
        CertificateSubmissionServiceServer::new(self)
            .max_decoding_message_size(config.grpc.max_decoding_message_size)
            .max_encoding_message_size(config.grpc.max_encoding_message_size)
            .send_compressed(CompressionEncoding::Zstd)
            .accept_compressed(CompressionEncoding::Zstd)
    }

    // pub fn into_router(self) -> tonic::service::AxumRouter {
    //     tonic::service::Routes::new(self.start()).into_axum_router()
    // }

    pub fn reflection() -> (
        tonic_reflection::server::ServerReflectionServer<
            impl tonic_reflection::server::ServerReflection,
        >,
        tonic_reflection::server::v1alpha::ServerReflectionServer<
            impl tonic_reflection::server::v1alpha::ServerReflection,
        >,
    ) {
        let (reflection_v1, reflection_v1alpha) = [
            agglayer_grpc_types::node::v1::FILE_DESCRIPTOR_SET,
            agglayer_grpc_types::protocol::types::v1::FILE_DESCRIPTOR_SET,
        ]
        .iter()
        .fold(
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

        let reflection_v1 = reflection_v1.build_v1().unwrap();
        let reflection_v1alpha = reflection_v1alpha.build_v1alpha().unwrap();
        (reflection_v1, reflection_v1alpha)
    }
}

mod certificate_submission_service;
mod configuration_service;
mod network_state_service;
