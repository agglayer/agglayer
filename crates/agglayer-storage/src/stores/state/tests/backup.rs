use std::sync::Arc;

use agglayer_types::{
    Certificate, CertificateStatus, CertificateStatusError, Digest, Height, LocalNetworkStateData,
    NetworkId, SettlementTxHash,
};

use crate::{
    backup::{BackupClient, ObservedBackupRequests},
    stores::{
        state::StateStore, StateWriter as _, UpdateEvenIfAlreadyPresent, UpdateStatusToCandidate,
    },
    tests::TempDBDir,
};

fn setup_store(
    status: CertificateStatus,
) -> (TempDBDir, StateStore, Certificate, ObservedBackupRequests) {
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

#[test]
fn proven_status_triggers_a_state_backup() {
    let (_tmp, store, certificate, mut backups) = setup_store(CertificateStatus::Pending);

    store
        .update_certificate_header_status(&certificate.hash(), &CertificateStatus::Proven)
        .expect("Unable to update certificate header status");

    backups
        .state
        .try_recv()
        .expect("Moving a certificate to Proven should request a state backup");

    assert!(
        backups.state.try_recv().is_err(),
        "A single status update should request a single backup"
    );
    assert!(
        backups.epoch.try_recv().is_err(),
        "Proven should back up the state and pending DBs, not an epoch DB"
    );
}

#[test]
fn non_proven_statuses_do_not_trigger_a_state_backup() {
    for status in [
        CertificateStatus::Pending,
        CertificateStatus::Candidate,
        CertificateStatus::Settled,
        CertificateStatus::error(CertificateStatusError::InternalError("failed".to_string())),
    ] {
        let (_tmp, store, certificate, mut backups) = setup_store(CertificateStatus::Pending);

        store
            .update_certificate_header_status(&certificate.hash(), &status)
            .expect("Unable to update certificate header status");

        assert!(
            backups.state.try_recv().is_err(),
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
        backups.state.try_recv().is_ok(),
        "Recording a settlement tx hash should request a backup"
    );
}

#[test]
fn removing_a_settlement_tx_hash_triggers_a_state_backup() {
    let (_tmp, store, certificate, mut backups) = setup_store(CertificateStatus::Candidate);

    store
        .update_settlement_tx_hash(
            &certificate.hash(),
            SettlementTxHash::new(Digest::from([1u8; 32])),
            UpdateEvenIfAlreadyPresent::No,
            UpdateStatusToCandidate::No,
        )
        .expect("Unable to update settlement tx hash");
    backups
        .state
        .try_recv()
        .expect("Recording a settlement tx hash should request a backup");

    store
        .remove_settlement_tx_hash(&certificate.hash())
        .expect("Unable to remove settlement tx hash");

    assert!(
        backups.state.try_recv().is_ok(),
        "Removing a settlement tx hash should request a backup"
    );
}

#[test]
fn writing_a_local_network_state_triggers_a_state_backup() {
    let (_tmp, store, _certificate, mut backups) = setup_store(CertificateStatus::Settled);

    store
        .write_local_network_state(&NetworkId::new(1), &LocalNetworkStateData::default(), &[])
        .expect("Unable to write local network state");

    assert!(
        backups.state.try_recv().is_ok(),
        "Writing a local network state should request a backup"
    );
}
