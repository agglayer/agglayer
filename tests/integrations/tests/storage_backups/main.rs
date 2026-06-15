use std::time::Duration;

use agglayer_config::storage::backup::BackupConfig;
use agglayer_storage::{
    backup::{BackupEngine, BackupEngineInfo},
    tests::TempDBDir,
};
use agglayer_types::{CertificateHeader, CertificateId, CertificateStatus};
use fail::FailScenario;
use integrations::{
    agglayer_setup::{setup_network, start_agglayer, wait_for_condition},
    wait_for_settlement_or_error,
};
use jsonrpsee::{core::client::ClientT as _, rpc_params};
use pessimistic_proof_test_suite::forest::Forest;
use rstest::rstest;
use tokio_util::sync::CancellationToken;

#[path = "../common/mod.rs"]
mod common;

const RESOURCE_NOT_FOUND_ERROR: i32 = -10008;

async fn wait_for_backup_counts(
    backup_dir: &std::path::Path,
    minimum_state_backups: usize,
    minimum_pending_backups: usize,
) {
    wait_for_condition("backup creation", Duration::from_secs(30), || async {
        let backup_report = BackupEngine::list_backups(backup_dir).unwrap();
        backup_report.get_state().len() >= minimum_state_backups
            && backup_report.get_pending().len() >= minimum_pending_backups
    })
    .await;
}

fn latest_backup_id(backups: &[BackupEngineInfo]) -> Option<u32> {
    backups.iter().map(|backup| backup.backup_id).max()
}

