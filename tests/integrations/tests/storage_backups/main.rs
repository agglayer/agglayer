use std::time::Duration;

use agglayer_config::storage::backup::BackupConfig;
use agglayer_storage::{storage::backup::BackupEngine, tests::TempDBDir};
use agglayer_types::{CertificateHeader, CertificateId, CertificateStatus};
use fail::FailScenario;
use integrations::{
    agglayer_setup::{get_signer, setup_network, start_agglayer},
    wait_for_settlement_or_error,
};
use jsonrpsee::core::client::ClientT as _;
use jsonrpsee::rpc_params;
use pessimistic_proof_test_suite::forest::Forest;
use rstest::rstest;

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
async fn recover_with_backup() {
    let tmp_dir = TempDBDir::new();
    let backup_dir = TempDBDir::new();

    assert_ne!(tmp_dir.path, backup_dir.path);

    let scenario = FailScenario::setup();

    let mut config = agglayer_config::Config::new(&tmp_dir.path);
    config.storage.backup = BackupConfig::with_path(backup_dir.path.clone());

    // L1 is a RAII guard
    let (agglayer_shutdowned, l1, client, handle) =
        setup_network(&tmp_dir.path, Some(config)).await;
    let signer = get_signer(0);

    let state = Forest::default().with_signer(signer);

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert_eq!(result.status, CertificateStatus::Settled);
    handle.cancel();
    _ = agglayer_shutdowned.await;

    let config = agglayer_config::Config::new(&tmp_dir.path);
    std::fs::remove_dir_all(&config.storage.pending_db_path).unwrap();
    std::fs::remove_dir_all(&config.storage.epochs_db_path).unwrap();
    std::fs::remove_dir_all(&config.storage.state_db_path).unwrap();

    BackupEngine::restore(
        &backup_dir.path.join("state"),
        &config.storage.state_db_path,
    )
    .unwrap();

    let (agglayer_shutdowned, client, handle) =
        start_agglayer(&tmp_dir.path, &l1, Some(config)).await;

    let certificate: CertificateHeader = client
        .request("interop_getCertificateHeader", rpc_params![certificate_id])
        .await
        .unwrap();

    assert_eq!(certificate.status, CertificateStatus::Settled);

    handle.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(360))]
async fn purge_after_n_backup() {
    let tmp_dir = TempDBDir::new();
    let backup_dir = TempDBDir::new();

    assert_ne!(tmp_dir.path, backup_dir.path);

    let scenario = FailScenario::setup();

    let mut config = agglayer_config::Config::new(&tmp_dir.path);
    config.storage.backup = BackupConfig::Enabled {
        path: backup_dir.path.clone(),
        state_max_backup_count: 1,
        pending_max_backup_count: 1,
    };

    // L1 is a RAII guard
    let (agglayer_shutdowned, l1, client, handle) =
        setup_network(&tmp_dir.path, Some(config)).await;
    let signer = get_signer(0);

    let state = Forest::default().with_signer(signer);

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);
    let mut certificate2 = state.clone().apply_events(&[], &[]);
    certificate2.height = 1;

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert_eq!(result.status, CertificateStatus::Settled);

    let certificate_id2: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate2])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id2).await;

    assert_eq!(result.status, CertificateStatus::Settled);

    handle.cancel();
    _ = agglayer_shutdowned.await;

    let config = agglayer_config::Config::new(&tmp_dir.path);
    std::fs::remove_dir_all(&config.storage.pending_db_path).unwrap();
    std::fs::remove_dir_all(&config.storage.epochs_db_path).unwrap();
    std::fs::remove_dir_all(&config.storage.state_db_path).unwrap();

    let backup_report = BackupEngine::list_backups(&backup_dir.path).unwrap();

    assert_eq!(backup_report.get_state().len(), 1);
    assert_eq!(backup_report.get_pending().len(), 1);

    BackupEngine::restore(
        &backup_dir.path.join("state"),
        &config.storage.state_db_path,
    )
    .unwrap();

    let (agglayer_shutdowned, client, handle) =
        start_agglayer(&tmp_dir.path, &l1, Some(config)).await;

    let certificate: CertificateHeader = client
        .request("interop_getCertificateHeader", rpc_params![certificate_id])
        .await
        .unwrap();

    assert_eq!(certificate.status, CertificateStatus::Settled);

    let certificate: CertificateHeader = client
        .request("interop_getCertificateHeader", rpc_params![certificate_id2])
        .await
        .unwrap();

    assert_eq!(certificate.status, CertificateStatus::Settled);

    handle.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(360))]
