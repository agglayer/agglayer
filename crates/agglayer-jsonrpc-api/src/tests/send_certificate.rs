use agglayer_config::Config;
use agglayer_storage::{
    stores::{PendingCertificateWriter as _, StateReader as _, StateWriter as _},
    tests::TempDBDir,
};
use agglayer_types::{
    Certificate, CertificateHeader, CertificateId, CertificateStatus, Digest, Height, Metadata,
    NetworkId, SettlementTxHash,
};
use jsonrpsee::{core::client::ClientT, rpc_params};
use tracing::info;

use crate::testutils::TestContext;

#[test_log::test(tokio::test)]
async fn send_certificate_method_can_be_called_and_succeed() {
    let mut config = TestContext::get_default_config();
    config.proof_signers.insert(
        1,
        Certificate::wallet_for_test(NetworkId::new(1))
            .address()
            .into(),
    );
    let mut context = TestContext::new_with_config(config).await;
    let client = context.api_client.clone();

    let cert_id: CertificateId = client
        .request(
            "interop_sendCertificate",
            rpc_params![Certificate::new_for_test(1.into(), Height::ZERO)],
        )
        .await
        .unwrap();
    let received_cert = context.certificate_receiver.try_recv();

    assert!(received_cert.is_ok());
    assert_eq!(received_cert.unwrap().2, cert_id);
}

#[test_log::test(tokio::test)]
async fn send_certificate_method_can_be_called_and_fail() {
    let path = TempDBDir::new();
    let config = Config::new(&path.path);
    let context = TestContext::new_with_config(config).await;

    let res: Result<(), _> = context
        .api_client
        .request(
            "interop_sendCertificate",
            rpc_params![Certificate::new_for_test(0.into(), Height::ZERO)],
        )
        .await;

    assert!(res.is_err());
}

#[test_log::test(tokio::test)]
async fn send_certificate_with_blocked_networks() {
    let path = TempDBDir::new();
    let mut config = Config::new(&path.path);
    config.proxied_networks = Some(agglayer_config::ProxiedNetworksConfig::for_tests(vec![
        NetworkId::new(1),
    ]));
    config.proof_signers.insert(
        1,
        Certificate::wallet_for_test(NetworkId::new(1))
            .address()
            .into(),
    );
    config.proof_signers.insert(
        2,
        Certificate::wallet_for_test(NetworkId::new(2))
            .address()
            .into(),
    );
    let context = TestContext::new_with_config(config).await;

    let res: Result<CertificateId, _> = context
        .api_client
        .request(
            "interop_sendCertificate",
            rpc_params![Certificate::new_for_test(1.into(), Height::ZERO)],
        )
        .await;
    info!(?res, "Sending proxied cert to public port");
    assert!(
        res.is_err(),
        "Certificate from blocked network should be rejected"
    );

    let res: Result<CertificateId, _> = context
        .api_client
        .request(
            "interop_sendCertificate",
            rpc_params![Certificate::new_for_test(2.into(), Height::ZERO)],
        )
        .await;
    info!(?res, "Sending non-proxied cert to public port");
    assert!(
        res.is_ok(),
        "Certificate from non-blocked network should be allowed"
    );
}

#[test_log::test(tokio::test)]
async fn send_certificate_method_requires_known_signer() {
    let path = TempDBDir::new();
    let mut config = Config::new(&path.path);
    // Willingly insert a signer that is not the one that'll be used down below
    config.proof_signers.insert(
        1,
        Certificate::wallet_for_test(NetworkId::new(2))
            .address()
            .into(),
    );

    let context = TestContext::new_with_config(config).await;
    let send_request: Result<CertificateId, _> = context
        .api_client
        .request(
            "interop_sendCertificate",
            rpc_params![Certificate::new_for_test(1.into(), Height::ZERO)],
        )
        .await;

    assert!(send_request.is_err());
}

#[test_log::test(tokio::test)]
async fn pending_certificate_in_error_can_be_replaced() {
    let path = TempDBDir::new();

    let mut config = Config::new(&path.path);
    config.proof_signers.insert(
        1,
        Certificate::wallet_for_test(NetworkId::new(1))
            .address()
            .into(),
    );

    let context = TestContext::new_with_config(config).await;
    let network_id = 1.into();

    let pending_certificate = Certificate::new_for_test(network_id, Height::ZERO);
    let mut second_pending = Certificate::new_for_test(network_id, Height::ZERO);
    second_pending.metadata = Metadata::new([1; 32].into());

    assert_ne!(pending_certificate.hash(), second_pending.hash());
    context
        .state_store
        .insert_certificate_header(&pending_certificate, CertificateStatus::Pending)
        .expect("unable to insert pending certificate header");
    context
        .pending_store
        .insert_pending_certificate(network_id, Height::ZERO, &pending_certificate)
        .expect("unable to insert pending certificate");

    let res: Result<CertificateId, _> = context
        .api_client
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
        .api_client
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

    let pending_certificate = Certificate::new_for_test(network_id, Height::ZERO);
    let certificate_id = pending_certificate.hash();

    context
        .state_store
        .insert_certificate_header(&pending_certificate, CertificateStatus::Pending)
        .expect("unable to insert pending certificate header");

    context
        .state_store
        .update_settlement_tx_hash(
            &certificate_id,
            SettlementTxHash::from(Digest::from([1; 32])),
        )
        .expect("unable to update settlement tx hash");

    context
        .pending_store
        .insert_pending_certificate(network_id, Height::ZERO, &pending_certificate)
        .expect("unable to insert pending certificate");

    let res: Result<CertificateId, _> = context
        .api_client
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
        .api_client
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
