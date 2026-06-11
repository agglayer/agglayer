use std::{sync::Arc, time::Duration};

use agglayer_config::Config;
use agglayer_contracts::{AggchainContract, L1RpcError, L1TransactionFetcher, RollupContract};
use agglayer_grpc_client::node::v1::certificate_submission_service_client::CertificateSubmissionServiceClient;
use agglayer_grpc_server::node::v1::certificate_submission_service_server::CertificateSubmissionServiceServer;
use agglayer_grpc_types::node::{
    types::v1,
    v1::{SubmitCertificateErrorKind, SubmitCertificateRequest},
};
use agglayer_interop::grpc::v1 as interop_v1;
use agglayer_rpc::AgglayerService;
use agglayer_storage::{
    backup::BackupClient,
    stores::{debug::DebugStore, epochs::EpochsStore, pending::PendingStore, state::StateStore},
    tests::TempDBDir,
};
use agglayer_types::{
    aggchain_proof::AggchainProof, testutils::dummy_sp1_stark_proof_with_version, Certificate,
    Digest,
};
use alloy::{network::Ethereum, providers::RootProvider, rpc::types::TransactionReceipt};
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};
use tonic::{transport::Channel, Code};
use tonic_types::StatusExt as _;
use tower::ServiceExt as _;

use crate::certificate_submission_service::CertificateSubmissionServer;

const SUBMIT_CERTIFICATE_METHOD_PATH: &str =
    "agglayer-node.grpc-api.v1.certificate-submission-service.submit_certificate";

struct L1Rpc;

#[async_trait::async_trait]
impl RollupContract for L1Rpc {
    async fn get_trusted_sequencer_address(
        &self,
        _rollup_id: u32,
        _proof_signers: std::collections::HashMap<u32, agglayer_types::Address>,
    ) -> Result<agglayer_types::Address, L1RpcError> {
        unreachable!("invalid certificates are rejected before L1 access")
    }

    async fn get_rollup_contract_address(
        &self,
        _rollup_id: u32,
    ) -> Result<agglayer_types::Address, L1RpcError> {
        unreachable!("invalid certificates are rejected before L1 access")
    }

    async fn get_prev_pessimistic_root(
        &self,
        _rollup_id: u32,
        _before_tx: Option<alloy::primitives::TxHash>,
    ) -> Result<[u8; 32], L1RpcError> {
        unreachable!("invalid certificates are rejected before L1 access")
    }

    async fn get_l1_info_root(&self, _l1_leaf_count: u32) -> Result<[u8; 32], L1RpcError> {
        unreachable!("invalid certificates are rejected before L1 access")
    }

    async fn get_verifier_type(
        &self,
        _rollup_id: u32,
    ) -> Result<agglayer_contracts::rollup::VerifierType, L1RpcError> {
        unreachable!("invalid certificates are rejected before L1 access")
    }

    fn default_l1_info_tree_entry(&self) -> (u32, [u8; 32]) {
        unreachable!("invalid certificates are rejected before L1 access")
    }

    fn get_rollup_manager_address(&self) -> agglayer_types::Address {
        unreachable!("invalid certificates are rejected before L1 access")
    }

    fn get_event_filter_block_range(&self) -> u64 {
        unreachable!("invalid certificates are rejected before L1 access")
    }
}

#[async_trait::async_trait]
impl AggchainContract for L1Rpc {
    async fn get_aggchain_vkey_hash(
        &self,
        _rollup_address: agglayer_types::Address,
        _aggchain_vkey_selector: u16,
    ) -> Result<agglayer_contracts::aggchain::VKeyHash, L1RpcError> {
        unreachable!("invalid certificates are rejected before L1 access")
    }

    async fn get_aggchain_hash(
        &self,
        _rollup_address: agglayer_types::Address,
        _aggchain_data: alloy::primitives::Bytes,
        _before_tx_hash: Option<alloy::primitives::TxHash>,
    ) -> Result<[u8; 32], L1RpcError> {
        unreachable!("invalid certificates are rejected before L1 access")
    }

    async fn get_multisig_context(
        &self,
        _rollup_address: agglayer_types::Address,
    ) -> Result<(Vec<agglayer_types::Address>, usize), L1RpcError> {
        unreachable!("invalid certificates are rejected before L1 access")
    }
}

