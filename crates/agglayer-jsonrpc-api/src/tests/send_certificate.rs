use agglayer_config::Config;
use agglayer_storage::{
    stores::{
        PendingCertificateWriter as _, StateReader as _, StateWriter as _,
        UpdateEvenIfAlreadyPresent, UpdateStatusToCandidate,
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
            UpdateEvenIfAlreadyPresent::No,
            UpdateStatusToCandidate::Yes,
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
            UpdateEvenIfAlreadyPresent::No,
            UpdateStatusToCandidate::Yes,
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
            "admin_forceEditCertificate",
            rpc_params![
                pending_certificate.hash(),
                "process-now=false",
                "set-status,from=InError,to=Candidate"
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
            "admin_forceEditCertificate",
            rpc_params![pending_certificate.hash(), "process-now=true"],
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

#[test_log::test(tokio::test)]
async fn pending_certificate_in_error_with_settlement_tx_hash_force_set_status() {
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
        .update_settlement_tx_hash(
            &certificate_id,
            fake_settlement_tx_hash,
            UpdateEvenIfAlreadyPresent::No,
            UpdateStatusToCandidate::Yes,
        )
        .expect("unable to update settlement tx hash");

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
            "admin_forceEditCertificate",
            rpc_params![
                pending_certificate.hash(),
                "process-now=false",
                "set-status,from=Candidate,to=Proven",
                format!("set-settlement-tx-hash,from={fake_settlement_tx_hash},to=null")
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

    assert!(res.settlement_tx_hash.is_none());
    assert_eq!(res.status, CertificateStatus::Proven);
}

#[test_log::test(tokio::test)]
async fn pending_certificate_in_error_with_settlement_tx_hash_admin_fixup_tx_hash() {
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
    let fake_settlement_tx_hash_2 = SettlementTxHash::from(Digest::from([2; 32]));
    assert_ne!(fake_settlement_tx_hash, fake_settlement_tx_hash_2);

    context
        .state_store
        .update_settlement_tx_hash(
            &certificate_id,
            fake_settlement_tx_hash,
            UpdateEvenIfAlreadyPresent::No,
            UpdateStatusToCandidate::Yes,
        )
        .expect("unable to update settlement tx hash");
    context
        .state_store
        .update_certificate_header_status(
            &certificate_id,
            &CertificateStatus::InError {
                error: Box::new(agglayer_types::CertificateStatusError::InternalError(
                    "test".into(),
                )),
            },
        )
        .unwrap();

    let res: CertificateHeader = context
        .state_store
        .get_certificate_header(&certificate_id)
        .unwrap()
        .unwrap();

    assert_eq!(res.settlement_tx_hash, Some(fake_settlement_tx_hash));
    assert!(matches!(res.status, CertificateStatus::InError { .. }));

    // InError -> Candidate, fixup tx hash
    let res: Result<(), _> = context
        .admin_client
        .request(
            "admin_forceEditCertificate",
            rpc_params![
                pending_certificate.hash(),
                "process-now=false",
                "set-status,from=InError,to=Candidate",
                format!(
                    "set-settlement-tx-hash,from={fake_settlement_tx_hash},\
                     to={fake_settlement_tx_hash_2}"
                )
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

    assert_eq!(res.settlement_tx_hash, Some(fake_settlement_tx_hash_2));
    assert_eq!(res.status, CertificateStatus::Candidate);

    // Candidate -> InError, fixup tx hash
    let res: Result<(), _> = context
        .admin_client
        .request(
            "admin_forceEditCertificate",
            rpc_params![
                pending_certificate.hash(),
                "process-now=false",
                "set-status,from=Candidate,to=InError",
                format!(
                    "set-settlement-tx-hash,from={fake_settlement_tx_hash_2},\
                     to={fake_settlement_tx_hash}"
                )
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

    assert_eq!(res.settlement_tx_hash, Some(fake_settlement_tx_hash));
    assert!(matches!(res.status, CertificateStatus::InError { .. }));

    // Candidate -> InError, fixup tx hash, but "from" status is wrong so nothing
    // happens
    let res: Result<(), _> = context
        .admin_client
        .request(
            "admin_forceEditCertificate",
            rpc_params![
                pending_certificate.hash(),
                "process-now=false",
                "set-status,from=Candidate,to=InError",
                format!(
                    "set-settlement-tx-hash,from={fake_settlement_tx_hash},\
                     to={fake_settlement_tx_hash_2}"
                )
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

    assert_eq!(res.settlement_tx_hash, Some(fake_settlement_tx_hash));
    assert!(matches!(res.status, CertificateStatus::InError { .. }));

    // InError -> Candidate, fixup tx hash, but "from" tx hash is wrong so nothing
    // happens
    let res: Result<(), _> = context
        .admin_client
        .request(
            "admin_forceEditCertificate",
            rpc_params![
                pending_certificate.hash(),
                "process-now=false",
                "set-status,from=InError,to=Candidate",
                format!(
                    "set-settlement-tx-hash,from={fake_settlement_tx_hash_2},\
                     to={fake_settlement_tx_hash}"
                )
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

    assert_eq!(res.settlement_tx_hash, Some(fake_settlement_tx_hash));
    assert!(matches!(res.status, CertificateStatus::InError { .. }));
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
        .update_settlement_tx_hash(
            &certificate_id,
            fake_settlement_tx_hash,
            UpdateEvenIfAlreadyPresent::No,
            UpdateStatusToCandidate::Yes,
        )
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
            "admin_forceEditCertificate",
            rpc_params![
                pending_certificate.hash(),
                "process-now=false",
                "set-status,from=Settled,to=Proven",
                format!("set-settlement-tx-hash,from{fake_settlement_tx_hash},to=null")
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
