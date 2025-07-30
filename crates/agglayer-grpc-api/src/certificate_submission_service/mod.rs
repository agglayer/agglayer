use std::{str::FromStr, sync::Arc};

use agglayer_contracts::{L1TransactionFetcher, RollupContract};
use agglayer_grpc_server::node::v1::certificate_submission_service_server::CertificateSubmissionService;
use agglayer_grpc_types::node::v1::{
    SubmitCertificateErrorKind, SubmitCertificateRequest, SubmitCertificateResponse,
};
use agglayer_rpc::AgglayerService;
use agglayer_storage::stores::{
    DebugReader, DebugWriter, PendingCertificateReader, PendingCertificateWriter, StateReader,
    StateWriter,
};
use agglayer_types::Signature;
use error::CertificateSubmissionErrorWrapper;
use tonic::Status;
use tonic_types::{ErrorDetails, StatusExt};
use tracing::instrument;

const GRPC_METADATA_NAME_EXTRA_CERTIFICATE_SIGNATURE: &str =
    "x-agglayer-extra-certificate-signature";

const SUBMIT_CERTIFICATE_METHOD_PATH: &str =
    "agglayer-node.grpc-api.v1.certificate-submission-service.submit_certificate";

pub struct CertificateSubmissionServer<L1Rpc, PendingStore, StateStore, DebugStore> {
    pub(crate) service: Arc<AgglayerService<L1Rpc, PendingStore, StateStore, DebugStore>>,
}

mod error;

#[tonic::async_trait]
impl<L1Rpc, PendingStore, StateStore, DebugStore> CertificateSubmissionService
    for CertificateSubmissionServer<L1Rpc, PendingStore, StateStore, DebugStore>
where
    PendingStore: PendingCertificateReader + PendingCertificateWriter + 'static,
    StateStore: StateReader + StateWriter + 'static,
    DebugStore: DebugReader + DebugWriter + 'static,
    L1Rpc: RollupContract + L1TransactionFetcher + Send + Sync + 'static,
{
    #[instrument(skip(self, request), level = "debug", fields(certificate_id = tracing::field::Empty))]
    async fn submit_certificate(
        &self,
        request: tonic::Request<SubmitCertificateRequest>,
    ) -> Result<tonic::Response<SubmitCertificateResponse>, tonic::Status> {
        // Retrieve extra signature from query metadata if any
        let extra_signature: Option<Signature> =
            get_extra_signature(request.metadata()).map_err(|e| *e)?;

        let certificate: agglayer_types::Certificate = match request.into_inner().certificate {
            Some(certificate) => certificate.try_into().map_err(
                |error: agglayer_grpc_types::compat::v1::Error| {
                    tonic::Status::with_error_details(
                        tonic::Code::InvalidArgument,
                        "Invalid certificate",
                        ErrorDetails::with_error_info(
                            SubmitCertificateErrorKind::from(error.kind()),
                            SUBMIT_CERTIFICATE_METHOD_PATH,
                            [("error".into(), format!("{error:?}"))],
                        ),
                    )
                },
            )?,
            None => return Err(tonic::Status::invalid_argument("Missing certificate")),
        };

        let certificate_id = self
            .service
            .send_certificate(certificate, extra_signature)
            .await
            .map_err(|error| {
                CertificateSubmissionErrorWrapper::new(error, SUBMIT_CERTIFICATE_METHOD_PATH)
            })?;

        Ok(tonic::Response::new(SubmitCertificateResponse {
            certificate_id: Some(certificate_id.into()),
        }))
    }
}

pub(crate) fn get_extra_signature(
    metadata: &tonic::metadata::MetadataMap,
) -> Result<Option<Signature>, Box<Status>> {
    if let Some(value) = metadata.get(GRPC_METADATA_NAME_EXTRA_CERTIFICATE_SIGNATURE) {
        let sig_str = value.to_str().map_err(|e| {
            tonic::Status::invalid_argument(format!("invalid signature encoding: {e}"))
        })?;

        let sig = Signature::from_str(sig_str).map_err(|e| {
            tonic::Status::invalid_argument(format!("invalid hex in signature: {e}"))
        })?;

        Ok(Some(sig))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use agglayer_types::{aggchain_proof::AggchainData, Certificate, NetworkId};
    use ethers::utils::hex;
    use rstest::rstest;

    use super::{get_extra_signature, GRPC_METADATA_NAME_EXTRA_CERTIFICATE_SIGNATURE};

    #[rstest]
    #[case("0x")]
    #[case("")]
    fn parse_query_metadata(#[case] prefix: String) {
        let mut metadata = tonic::metadata::MetadataMap::new();

        let expected_signature = {
            let cert = Certificate::new_for_test(NetworkId::new(10), 1u64);
            let AggchainData::ECDSA { signature } = cert.aggchain_data else {
                panic!("dummy test certificate changed")
            };

            signature
        };

        metadata.insert(
            GRPC_METADATA_NAME_EXTRA_CERTIFICATE_SIGNATURE,
            format!("{prefix}{}", hex::encode(expected_signature.as_bytes()))
                .parse()
                .unwrap(),
        );

        assert!(
            matches!(get_extra_signature(&metadata), Ok(Some(signature)) if signature == expected_signature)
        );
    }
}