#[async_trait::async_trait]
impl L1TransactionFetcher for L1Rpc {
    type Provider = RootProvider<Ethereum>;

    async fn fetch_transaction_receipt(
        &self,
        _tx_hash: agglayer_types::SettlementTxHash,
    ) -> Result<Option<TransactionReceipt>, L1RpcError> {
        unreachable!("invalid certificates are rejected before L1 access")
    }

    fn get_provider(&self) -> &Self::Provider {
        unreachable!("invalid certificates are rejected before L1 access")
    }
}

#[tokio::test]
async fn submit_certificate_preserves_unsupported_proof_version_error() {
    let tmp = TempDBDir::new();
    let config = Arc::new(Config::new(&tmp.path));
    let certificate = generic_certificate_proto("v5.2.2");

    let (mut client, tx, jh) = start_server_with_certificate_submission_service(config).await;

    let response = client
        .submit_certificate(SubmitCertificateRequest {
            certificate: Some(certificate),
        })
        .await;

    let error = response.expect_err("non-writable proof version should be rejected");
    assert_eq!(error.code(), Code::InvalidArgument);
    assert_eq!(error.message(), "Unsupported proof version");

    let error_details = error.get_error_details();
    let error_info = error_details
        .error_info()
        .expect("error info should be present");
    assert_eq!(
        error_info.reason,
        SubmitCertificateErrorKind::UnsupportedProofVersion.as_str_name()
    );
    assert_eq!(error_info.domain, SUBMIT_CERTIFICATE_METHOD_PATH);
    assert_eq!(
        error_info.metadata.get("proof_version"),
        Some(&"v5.2.2".to_owned())
    );

    tx.send(()).unwrap();
    jh.await.unwrap();
}

fn generic_certificate_proto(version: &str) -> v1::Certificate {
    let certificate = Certificate::default();
    let proof = interop_v1::AggchainProof::try_from(AggchainProof {
        proof: dummy_sp1_stark_proof_with_version(version),
        aggchain_params: Digest::default(),
        public_values: None,
    })
    .expect("aggchain proof should serialize");

    let mut certificate =
        v1::Certificate::try_from(certificate).expect("test certificate should serialize");
    certificate.aggchain_data = Some(interop_v1::AggchainData {
        data: Some(interop_v1::aggchain_data::Data::Generic(
            interop_v1::AggchainProof {
                signature: None,
                ..proof
            },
        )),
    });
    certificate.metadata = None;
    certificate
}

async fn start_server_with_certificate_submission_service(
    config: Arc<Config>,
) -> (
    CertificateSubmissionServiceClient<Channel>,
    oneshot::Sender<()>,
    JoinHandle<()>,
) {
    let (sender, _receiver) = tokio::sync::mpsc::channel(10);
    let pending_store =
        Arc::new(PendingStore::new_with_path(&config.storage.pending_db_path).unwrap());
    let state_store = Arc::new(
        StateStore::new_with_path(&config.storage.state_db_path, BackupClient::noop()).unwrap(),
    );
    let service = Arc::new(AgglayerService::new(
        sender,
        pending_store.clone(),
        state_store.clone(),
        Arc::new(DebugStore::new_with_path(&config.storage.debug_db_path).unwrap()),
        Arc::new(
            EpochsStore::new(
                config.clone(),
                pending_store,
                state_store,
                BackupClient::noop(),
            )
            .unwrap(),
        ),
        config,
        Arc::new(L1Rpc),
    ));
    let (tx, rx) = oneshot::channel::<()>();
    let svc = CertificateSubmissionServiceServer::new(CertificateSubmissionServer { service });

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let app = axum::Router::new().route_service(
        "/agglayer.node.v1.CertificateSubmissionService/{*rest}",
        svc.map_request(|r: http::Request<axum::body::Body>| r.map(tonic::body::Body::new)),
    );

    let jh = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async { drop(rx.await) })
            .await
            .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = CertificateSubmissionServiceClient::connect(format!("http://{addr}"))
        .await
        .unwrap();

    (client, tx, jh)
}
