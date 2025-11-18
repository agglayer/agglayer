use agglayer_config::Config;
use agglayer_storage::{
    stores::{
        PendingCertificateReader as _, PendingCertificateWriter as _, StateReader as _,
        StateWriter as _,
    },
    tests::TempDBDir,
};
use agglayer_types::{
    Certificate, CertificateHeader, CertificateId, CertificateStatus, Digest, Height, Metadata,
    NetworkId, SettlementTxHash,
};
use jsonrpsee::{core::client::ClientT, rpc_params};

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

    tracing::info!(
        pending_certificate = %pending_certificate.hash(),
        second_pending = %second_pending.hash(),
    );

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

    assert!(res.is_err(), "{res:?}");

    context
        .state_store
        .insert_certificate_header(
            &pending_certificate,
            CertificateStatus::error(agglayer_types::CertificateStatusError::InternalError(
                "testing".to_string(),
            )),
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
            false,
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
            &CertificateStatus::error(agglayer_types::CertificateStatusError::InternalError(
                "testing".to_string(),
            )),
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

#[test_log::test(tokio::test)]
async fn pending_certificate_in_error_force_set_status() {
    let path = TempDBDir::new();

    let mut config = Config::new(&path.path);
    config.debug_mode = true;

    let mut context = TestContext::new_with_config(config).await;
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
            false,
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
            &CertificateStatus::error(agglayer_types::CertificateStatusError::InternalError(
                "testing".to_string(),
            )),
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
            "admin_forceSetCertificateStatus",
            rpc_params![
                pending_certificate.hash(),
                CertificateStatus::Candidate,
                false
            ],
        )
        .await;

    assert!(res.is_ok());
    assert!(context.certificate_receiver.try_recv().is_err());

    let res: CertificateHeader = context
        .state_store
        .get_certificate_header(&certificate_id)
        .unwrap()
        .unwrap();

    assert!(res.settlement_tx_hash.is_some());
    assert_eq!(res.status, CertificateStatus::Candidate);

    let res: Result<(), _> = context
        .admin_client
        .request(
            "admin_forceSetCertificateStatus",
            rpc_params![
                pending_certificate.hash(),
                CertificateStatus::Candidate,
                true
            ],
        )
        .await;

    assert!(res.is_ok());
    assert_eq!(
        context.certificate_receiver.try_recv(),
        Ok((network_id, Height::ZERO, certificate_id))
    );

    let res: CertificateHeader = context
        .state_store
        .get_certificate_header(&certificate_id)
        .unwrap()
        .unwrap();

    assert!(res.settlement_tx_hash.is_some());
    assert_eq!(res.status, CertificateStatus::Candidate);
}

#[rstest::rstest]
#[test_log::test(tokio::test)]
async fn pending_certificate_in_error_with_settlement_tx_hash_force_set_status(
    #[values(
        CertificateStatus::Pending,
        CertificateStatus::Proven,
        CertificateStatus::Candidate
    )]
    initial_status: CertificateStatus,
) {
    let path = TempDBDir::new();

    let mut config = Config::new(&path.path);
    config.debug_mode = true;

    let mut context = TestContext::new_with_config(config).await;
    let network_id = 1.into();

    let pending_certificate = Certificate::new_for_test(network_id, Height::ZERO);
    let certificate_id = pending_certificate.hash();

    context
        .state_store
        .insert_certificate_header(&pending_certificate, initial_status.clone())
        .expect("unable to insert pending certificate header");

    let fake_settlement_tx_hash = SettlementTxHash::from(Digest::from([1; 32]));

    context
        .pending_store
        .insert_settlement_tx_hash_for_certificate(&certificate_id, fake_settlement_tx_hash)
        .expect("unable to insert settlement tx hash in pending store");

    let res: CertificateHeader = context
        .state_store
        .get_certificate_header(&certificate_id)
        .unwrap()
        .unwrap();

    // Settlement tx hash should not be in the header for non-settled certificates
    assert!(res.settlement_tx_hash.is_none());
    assert_eq!(res.status, initial_status);

    // But it should be in pending storage
    let pending_hashes = context
        .pending_store
        .get_settlement_tx_hashes_for_certificate(certificate_id)
        .unwrap();
    assert_eq!(pending_hashes, &[fake_settlement_tx_hash]);

    let res: Result<(), _> = context
        .admin_client
        .request(
            "admin_forceSetCertificateStatus",
            rpc_params![
                pending_certificate.hash(),
                CertificateStatus::Proven,
                false,
                Some(vec![fake_settlement_tx_hash])
            ],
        )
        .await;

    tracing::debug!("Force set certificate status result: {:?}", res);
    assert!(res.is_ok());
    assert!(context.certificate_receiver.try_recv().is_err());

    let res: CertificateHeader = context
        .state_store
        .get_certificate_header(&certificate_id)
        .unwrap()
        .unwrap();

    assert_eq!(res.status, CertificateStatus::Proven);
    assert!(res.settlement_tx_hash.is_none());

    // Verify the settlement tx hash was removed from pending store
    let pending_hashes = context
        .pending_store
        .get_settlement_tx_hashes_for_certificate(certificate_id)
        .unwrap();
    assert!(pending_hashes.is_empty());
}

#[test_log::test(tokio::test)]
async fn pending_certificate_settled_force_set_status() {
    let path = TempDBDir::new();

    let mut config = Config::new(&path.path);
    config.debug_mode = true;

    let mut context = TestContext::new_with_config(config).await;
    let network_id = 1.into();

    let pending_certificate = Certificate::new_for_test(network_id, Height::ZERO);
    let certificate_id = pending_certificate.hash();

    context
        .state_store
        .insert_certificate_header(&pending_certificate, CertificateStatus::Pending)
        .expect("unable to insert pending certificate header");

    let fake_settlement_tx_hash = SettlementTxHash::from(Digest::from([1; 32]));
    context
        .state_store
        .update_settlement_tx_hash(&certificate_id, fake_settlement_tx_hash, false)
        .expect("unable to update settlement tx hash");
    context
        .state_store
        .update_certificate_header_status(&certificate_id, &CertificateStatus::Settled)
        .expect("unable to update certificate status to settled");

    let res: CertificateHeader = context
        .state_store
        .get_certificate_header(&certificate_id)
        .unwrap()
        .unwrap();

    assert!(res.settlement_tx_hash.is_some());
    assert_eq!(res.status, CertificateStatus::Settled);

    let res: Result<(), _> = context
        .admin_client
        .request(
            "admin_forceSetCertificateStatus",
            rpc_params![
                pending_certificate.hash(),
                CertificateStatus::Proven,
                false,
                Some(fake_settlement_tx_hash)
            ],
        )
        .await;

    assert!(res.is_err());
    assert!(context.certificate_receiver.try_recv().is_err());

    let res: CertificateHeader = context
        .state_store
        .get_certificate_header(&certificate_id)
        .unwrap()
        .unwrap();

    assert!(res.settlement_tx_hash.is_some());
    assert_eq!(res.status, CertificateStatus::Settled);
}