async fn report_contains_all_backups() {
    let tmp_dir = TempDBDir::new();
    let backup_dir = TempDBDir::new();

    assert_ne!(tmp_dir.path, backup_dir.path);

    let scenario = FailScenario::setup();

    let mut config = agglayer_config::Config::new(&tmp_dir.path);
    config.storage.backup = BackupConfig::with_path(backup_dir.path.clone());

    // L1 is a RAII guard
    let (agglayer_shutdowned, l1, client, handle) =
        setup_network(&tmp_dir.path, Some(config)).await;
    let signer = get_signer(0);

    let state = Forest::default().with_signer(signer);

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);
    let mut certificate2 = state.clone().apply_events(&[], &[]);
    certificate2.height = 1;

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert_eq!(result.status, CertificateStatus::Settled);

    let certificate_id2: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate2])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id2).await;

    assert_eq!(result.status, CertificateStatus::Settled);

    handle.cancel();
    _ = agglayer_shutdowned.await;

    let config = agglayer_config::Config::new(&tmp_dir.path);
    std::fs::remove_dir_all(&config.storage.pending_db_path).unwrap();
    std::fs::remove_dir_all(&config.storage.epochs_db_path).unwrap();
    std::fs::remove_dir_all(&config.storage.state_db_path).unwrap();

    let backup_report = BackupEngine::list_backups(&backup_dir.path).unwrap();

    // There are 4 backups because 2 actions triggers a backup per certs:
    // - One when the L1 `tx_hash` is known
    // - One when the `Certificate` is settled and the network state is updated
    assert_eq!(backup_report.get_state().len(), 4);
    assert_eq!(backup_report.get_pending().len(), 4);

    BackupEngine::restore(
        &backup_dir.path.join("state"),
        &config.storage.state_db_path,
    )
    .unwrap();

    let (agglayer_shutdowned, client, handle) =
        start_agglayer(&tmp_dir.path, &l1, Some(config)).await;

    let certificate: CertificateHeader = client
        .request("interop_getCertificateHeader", rpc_params![certificate_id])
        .await
        .unwrap();

    assert_eq!(certificate.status, CertificateStatus::Settled);

    let certificate: CertificateHeader = client
        .request("interop_getCertificateHeader", rpc_params![certificate_id2])
        .await
        .unwrap();

    assert_eq!(certificate.status, CertificateStatus::Settled);

    handle.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(360))]
async fn restore_at_particular_level() {
    let tmp_dir = TempDBDir::new();
    let backup_dir = TempDBDir::new();

    assert_ne!(tmp_dir.path, backup_dir.path);

    let scenario = FailScenario::setup();

    let mut config = agglayer_config::Config::new(&tmp_dir.path);
    config.storage.backup = BackupConfig::with_path(backup_dir.path.clone());

    // L1 is a RAII guard
    let (agglayer_shutdowned, l1, client, handle) =
        setup_network(&tmp_dir.path, Some(config)).await;
    let signer = get_signer(0);

    let state = Forest::default().with_signer(signer);

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);
    let mut certificate2 = state.clone().apply_events(&[], &[]);
    certificate2.height = 1;

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert_eq!(result.status, CertificateStatus::Settled);

    let certificate_id2: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate2])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id2).await;

    assert_eq!(result.status, CertificateStatus::Settled);

    handle.cancel();
    _ = agglayer_shutdowned.await;

    let config = agglayer_config::Config::new(&tmp_dir.path);
    std::fs::remove_dir_all(&config.storage.pending_db_path).unwrap();
    std::fs::remove_dir_all(&config.storage.epochs_db_path).unwrap();
    std::fs::remove_dir_all(&config.storage.state_db_path).unwrap();

    let backup_report = BackupEngine::list_backups(&backup_dir.path).unwrap();

    assert_eq!(backup_report.get_state().len(), 4);
    assert_eq!(backup_report.get_pending().len(), 4);

    BackupEngine::restore_at(
        &backup_dir.path.join("state"),
        &config.storage.state_db_path,
        2,
    )
    .unwrap();

    let (agglayer_shutdowned, client, handle) =
        start_agglayer(&tmp_dir.path, &l1, Some(config)).await;

    let certificate: CertificateHeader = client
        .request("interop_getCertificateHeader", rpc_params![certificate_id])
        .await
        .unwrap();

    assert_eq!(certificate.status, CertificateStatus::Settled);

    let error: Result<CertificateHeader, jsonrpsee::core::ClientError> = client
        .request("interop_getCertificateHeader", rpc_params![certificate_id2])
        .await;

    let expected_message = format!("Resource not found: Certificate({:#})", certificate_id2);

    assert!(
        matches!(error.unwrap_err(), jsonrpsee::core::ClientError::Call(obj) if obj.message() == expected_message)
    );

    handle.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}
