use std::sync::Arc;

use super::*;
use crate::{
    backup::BackupClient,
    stores::{
        pending::PendingStore, state::StateStore, NetworkInfoReader, PendingCertificateReader,
    },
    tests::{
        mocks::{MockPendingStore, MockStateStore},
        TempDBDir,
    },
};

fn stores() -> (TempDBDir, PendingStore, StateStore) {
    let tmp = TempDBDir::new();
    let pending = PendingStore::new_with_path(&tmp.path.join("pending")).unwrap();
    let state = StateStore::new_with_path(&tmp.path.join("state"), BackupClient::noop()).unwrap();
    (tmp, pending, state)
}

fn assert_failure(result: eyre::Result<()>, context: &str, message: &str) {
    let error = result.unwrap_err();
    assert_eq!(error.to_string(), context);
    assert!(
        matches!(error.downcast_ref::<Error>(), Some(Error::Unexpected(cause)) if cause == message)
    );
}

#[test]
fn submission_tags_failed_header_read_without_continuing() {
    let certificate = Certificate::new_for_test(NetworkId::new(1), Height::ZERO);
    let mut state = MockStateStore::new();
    state
        .expect_get_certificate_header()
        .once()
        .return_once(|_| Err(Error::Unexpected("header read".into())));
    let pending = MockPendingStore::new();
    let result = submit(&pending, &state, &certificate, &CertificateStatus::Pending);
    assert_failure(result, "certificate header", "header read");
}

#[test]
fn submission_tags_failed_pending_insert_without_continuing() {
    let certificate = Certificate::new_for_test(NetworkId::new(1), Height::ZERO);
    let mut state = MockStateStore::new();
    state
        .expect_get_certificate_header()
        .once()
        .return_once(|_| Ok(None));
    let mut pending = MockPendingStore::new();
    pending
        .expect_insert_pending_certificate()
        .once()
        .return_once(|_, _, _| Err(Error::Unexpected("pending insert".into())));
    let result = submit(&pending, &state, &certificate, &CertificateStatus::Pending);
    assert_failure(
        result,
        "legacy pending pointer and certificate body",
        "pending insert",
    );
}

#[test]
fn submission_checks_settled_status_before_both_writes() {
    let (_tmp, _pending, state) = stores();
    let certificate = Certificate::new_for_test(NetworkId::new(1), Height::ZERO);
    state
        .insert_certificate_header(&certificate, CertificateStatus::Pending)
        .unwrap();
    let header = state.get_certificate_header(&certificate.hash()).unwrap();
    let mut sequence = mockall::Sequence::new();
    let mut state = MockStateStore::new();
    state
        .expect_get_certificate_header()
        .once()
        .in_sequence(&mut sequence)
        .return_once(move |_| Ok(header));
    let mut pending = MockPendingStore::new();
    pending
        .expect_insert_pending_certificate()
        .once()
        .in_sequence(&mut sequence)
        .return_once(|_, _, _| Ok(()));
    state
        .expect_publish_pending_certificate()
        .once()
        .in_sequence(&mut sequence)
        .return_once(|_, _| Ok(()));
    submit(&pending, &state, &certificate, &CertificateStatus::Pending).unwrap();
}

#[test]
fn submission_and_overrides_publish_both_pointers_without_type() {
    let (_tmp, pending, state) = stores();
    let network = NetworkId::new(1);
    let first = Certificate::new_for_test(network, Height::ZERO);
    let next = Certificate::new_for_test(network, Height::new(1));
    for certificate in [&first, &next] {
        {
            submit(&pending, &state, certificate, &CertificateStatus::Pending).unwrap();
        };
        assert_eq!(
            pending
                .get_certificate(network, certificate.height)
                .unwrap()
                .unwrap()
                .hash(),
            certificate.hash()
        );
        assert_eq!(
            state.get_latest_pending_certificate_id(network).unwrap(),
            Some(certificate.hash())
        );
        assert_eq!(
            pending
                .get_latest_pending_certificate_for_network(&network)
                .unwrap(),
            Some((certificate.hash(), certificate.height))
        );
        assert_eq!(
            state
                .get_certificate_header(&certificate.hash())
                .unwrap()
                .unwrap()
                .status,
            CertificateStatus::Pending
        );
        assert_eq!(
            state.get_network_info(network).unwrap().network_type,
            agglayer_types::NetworkType::Unspecified
        );
    }
    {
        assert!(matches!(
            submit(&pending, &state, &first, &CertificateStatus::Pending)
                .unwrap_err()
                .downcast_ref::<Error>(),
            Some(Error::InvalidPendingHeight(..))
        ));
        // Explicit admin overrides may move backwards, unlike submission.
        set_pending(&pending, &state, network, first.height, first.hash()).unwrap();
        set_proven(&pending, &state, network, first.height, first.hash()).unwrap();
        assert_eq!(
            state.get_latest_pending_certificate_id(network).unwrap(),
            Some(first.hash())
        );
        assert_eq!(
            pending
                .get_latest_pending_certificate_for_network(&network)
                .unwrap(),
            Some((first.hash(), first.height))
        );
        assert_eq!(
            state.get_latest_proven_certificate_id(network).unwrap(),
            Some(first.hash())
        );
        assert_eq!(
            pending
                .get_latest_proven_certificate_per_network(&network)
                .unwrap(),
            Some((network, first.height, first.hash()))
        );
    };
}

