use std::time::Duration;

use agglayer_config::storage::backup::BackupConfig;
use agglayer_storage::{
    backup::{BackupClient, BackupEngine, BackupEngineInfo},
    stores::{
        pending::PendingStore, state::StateStore, PendingCertificateReader as _, StateReader as _,
    },
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

/// How long the backup engine must stay idle before its newest backup is taken
/// to cover everything written so far.
const BACKUP_QUIESCENT_FOR: Duration = Duration::from_secs(5);

/// Waits until at least one backup exists and no new one has appeared for
/// [`BACKUP_QUIESCENT_FOR`], then returns the newest backup id.
///
/// Backup requests coalesce, so a certificate no longer produces a fixed
/// number of backups and tests cannot count them. What they can rely on is
/// that once the engine has gone quiet, its newest backup covers every write
/// made before it did.
async fn wait_for_backup_quiescence(backup_dir: &std::path::Path) -> u32 {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    let mut newest = None;
    let mut unchanged_since = tokio::time::Instant::now();

    loop {
        let report = BackupEngine::list_backups(backup_dir).unwrap();
        let state = latest_backup_id(report.get_state());

        // The two databases are backed up in lockstep, so a mismatch means a
        // run is still in progress.
        let quiet = if state.is_none() || state != latest_backup_id(report.get_pending()) {
            unchanged_since = tokio::time::Instant::now();
            false
        } else if state != newest {
            newest = state;
            unchanged_since = tokio::time::Instant::now();
            false
        } else {
            unchanged_since.elapsed() >= BACKUP_QUIESCENT_FOR
        };

        if quiet {
            return newest.expect("quiescence implies at least one backup");
        }

        assert!(
            tokio::time::Instant::now() < deadline,
            "Timed out waiting for backups to settle"
        );
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

fn latest_backup_id(backups: &[BackupEngineInfo]) -> Option<u32> {
    backups.iter().map(|backup| backup.backup_id).max()
}

/// Restores backup `backup_id` from both `state/` and `pending/` into a fresh
/// directory, so one specific snapshot can be inspected without touching the
/// node's live databases.
///
/// The returned [`TempDBDir`] owns the restored files and must be kept alive
/// for as long as the stores are used.
fn restore_snapshot(
    backup_dir: &std::path::Path,
    backup_id: u32,
) -> (TempDBDir, StateStore, PendingStore) {
    let restored = TempDBDir::new();
    let state_path = restored.path.join("state");
    let pending_path = restored.path.join("pending");

    BackupEngine::restore_at(&backup_dir.join("state"), &state_path, backup_id).unwrap();
    BackupEngine::restore_at(&backup_dir.join("pending"), &pending_path, backup_id).unwrap();

    let state = StateStore::new_with_path(&state_path, BackupClient::noop()).unwrap();
    let pending = PendingStore::new_with_path(&pending_path).unwrap();

    (restored, state, pending)
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

    let newest_backup = wait_for_backup_quiescence(&backup_dir.path).await;

    handle.cancel();
    _ = agglayer_shutdowned.await;

    // The invariant the `Proven` trigger exists for: a backup is requested
    // before the settlement tx is submitted, and it must carry the certificate
    // header and its generated proof together — they live in two different
    // databases, and once the certificate settles the proof is gone.
    //
    // Which backup that is, is not pinned. Requests coalesce, so the snapshot
    // that predates settlement is whichever one the engine happened to take
    // first; asserting on an id would only buy a flaky test.
    {
        let pre_settlement = (1..=newest_backup).find(|backup_id| {
            let (_restored, state, pending) = restore_snapshot(&backup_dir.path, *backup_id);

            let predates_settlement = state
                .get_certificate_header(&certificate_id)
                .unwrap()
                .is_some_and(|header| header.status != CertificateStatus::Settled);

            predates_settlement && pending.get_proof(certificate_id).unwrap().is_some()
        });

        assert!(
            pre_settlement.is_some(),
            "no backup carries the certificate header and its proof together from before \
             settlement; the proof is unrecoverable from any backup"
        );
    }

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

    // Let certificate1 finish being backed up before sending certificate2, so
    // the retention behaviour is exercised across two distinct settlements.
    wait_for_backup_quiescence(&backup_dir.path).await;

    let certificate_id2: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate2])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id2).await;

    assert_eq!(result.status, CertificateStatus::Settled);

    // Shutdown drains whatever is still outstanding, so the single retained
    // backup covers both settlements by the time the node is down.
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

    handle.cancel();
    _ = agglayer_shutdowned.await;

    let config = agglayer_config::Config::new(&tmp_dir.path);
    std::fs::remove_dir_all(&config.storage.pending_db_path).unwrap();
    std::fs::remove_dir_all(&config.storage.epochs_db_path).unwrap();
    std::fs::remove_dir_all(&config.storage.state_db_path).unwrap();

    let backup_report = BackupEngine::list_backups(&backup_dir.path).unwrap();

    // How many backups two settlements produce is not fixed: requests coalesce,
    // so a burst collapses into a single run. What does hold is that the state
    // and pending databases are backed up together on every run, and that the
    // report lists every one of them.
    let state_ids: Vec<u32> = backup_report
        .get_state()
        .iter()
        .map(|backup| backup.backup_id)
        .collect();
    let pending_ids: Vec<u32> = backup_report
        .get_pending()
        .iter()
        .map(|backup| backup.backup_id)
        .collect();

    assert!(!state_ids.is_empty(), "no backup was reported");
    assert_eq!(
        state_ids, pending_ids,
        "state and pending backups are written in lockstep"
    );
    assert_eq!(
        state_ids,
        (1..=state_ids.len() as u32).collect::<Vec<_>>(),
        "the report should list every backup taken, without gaps"
    );

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

    // The snapshot this test restores to: taken once certificate1 has settled
    // and before certificate2 exists at all.
    let after_certificate1 = wait_for_backup_quiescence(&backup_dir.path).await;

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

    assert!(
        latest_backup_id(backup_report.get_state()).is_some_and(|id| id > after_certificate1),
        "certificate2 should have produced backups after the one being restored"
    );

    BackupEngine::restore_at(
        &backup_dir.path.join("state"),
        &config.storage.state_db_path,
        after_certificate1,
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
