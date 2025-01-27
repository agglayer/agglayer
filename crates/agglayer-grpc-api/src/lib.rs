use agglayer_grpc_server::node::v1::certificate_submission_service_server::CertificateSubmissionServiceServer;
use tonic::codec::CompressionEncoding;

#[derive(Default)]
pub struct Server;

impl Server {
    pub fn start(self) -> CertificateSubmissionServiceServer<Server> {
        CertificateSubmissionServiceServer::new(self)
            // .max_decoding_message_size(config.grpc.max_decoding_message_size)
            // .max_encoding_message_size(config.grpc.max_encoding_message_size)
            .send_compressed(CompressionEncoding::Zstd)
            .accept_compressed(CompressionEncoding::Zstd)
    }
}

mod certificate_submission_service;
mod configuration_service;
mod network_state_service;