#[test]
fn submission_rejects_backward_height_using_legacy_fallback_without_cache() {
    let (_tmp, pending, state) = stores();
    let network = NetworkId::new(1);
    let old = Certificate::new_for_test(network, Height::new(10));
    let backwards = Certificate::new_for_test(network, Height::new(9));
    pending
        .set_latest_pending_certificate_per_network(&network, &old.height, &old.hash())
        .unwrap();
    assert_eq!(state.get_latest_pending_height(network).unwrap(), None);
    {
        assert!(
            matches!(submit(&pending, &state, &backwards, &CertificateStatus::Pending).unwrap_err().downcast_ref::<Error>(), Some(Error::InvalidPendingHeight(_, requested, latest)) if *requested == backwards.height && *latest == old.height)
        );
        assert!(pending
            .get_certificate(network, backwards.height)
            .unwrap()
            .is_none());
        assert!(state
            .get_certificate_header(&backwards.hash())
            .unwrap()
            .is_none());
    };
}

#[test]
fn failed_new_submission_batch_keeps_header_absent_and_returns_without_retry() {
    use crate::storage::{DBError, DB};
    let (tmp, pending, state) = stores();
    let network = NetworkId::new(1);
    let first = Certificate::new_for_test(network, Height::ZERO);
    let next = Certificate::new_for_test(network, Height::new(1));
    {
        submit(&pending, &state, &first, &CertificateStatus::Pending).unwrap();
    };
    drop(state);
    let state = StateStore::new(
        Arc::new(
            DB::open_cf_readonly(
                &tmp.path.join("state"),
                crate::stores::state::cf_definitions::STATE_DB,
            )
            .unwrap(),
        ),
        BackupClient::noop(),
    );
    {
        let result = submit(&pending, &state, &next, &CertificateStatus::Pending);
        let error = result.unwrap_err();
        assert_eq!(error.to_string(), "state pending cache");
        assert!(matches!(
            error.downcast_ref::<Error>(),
            Some(Error::DBError(DBError::ReadOnlyMode))
        ));
        assert!(state
            .get_certificate_header(&next.hash())
            .unwrap()
            .is_none());
        assert_eq!(
            state.get_latest_pending_certificate_id(network).unwrap(),
            Some(first.hash())
        );
        assert_eq!(
            pending
                .get_latest_pending_certificate_for_network(&network)
                .unwrap(),
            Some((next.hash(), next.height))
        );
        assert_eq!(
            pending
                .get_certificate(network, next.height)
                .unwrap()
                .unwrap()
                .hash(),
            next.hash()
        );
    };
}

#[test]
fn submission_requests_backup_only_after_successful_pending_insert() {
    for pending_succeeds in [true, false] {
        let tmp = TempDBDir::new();
        let (backup_client, mut backups) = BackupClient::observable();
        let state = StateStore::new_with_path(&tmp.path.join("state"), backup_client).unwrap();
        let certificate = Certificate::new_for_test(NetworkId::new(1), Height::ZERO);
        let mut pending = MockPendingStore::new();
        pending
            .expect_insert_pending_certificate()
            .once()
            .return_once(move |_, _, _| {
                if pending_succeeds {
                    Ok(())
                } else {
                    Err(Error::Unexpected("pending failure".into()))
                }
            });

        let result = submit(&pending, &state, &certificate, &CertificateStatus::Pending);
        if pending_succeeds {
            assert!(result.is_ok());
            assert!(backups.state.try_recv().is_ok());
        } else {
            assert_failure(
                result,
                "legacy pending pointer and certificate body",
                "pending failure",
            );
            assert!(state
                .get_certificate_header(&certificate.hash())
                .unwrap()
                .is_none());
            assert!(backups.state.try_recv().is_err());
        }
        assert!(backups.state.try_recv().is_err());
    }
}

#[test]
fn cache_failure_returns_once_before_legacy_publication() {
    let certificate = Certificate::new_for_test(NetworkId::new(1), Height::ZERO);
    let mut state = MockStateStore::new();
    state
        .expect_set_latest_pending_certificate_pointer()
        .once()
        .return_once(|_, _, _| Err(Error::Unexpected("cache failure".into())));
    // No legacy write is allowed after the cache write fails.
    let pending = MockPendingStore::new();
    let result = set_pending(
        &pending,
        &state,
        certificate.network_id,
        certificate.height,
        certificate.hash(),
    );
    assert_failure(result, "state pending cache", "cache failure");
}

