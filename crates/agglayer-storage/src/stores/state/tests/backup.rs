use std::sync::Arc;

use agglayer_types::{
    Certificate, CertificateStatus, CertificateStatusError, Digest, Height, NetworkId,
    SettlementTxHash,
};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    backup::{BackupClient, BackupRequest},
    stores::{
        state::StateStore, StateWriter as _, UpdateEvenIfAlreadyPresent, UpdateStatusToCandidate,
    },
    tests::TempDBDir,
};

fn setup_store(
    status: CertificateStatus,
) -> (
    TempDBDir,
    StateStore,
    Certificate,
    UnboundedReceiver<BackupRequest>,
) {
    let tmp = TempDBDir::new();
    let db = Arc::new(StateStore::init_db(tmp.path.as_path()).expect("Unable to init db"));
    let (backup_client, backups) = BackupClient::observable();
    let store = StateStore::new(db, backup_client);

    let certificate = Certificate::new_for_test(NetworkId::new(1), Height::ZERO);
    store
        .insert_certificate_header(&certificate, status)
        .expect("Unable to insert certificate header");

    (tmp, store, certificate, backups)
}

/// No certificate status change should request a backup on its own.
///
/// The backup that has to precede an L1 settlement is taken by the settlement
/// service, which waits for it; a status write here cannot make that promise
/// and used only to duplicate it moments beforehand.
#[test]
fn certificate_status_changes_do_not_trigger_a_state_backup() {
    for status in [
        CertificateStatus::Pending,
        CertificateStatus::Proven,
        CertificateStatus::Candidate,
        CertificateStatus::Settled,
        CertificateStatus::error(CertificateStatusError::InternalError("failed".to_string())),
    ] {
        let (_tmp, store, certificate, mut backups) = setup_store(CertificateStatus::Pending);

        store
            .update_certificate_header_status(&certificate.hash(), &status)
            .expect("Unable to update certificate header status");

        assert!(
            backups.try_recv().is_err(),
            "Moving a certificate to {status} should not request a backup"
        );
    }
}

#[test]
fn recording_a_settlement_tx_hash_triggers_a_state_backup() {
    let (_tmp, store, certificate, mut backups) = setup_store(CertificateStatus::Candidate);

    store
        .update_settlement_tx_hash(
            &certificate.hash(),
            SettlementTxHash::new(Digest::from([1u8; 32])),
            UpdateEvenIfAlreadyPresent::No,
            UpdateStatusToCandidate::No,
        )
        .expect("Unable to update settlement tx hash");

    assert!(
        backups.try_recv().is_ok(),
        "Recording a settlement tx hash should request a backup"
    );
}
