use agglayer_config::Config;
use agglayer_types::{Certificate, CertificateId, NetworkId};
use ethers::{providers, signers::Signer as _};
use jsonrpsee::{core::client::ClientT, http_client::HttpClientBuilder, rpc_params};

use super::next_available_addr;
use crate::{
    kernel::Kernel,
    rpc::{tests::DummyStore, AgglayerImpl},
    service::AgglayerService,
};
use agglayer_types::{Certificate, CertificateHeader, CertificateId, CertificateStatus, NetworkId};
use ethers::signers::Signer as _;
use jsonrpsee::{core::client::ClientT, rpc_params};

use crate::rpc::tests::TestContext;

#[test_log::test(tokio::test)]
async fn send_certificate_method_can_be_called_and_succeed() {
    let path = TempDBDir::new();
    let mut config = Config::new(&path.path);
    config
        .proof_signers
        .insert(1, Certificate::wallet_for_test(NetworkId::new(1)).address());

    let mut context = TestContext::new_with_config(config).await;
    let _: CertificateId = context
        .client
        .request(
            "interop_sendCertificate",
            rpc_params![Certificate::new_for_test(1.into(), 0)],
        )
        .await
        .unwrap();

    assert!(context.certificate_receiver.try_recv().is_ok());
}

#[test_log::test(tokio::test)]
async fn send_certificate_method_can_be_called_and_fail() {
    let path = TempDBDir::new();
    let config = Config::new(&path.path);
    let context = TestContext::new_with_config(config).await;

    let res: Result<(), _> = context
        .client
        .request(
            "interop_sendCertificate",
            rpc_params![Certificate::new_for_test(0.into(), 0)],
        )
        .await;

    assert!(res.is_err());
}

#[test_log::test(tokio::test)]
async fn send_certificate_method_requires_known_signer() {
    let mut config = Config::new_for_test();
    // Willingly insert a signer that is not the one that’ll be used down below
    config
        .proof_signers
        .insert(1, Certificate::wallet_for_test(NetworkId::new(2)).address());

    let context = TestContext::new_with_config(config).await;
    let send_request: Result<CertificateId, _> = context
        .client
        .request(
            "interop_sendCertificate",
            rpc_params![Certificate::new_for_test(1.into(), 0)],
        )
        .await;

    assert!(send_request.is_err());
}
#[test_log::test(tokio::test)]
async fn pending_certificate_in_error_can_be_replaced() {
    let path = TempDBDir::new();

    let mut config = Config::new(&path.path);
    config
        .proof_signers
        .insert(1, Certificate::wallet_for_test(NetworkId::new(1)).address());

    let context = TestContext::new_with_config(config).await;
    let network_id = 1.into();

    let pending_certificate = Certificate::new_for_test(network_id, 0);
    let mut second_pending = Certificate::new_for_test(network_id, 0);
    second_pending.metadata = [1; 32].into();

    assert_ne!(pending_certificate.hash(), second_pending.hash());
    context
        .state_store
        .insert_certificate_header(&pending_certificate, CertificateStatus::Pending)
        .expect("unable to insert pending certificate header");
    context
        .pending_store
        .insert_pending_certificate(network_id, 0, &pending_certificate)
        .expect("unable to insert pending certificate");

    let res: Result<CertificateId, _> = context
        .client
        .request(
            "interop_sendCertificate",
            rpc_params![second_pending.clone()],
        )
        .await;

    assert!(res.is_err());

    context
        .state_store
        .insert_certificate_header(
            &pending_certificate,
            CertificateStatus::InError {
                error: agglayer_types::CertificateStatusError::InternalError("testing".to_string()),
            },
        )
        .expect("unable to insert pending certificate header");

    let res: Result<CertificateId, _> = context
        .client
        .request("interop_sendCertificate", rpc_params![second_pending])
        .await;

    assert!(res.is_ok());
}

#[test_log::test(tokio::test)]
async fn pending_certificate_in_error_force_push() {
    let path = TempDBDir::new();

    let mut config = Config::new(&path.path);
    config.debug_mode = true;

    let context = TestContext::new_with_config(config).await;
    let network_id = 1.into();

    let pending_certificate = Certificate::new_for_test(network_id, 0);
    let certificate_id = pending_certificate.hash();

    context
        .state_store
        .insert_certificate_header(&pending_certificate, CertificateStatus::Pending)
        .expect("unable to insert pending certificate header");

    context
        .state_store
        .update_settlement_tx_hash(&certificate_id, [1; 32].into())
        .expect("unable to update settlement tx hash");

    context
        .pending_store
        .insert_pending_certificate(network_id, 0, &pending_certificate)
        .expect("unable to insert pending certificate");

    let res: Result<CertificateId, _> = context
        .client
        .request(
            "interop_sendCertificate",
            rpc_params![pending_certificate.clone()],
        )
        .await;

    assert!(res.is_err());

    context
        .state_store
        .update_certificate_header_status(
            &certificate_id,
            &CertificateStatus::InError {
                error: agglayer_types::CertificateStatusError::InternalError("testing".to_string()),
            },
        )
        .expect("Unable to update certificate header status");

    let res: Result<CertificateId, _> = context
        .client
        .request(
            "interop_sendCertificate",
            rpc_params![pending_certificate.clone()],
        )
        .await;

    assert!(res.is_err());

    let res: Result<(), _> = context
        .admin_client
        .request(
            "admin_forcePushPendingCertificate",
            rpc_params![pending_certificate, CertificateStatus::Candidate],
        )
        .await;

    assert!(res.is_ok());

    let res: CertificateHeader = context
        .state_store
        .get_certificate_header(&certificate_id)
        .unwrap()
        .unwrap();

    assert!(res.settlement_tx_hash.is_some());
    assert_eq!(res.status, CertificateStatus::Candidate);
}

#[test_log::test(tokio::test)]
async fn send_certificate_method_requires_known_signer() {
    let mut config = Config::new_for_test();
    // Willingly insert a signer that is not the one that’ll be used down below
    config
        .proof_signers
        .insert(1, Certificate::wallet_for_test(NetworkId::new(2)).address());
    let addr = next_available_addr();
    if let IpAddr::V4(ip) = addr.ip() {
        config.rpc.host = ip;
    }
    config.rpc.port = addr.port();

    let config = Arc::new(config);

    let (provider, _mock) = providers::Provider::mocked();
    let (certificate_sender, _certificate_receiver) = tokio::sync::mpsc::channel(1);

    let kernel = Kernel::new(Arc::new(provider), config.clone());

    let service = AgglayerService::new(
        kernel,
        certificate_sender,
        Arc::new(DummyStore {}),
        Arc::new(DummyStore {}),
        Arc::new(DummyStore {}),
        config.clone(),
    );
    let _server_handle = AgglayerImpl::new(Arc::new(service)).start().await.unwrap();

    let url = format!("http://{}/", config.rpc_addr());
    let client = HttpClientBuilder::default().build(url).unwrap();

    let send_request: Result<CertificateId, _> = client
        .request(
            "interop_sendCertificate",
            rpc_params![Certificate::new_for_test(1.into(), 0)],
        )
        .await;

    assert!(send_request.is_err());
}