#[test]
fn legacy_failure_returns_once_and_other_pointer_write_does_not_repair_it() {
    let (_tmp, pending, state) = stores();
    let certificate = Certificate::new_for_test(NetworkId::new(1), Height::ZERO);
    let network = certificate.network_id;
    let id = certificate.hash();
    state
        .insert_certificate_header(&certificate, CertificateStatus::Pending)
        .unwrap();
    let mut failing_legacy = MockPendingStore::new();
    failing_legacy
        .expect_set_latest_pending_certificate_per_network()
        .once()
        .return_once(|_, _, _| Err(Error::Unexpected("legacy failure".into())));
    let result = set_pending(&failing_legacy, &state, network, certificate.height, id);
    assert_failure(result, "legacy pending pointer", "legacy failure");
    assert_eq!(
        state.get_latest_pending_certificate_id(network).unwrap(),
        Some(id)
    );
    assert_eq!(
        pending
            .get_latest_pending_certificate_for_network(&network)
            .unwrap(),
        None
    );
    set_proven(&pending, &state, network, certificate.height, id).unwrap();
    assert_eq!(
        pending
            .get_latest_pending_certificate_for_network(&network)
            .unwrap(),
        None
    );
    assert_eq!(
        pending
            .get_latest_proven_certificate_per_network(&network)
            .unwrap(),
        Some((network, certificate.height, id))
    );
}

#[test]
fn settled_submission_is_rejected_before_body_and_pointer_writes() {
    let (_tmp, pending, state) = stores();
    let certificate = Certificate::new_for_test(NetworkId::new(1), Height::ZERO);
    state
        .insert_certificate_header(&certificate, CertificateStatus::Settled)
        .unwrap();
    assert!(matches!(
        submit(&pending, &state, &certificate, &CertificateStatus::Pending)
            .unwrap_err()
            .downcast_ref::<Error>(),
        Some(Error::UnprocessedAction(_))
    ));
    assert!(pending
        .get_certificate(certificate.network_id, certificate.height)
        .unwrap()
        .is_none());
    assert_eq!(
        state
            .get_latest_pending_certificate_id(certificate.network_id)
            .unwrap(),
        None
    );
}

#[test]
fn normal_submissions_serialize_both_database_writes() {
    use std::{sync::mpsc, time::Duration};

    let (_tmp, pending, state) = stores();
    let pending = Arc::new(pending);
    let state = Arc::new(state);
    let network = NetworkId::new(1);
    let first = Certificate::new_for_test(network, Height::ZERO);
    let next = Certificate::new_for_test(network, Height::new(1));
    let first_id = first.hash();
    let next_id = next.hash();
    let (legacy_written, legacy_written_rx) = mpsc::channel();
    let (release, release_rx) = mpsc::channel();
    let mut paused_pending = MockPendingStore::new();
    let first_pending = pending.clone();
    paused_pending
        .expect_insert_pending_certificate()
        .once()
        .return_once(move |network, height, certificate| {
            first_pending.insert_pending_certificate(network, height, certificate)?;
            legacy_written.send(()).unwrap();
            release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            Ok(())
        });
    let first_state = state.clone();
    let writer =
        std::thread::spawn(move || first_state.submit_pending_certificate(&paused_pending, &first));
    legacy_written_rx
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    assert_eq!(
        state.get_latest_pending_certificate_id(network).unwrap(),
        None
    );

    let (started, started_rx) = mpsc::channel();
    let (validated, validated_rx) = mpsc::channel();
    let next_state = state.clone();
    let next_pending = pending.clone();
    let observed_state = state.clone();
    let mut observed_pending = MockPendingStore::new();
    observed_pending
        .expect_insert_pending_certificate()
        .once()
        .return_once(move |network, height, certificate| {
            let published = observed_state.get_latest_pending_certificate_id(network)?;
            validated.send(published).unwrap();
            next_pending.insert_pending_certificate(network, height, certificate)
        });
    let second = std::thread::spawn(move || {
        started.send(()).unwrap();
        next_state.submit_pending_certificate(&observed_pending, &next)
    });
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let prematurely_written = validated_rx.recv_timeout(Duration::from_millis(50));
    release.send(()).unwrap();
    writer.join().unwrap().unwrap();
    second.join().unwrap().unwrap();
    assert!(matches!(
        prematurely_written,
        Err(mpsc::RecvTimeoutError::Timeout)
    ));
    assert_eq!(
        validated_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
        Some(first_id)
    );
    assert_eq!(
        state.get_latest_pending_certificate_id(network).unwrap(),
        Some(next_id)
    );
    assert_eq!(
        pending
            .get_latest_pending_certificate_for_network(&network)
            .unwrap(),
        Some((next_id, Height::new(1)))
    );
}
