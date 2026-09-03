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

/// Returns the id of the latest state backup if restoring it yields
/// `certificate_id` as settled, `None` otherwise (including while a backup is
/// mid-write, when listing or restoring can transiently fail).
fn settled_snapshot_id(backup_dir: &std::path::Path, certificate_id: CertificateId) -> Option<u32> {
    let backup_id = latest_backup_id(BackupEngine::list_backups(backup_dir).ok()?.get_state())?;

    let restored = TempDBDir::new();
    let state_path = restored.path.join("state");
    BackupEngine::restore_at(&backup_dir.join("state"), &state_path, backup_id).ok()?;
    let state = StateStore::new_with_path(&state_path, BackupClient::noop()).ok()?;

    (state.get_certificate_header(&certificate_id).ok()??.status == CertificateStatus::Settled)
        .then_some(backup_id)
}

/// Waits until the latest state backup contains `certificate_id` as settled,
/// and returns that backup's id.
///
/// The settled write requests a backup, so this always completes; but exact
/// backup counts are not deterministic: state backup requests coalesce into a
/// single queue slot, so two requests close together (such as the
/// accepted-as-pending and proven triggers, or any pair when backups are slow
/// under load) can produce one backup covering both writes. Waiting on the
/// restored snapshot content instead of on backup counts or ids stays correct
/// however many requests coalesced.
async fn wait_for_settled_snapshot(
    backup_dir: &std::path::Path,
    certificate_id: CertificateId,
) -> u32 {
    let start = tokio::time::Instant::now();

    loop {
        if let Some(backup_id) = settled_snapshot_id(backup_dir, certificate_id) {
            return backup_id;
        }

        if start.elapsed() >= Duration::from_secs(30) {
            panic!("Timed out waiting for a settled backup snapshot of {certificate_id}");
        }

        tokio::time::sleep(Duration::from_millis(250)).await;
    }
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
    let network_id = certificate.network_id;
    let height = certificate.height;

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;

    assert_eq!(result.status, CertificateStatus::Settled);

    // Wait until the settled state is durably backed up before shutting the
    // node down, so the final restore-from-latest captures a Settled
    // certificate rather than an earlier snapshot.
    wait_for_settled_snapshot(&backup_dir.path, certificate_id).await;

    handle.cancel();
    _ = agglayer_shutdowned.await;

    // The invariant the `Pending` trigger exists for: backup 1 is requested
    // as soon as the certificate is accepted, before the orchestrator picks
    // it up, and must carry the certificate body — it only lives in the
    // pending database until the certificate is processed.
    //
    // The status is asserted as a range rather than exactly `Pending`: the
    // backup engine flushes on a blocking task, so under load it can run
    // after the certificate task has advanced. All allowed statuses are
    // pre-settlement and all recover; pinning `Pending` would only buy a
    // flaky test.
    {
        let (_restored, state, pending) = restore_snapshot(&backup_dir.path, 1);

        let header = state
            .get_certificate_header(&certificate_id)
            .unwrap()
            .expect("backup 1 should contain the certificate header");
        assert!(
            matches!(
                header.status,
                CertificateStatus::Pending
                    | CertificateStatus::Proven
                    | CertificateStatus::Candidate
            ),
            "backup 1 should predate settlement, got {:?}",
            header.status
        );

        assert!(
            pending
                .get_certificate(network_id, height)
                .unwrap()
                .is_some(),
            "backup 1 should contain the submitted certificate body"
        );
    }

    // The invariant the `Proven` trigger exists for: backup 2 is requested
    // before the settlement tx is submitted, and must carry the certificate
    // header and its generated proof together — they live in two different
    // databases, and the later backups no longer hold the proof.
    //
    // The status is asserted as a range rather than exactly `Proven`: the
    // backup engine flushes on a blocking task, so under load it can run
    // after the certificate task has advanced to `Candidate`. Both are
    // pre-settlement and both recover; pinning `Proven` would only buy a
    // flaky test.
    {
        let (_restored, state, pending) = restore_snapshot(&backup_dir.path, 2);

        let header = state
            .get_certificate_header(&certificate_id)
            .unwrap()
            .expect("backup 2 should contain the certificate header");
        assert!(
            matches!(
                header.status,
                CertificateStatus::Proven | CertificateStatus::Candidate
            ),
            "backup 2 should predate settlement, got {:?}",
            header.status
        );

        assert!(
            pending.get_proof(certificate_id).unwrap().is_some(),
            "backup 2 should contain the generated proof"
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

    // Wait for certificate1's settled state to be durably backed up before
    // sending certificate2.
    wait_for_settled_snapshot(&backup_dir.path, certificate_id).await;

    let certificate_id2: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate2])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id2).await;

    assert_eq!(result.status, CertificateStatus::Settled);

    // This configuration purges state and pending backups eagerly, so the
    // retained backup count stays at 1 after both settlements. Wait for
    // certificate2's settled backup to be durable before shutting the node
    // down; otherwise the restore can capture one of certificate2's
    // pre-settlement snapshots and the post-restart status assertion flakes.
    wait_for_settled_snapshot(&backup_dir.path, certificate_id2).await;

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

    wait_for_settled_snapshot(&backup_dir.path, certificate_id2).await;

    handle.cancel();
    _ = agglayer_shutdowned.await;

    let config = agglayer_config::Config::new(&tmp_dir.path);
    std::fs::remove_dir_all(&config.storage.pending_db_path).unwrap();
    std::fs::remove_dir_all(&config.storage.epochs_db_path).unwrap();
    std::fs::remove_dir_all(&config.storage.state_db_path).unwrap();

    let backup_report = BackupEngine::list_backups(&backup_dir.path).unwrap();

    // Four actions request a backup per certificate: accepted as pending,
    // proven, L1 tx hash known, and settled. Close-together requests coalesce
    // into the single state queue slot, so the exact count depends on timing;
    // it is bounded by the request total, and state and pending snapshots are
    // always taken in pairs.
    let state_backups = backup_report.get_state().len();
    assert!(
        (2..=8).contains(&state_backups),
        "expected between 2 and 8 state backups, got {state_backups}"
    );
    assert_eq!(state_backups, backup_report.get_pending().len());

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

    // Captured before certificate2 is submitted, so this backup is
    // certificate1's settled snapshot and cannot contain certificate2.
    let certificate1_settled_backup =
        wait_for_settled_snapshot(&backup_dir.path, certificate_id).await;

    let certificate_id2: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate2])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id2).await;

    assert_eq!(result.status, CertificateStatus::Settled);

    wait_for_settled_snapshot(&backup_dir.path, certificate_id2).await;

    handle.cancel();
    _ = agglayer_shutdowned.await;

    let config = agglayer_config::Config::new(&tmp_dir.path);
    std::fs::remove_dir_all(&config.storage.pending_db_path).unwrap();
    std::fs::remove_dir_all(&config.storage.epochs_db_path).unwrap();
    std::fs::remove_dir_all(&config.storage.state_db_path).unwrap();

    BackupEngine::restore_at(
        &backup_dir.path.join("state"),
        &config.storage.state_db_path,
        certificate1_settled_backup,
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
