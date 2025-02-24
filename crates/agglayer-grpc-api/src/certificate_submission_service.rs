use std::collections::HashMap;
use std::sync::Arc;

use agglayer_contracts::L1TransactionFetcher;
use agglayer_contracts::RollupContract;
use agglayer_grpc_server::node::v1::certificate_submission_service_server::CertificateSubmissionService;
use agglayer_grpc_types::node::v1::{
    CertificateSubmissionErrorKind, SubmitCertificateRequest, SubmitCertificateResponse,
};
use agglayer_grpc_types::protocol::types::v1::aggchain_data::Data;
use agglayer_grpc_types::protocol::types::v1::aggchain_proof::Proof;
use agglayer_rpc::AgglayerService;
use agglayer_storage::columns::default_bincode_options;
use agglayer_storage::stores::{
    DebugReader, DebugWriter, PendingCertificateReader, PendingCertificateWriter, StateReader,
    StateWriter,
};
use agglayer_types::Digest;
use axum::body::Bytes;
use bincode::Options;
use tonic_types::ErrorDetails;
use tonic_types::StatusExt as _;
use tracing::instrument;

const SERVICE_PATH: &str = "agglayer-node.grpc-api.v1.certificate-submission-service";

pub struct CertificateSubmissionServer<L1Rpc, PendingStore, StateStore, DebugStore> {
    pub(crate) service: Arc<AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore>>,
}

#[tonic::async_trait]
impl<L1Rpc, PendingStore, StateStore, DebugStore> CertificateSubmissionService
    for CertificateSubmissionServer<L1Rpc, PendingStore, StateStore, DebugStore>
where
    PendingStore: PendingCertificateReader + PendingCertificateWriter + 'static,
    StateStore: StateReader + StateWriter + 'static,
    DebugStore: DebugReader + DebugWriter + 'static,
    L1Rpc: RollupContract + L1TransactionFetcher + Send + Sync + 'static,
{
    #[instrument(skip(self), level = "debug", fields(certificate_id = tracing::field::Empty))]
    async fn submit_certificate(
        &self,
        request: tonic::Request<SubmitCertificateRequest>,
    ) -> Result<tonic::Response<SubmitCertificateResponse>, tonic::Status> {
        let context = format!("{}.{}", SERVICE_PATH, "submit-certificate");
        let certificate = request.into_inner().certificate.unwrap();
        let certificate = agglayer_types::Certificate {
            network_id: certificate.network_id.into(),
            height: certificate.height,
            // TODO: removed when we have compat layer
            prev_local_exit_root: Digest::try_from(
                &*certificate.prev_local_exit_root.unwrap().value,
            )
            .unwrap(),
            new_local_exit_root: Digest::try_from(&*certificate.new_local_exit_root.unwrap().value)
                .unwrap(),
            bridge_exits: vec![],
            imported_bridge_exits: vec![],
            metadata: Digest::try_from(&*certificate.metadata.unwrap().value).unwrap(),
            aggchain_data: certificate
                .aggchain_data
                .ok_or_else(|| {
                    let mut error_details = ErrorDetails::new();

                    error_details.set_error_info(
                        CertificateSubmissionErrorKind::MissingRequiredField.as_str_name(),
                        &context,
                        [],
                    );
                    error_details
                        .add_bad_request_violation("aggchain_data", "aggchain_data is missing");

                    tonic::Status::with_error_details(
                        tonic::Code::InvalidArgument,
                        "Missing aggchain_data field",
                        error_details,
                    )
                })
                .map(|source| match source.data {
                    Some(witness) => match witness {
                        Data::Signature(signature) => {
                            let signature: agglayer_types::primitives::Signature =
                                (*signature.value).try_into().unwrap();
                            agglayer_types::aggchain_proof::AggchainData::ECDSA { signature }
                        }
                        Data::Generic(proof) => {
                            let aggchain_params = (*proof.aggchain_params).try_into().unwrap();
                            let proof = match proof.proof.unwrap() {
                                Proof::Sp1Stark(fixed_bytes32) => {
                                    agglayer_types::aggchain_proof::Proof::SP1Stark(Box::new(
                                        default_bincode_options()
                                            .deserialize(&fixed_bytes32.value)
                                            .unwrap(),
                                    ))
                                }
                            };

                            agglayer_types::aggchain_proof::AggchainData::Generic {
                                proof,
                                aggchain_params,
                            }
                        }
                    },
                    None => unreachable!(),
                })?,
        };

        let certificate_id = certificate.hash();
        let certificate_id = self
            .service
            .send_certificate(certificate)
            .await
            .map_err(|error| match error {
                agglayer_rpc::CertificateSubmissionError::Storage(_) => {
                    tonic::Status::internal("Internal storage error")
                }
                agglayer_rpc::CertificateSubmissionError::OrchestratorNotResponsive => {
                    tonic::Status::internal("Orchestrator not responsive")
                }
                agglayer_rpc::CertificateSubmissionError::SignatureError(
                    signature_verification_error,
                ) => {
                    let mut error_details = ErrorDetails::new();

                    error_details.set_error_info(
                        CertificateSubmissionErrorKind::SignatureVerification.as_str_name(),
                        &context,
                        [
                            ("certificate_id".into(), certificate_id.to_string()),
                            ("source".into(), signature_verification_error.to_string()),
                        ],
                    );

                    tonic::Status::with_error_details(
                        tonic::Code::InvalidArgument,
                        "Signature verification error",
                        error_details,
                    )
                }
                agglayer_rpc::CertificateSubmissionError::UnableToReplacePendingCertificate {
                    reason,
                    height,
                    network_id,
                    stored_certificate_id,
                    replacement_certificate_id,
                    source,
                } => {
                    let mut error_details = ErrorDetails::new();

                    let mut details: HashMap<String, String> = vec![
                        ("reason".into(), reason.to_string()),
                        ("height".into(), height.to_string()),
                        ("network_id".into(), network_id.to_string()),
                        (
                            "stored_certificate_id".into(),
                            stored_certificate_id.to_string(),
                        ),
                        (
                            "replacement_certificate_id".into(),
                            replacement_certificate_id.to_string(),
                        ),
                    ]
                    .into_iter()
                    .collect();

                    if let Some(source) = source {
                        details.insert("source".into(), source.to_string());
                    }

                    error_details.set_error_info(
                        CertificateSubmissionErrorKind::UnableToReplacePendingCertificate
                            .as_str_name(),
                        &context,
                        details,
                    );

                    tonic::Status::with_error_details(
                        tonic::Code::InvalidArgument,
                        "Unable to replace pending certificate",
                        error_details,
                    )
                }
            })?;

        Ok(tonic::Response::new(SubmitCertificateResponse {
            certificate_id: Some(agglayer_grpc_types::protocol::types::v1::CertificateId {
                value: Some(agglayer_grpc_types::protocol::types::v1::FixedBytes32 {
                    value: Bytes::copy_from_slice(certificate_id.as_ref()),
                }),
            }),
        }))
    }
}