/// Waits until the latest state and pending backup ids reach at least the given
/// values.
///
/// Unlike [`wait_for_backup_counts`], this works under aggressive purging
/// (where the retained backup count stays at 1) because RocksDB backup ids are
/// monotonic. Each settled certificate produces two backups (one when the L1 tx
/// hash is known, one when it is settled), so the Nth settled certificate's
/// durable state has backup id `2 * N`.
async fn wait_for_backup_ids(
    backup_dir: &std::path::Path,
    minimum_state_backup_id: u32,
    minimum_pending_backup_id: u32,
) {
    wait_for_condition("backup id advance", Duration::from_secs(30), || async {
        let backup_report = BackupEngine::list_backups(backup_dir).unwrap();
        latest_backup_id(backup_report.get_state()).is_some_and(|id| id >= minimum_state_backup_id)
            && latest_backup_id(backup_report.get_pending())
                .is_some_and(|id| id >= minimum_pending_backup_id)
    })
    .await;
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(common::type_0_ecdsa_forest())]
async fn recover_with_backup(#[case] state: Forest) {
    let tmp_dir = TempDBDir::new();
    let backup_dir = TempDBDir::new();

    assert_ne!(tmp_dir.path, backup_dir.path);

    let scenario = FailScenario::setup();

    let mut config = agglayer_config::Config::new(&tmp_dir.path);
    config.storage.backup = BackupConfig::with_path(backup_dir.path.clone());

    let handle = CancellationToken::new();
    // L1 is a RAII guard
    let (agglayer_shutdowned, l1, client) =
        setup_network(&tmp_dir.path, Some(config), Some(handle.clone())).await;

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert_eq!(result.status, CertificateStatus::Settled);

    // Each settled certificate produces two backups (tx-hash known, then
    // settled). Wait for both so the restore captures the settled state rather
    // than the earlier tx-hash snapshot, which would leave the certificate
    // non-Settled after restart.
    wait_for_backup_counts(&backup_dir.path, 2, 2).await;

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
        start_agglayer(&tmp_dir.path, &l1, Some(config), None).await;

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
#[case::type_0_ecdsa(common::type_0_ecdsa_forest())]
async fn purge_after_n_backup(#[case] state: Forest) {
    use agglayer_types::Height;

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

    let handle = CancellationToken::new();
    // L1 is a RAII guard
    let (agglayer_shutdowned, l1, client) =
        setup_network(&tmp_dir.path, Some(config), Some(handle.clone())).await;

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);
    let mut certificate2 = state.clone().apply_events(&[], &[]);
    certificate2.height = Height::new(1);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert_eq!(result.status, CertificateStatus::Settled);

    // Each settled certificate produces two backups (tx-hash known, then
    // settled). Wait for certificate1 to be fully backed up (state/pending
    // backup id >= 2) before sending certificate2.
    wait_for_backup_ids(&backup_dir.path, 2, 2).await;

    let certificate_id2: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate2])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id2).await;

    assert_eq!(result.status, CertificateStatus::Settled);

    // This configuration purges state and pending backups eagerly, so the
    // retained backup count stays at 1 after both settlements. Backup ids are
    // monotonic, so wait for certificate2's settled backup (id >= 4) to be
    // durable before shutting the node down; otherwise the restore can capture
    // certificate2's pre-settlement (tx-hash) snapshot and the post-restart
    // status assertion flakes.
    wait_for_backup_ids(&backup_dir.path, 4, 4).await;

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
        start_agglayer(&tmp_dir.path, &l1, Some(config), None).await;

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
#[case::type_0_ecdsa(common::type_0_ecdsa_forest())]
async fn report_contains_all_backups(#[case] state: Forest) {
    use agglayer_types::Height;

    let tmp_dir = TempDBDir::new();
    let backup_dir = TempDBDir::new();

    assert_ne!(tmp_dir.path, backup_dir.path);

    let scenario = FailScenario::setup();

    let mut config = agglayer_config::Config::new(&tmp_dir.path);
    config.storage.backup = BackupConfig::with_path(backup_dir.path.clone());

    let handle = CancellationToken::new();
    // L1 is a RAII guard
    let (agglayer_shutdowned, l1, client) =
        setup_network(&tmp_dir.path, Some(config), Some(handle.clone())).await;

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);
    let mut certificate2 = state.clone().apply_events(&[], &[]);
    certificate2.height = Height::new(1);

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

    wait_for_backup_counts(&backup_dir.path, 4, 4).await;

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
        start_agglayer(&tmp_dir.path, &l1, Some(config), None).await;

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
#[case::type_0_ecdsa(common::type_0_ecdsa_forest())]
async fn restore_at_particular_level(#[case] state: Forest) {
    use agglayer_types::Height;

    let tmp_dir = TempDBDir::new();
    let backup_dir = TempDBDir::new();

    assert_ne!(tmp_dir.path, backup_dir.path);

    let scenario = FailScenario::setup();

    let mut config = agglayer_config::Config::new(&tmp_dir.path);
    config.storage.backup = BackupConfig::with_path(backup_dir.path.clone());

    let handle = CancellationToken::new();
    // L1 is a RAII guard
    let (agglayer_shutdowned, l1, client) =
        setup_network(&tmp_dir.path, Some(config), Some(handle.clone())).await;

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);
    let mut certificate2 = state.clone().apply_events(&[], &[]);
    certificate2.height = Height::new(1);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert_eq!(result.status, CertificateStatus::Settled);

    wait_for_backup_counts(&backup_dir.path, 2, 2).await;

    let certificate_id2: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate2])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id2).await;

    assert_eq!(result.status, CertificateStatus::Settled);

    wait_for_backup_counts(&backup_dir.path, 4, 4).await;

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
        start_agglayer(&tmp_dir.path, &l1, Some(config), None).await;

    let certificate: CertificateHeader = client
        .request("interop_getCertificateHeader", rpc_params![certificate_id])
        .await
        .unwrap();

    assert_eq!(certificate.status, CertificateStatus::Settled);

    wait_for_condition(
        "restored certificate pruning",
        Duration::from_secs(15),
        || async {
            let error: Result<CertificateHeader, jsonrpsee::core::ClientError> = client
                .request("interop_getCertificateHeader", rpc_params![certificate_id2])
                .await;

            matches!(
                error,
                Err(jsonrpsee::core::ClientError::Call(obj)) if obj.code() == RESOURCE_NOT_FOUND_ERROR
            )
        },
    )
    .await;

    handle.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}
