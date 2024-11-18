use std::time::Duration;

use agglayer_storage::tests::mocks::{MockPendingStore, MockStateStore};
use agglayer_types::CertificateHeader;
use mockall::predicate::{always, eq};
use rstest::rstest;

use super::*;
use crate::tests::{clock, mocks::MockCertifier};

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(1))]
async fn start_from_zero() {
    let mut pending = MockPendingStore::new();
    let mut state = MockStateStore::new();
    let mut certifier = MockCertifier::new();
    let (certification_notifier, mut receiver) = mpsc::channel(1);
    let clock_ref = clock();
    let network_id = 1.into();
    let (sender, certificate_stream) = mpsc::channel(1);

    let certificate = Certificate::new_for_test(network_id, 0);
    let certificate_id = certificate.hash();

    state
        .expect_get_certificate_header_by_cursor()
        .once()
        .with(eq(network_id), eq(0))
        .returning(|_, _| Ok(None));

    state
        .expect_insert_certificate_header()
        .once()
        .returning(|_, _| Ok(()));

    state
        .expect_read_local_network_state()
        .returning(|_| Ok(Default::default()));

    state
        .expect_write_local_network_state()
        .returning(|_, _, _| Ok(()));

    pending
        .expect_insert_pending_certificate()
        .once()
        .returning(|_, _, _| Ok(()));

    pending
        .expect_get_certificate()
        .once()
        .with(eq(network_id), eq(0))
        .returning(|network_id, height| Ok(Some(Certificate::new_for_test(network_id, height))));

    state
        .expect_get_certificate_header()
        .once()
        .with(eq(certificate_id))
        .returning(|certificate_id| {
            Ok(Some(agglayer_types::CertificateHeader {
                network_id: 1.into(),
                height: 0,
                epoch_number: None,
                certificate_index: None,
                certificate_id: *certificate_id,
                new_local_exit_root: [0; 32].into(),
                metadata: [0; 32].into(),
                status: CertificateStatus::Pending,
            }))
        });

    certifier
        .expect_certify()
        .once()
        .with(always(), eq(network_id), eq(0))
        .return_once(move |new_state, network_id, _height| {
            Ok(Box::pin(async move {
                let result = crate::CertifierOutput {
                    certificate,
                    height: 0,
                    new_state,
                    network: network_id,
                };

                Ok(result)
            }))
        });

    pending
        .expect_set_latest_proven_certificate_per_network()
        .once()
        .with(eq(network_id), eq(0), eq(certificate_id))
        .returning(|_, _, _| Ok(()));
    state
        .expect_update_certificate_header_status()
        .once()
        .with(eq(certificate_id), eq(CertificateStatus::Proven))
        .returning(|_, _| Ok(()));

    let mut task = NetworkTask::new(
        Arc::new(pending),
        Arc::new(state),
        Arc::new(certifier),
        certification_notifier,
        clock_ref,
        network_id,
        certificate_stream,
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
    let mut next_expected_height = 0;

    let (reply_sx, reply_rx) = oneshot::channel();
    let _ = sender
        .send((Certificate::new_for_test(network_id, 0), reply_sx))
        .await;

    tokio::spawn(async move {
        let (sender, cert) = receiver.recv().await.unwrap();

        _ = sender.send(SettledCertificate(cert.0, cert.2, 0, 0));
    });

    task.make_progress(&mut epochs, &mut next_expected_height)
        .await
        .unwrap();

    assert_eq!(next_expected_height, 1);

    assert!(reply_rx.await.unwrap().is_ok());
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(1))]
async fn one_per_epoch() {
    let mut pending = MockPendingStore::new();
    let mut state = MockStateStore::new();
    let mut certifier = MockCertifier::new();
    let (certification_notifier, mut receiver) = mpsc::channel(1);
    let clock_ref = clock();
    let network_id = 1.into();
    let (sender, certificate_stream) = mpsc::channel(100);

    let certificate = Certificate::new_for_test(network_id, 0);
    let certificate2 = Certificate::new_for_test(network_id, 1);
    let certificate_id = certificate.hash();
    let certificate_id2 = certificate2.hash();

    state
        .expect_get_certificate_header_by_cursor()
        .once()
        .with(eq(network_id), eq(0))
        .returning(|_, _| Ok(None));

    state
        .expect_insert_certificate_header()
        .once()
        .returning(|_, _| Ok(()));

    state
        .expect_read_local_network_state()
        .returning(|_| Ok(Default::default()));

    state
        .expect_write_local_network_state()
        .returning(|_, _, _| Ok(()));

    pending
        .expect_insert_pending_certificate()
        .once()
        .returning(|_, _, _| Ok(()));

    pending
        .expect_get_certificate()
        .once()
        .with(eq(network_id), eq(0))
        .returning(|network_id, height| Ok(Some(Certificate::new_for_test(network_id, height))));

    pending
        .expect_get_certificate()
        .never()
        .with(eq(network_id), eq(1))
        .returning(|network_id, height| Ok(Some(Certificate::new_for_test(network_id, height))));
    state
        .expect_get_certificate_header()
        .once()
        .with(eq(certificate_id))
        .returning(|certificate_id| {
            Ok(Some(agglayer_types::CertificateHeader {
                network_id: 1.into(),
                height: 0,
                epoch_number: None,
                certificate_index: None,
                certificate_id: *certificate_id,
                new_local_exit_root: [0; 32].into(),
                metadata: [0; 32].into(),
                status: CertificateStatus::Pending,
            }))
        });

    state
        .expect_get_certificate_header()
        .never()
        .with(eq(certificate_id2))
        .returning(|certificate_id| {
            Ok(Some(agglayer_types::CertificateHeader {
                network_id: 1.into(),
                height: 1,
                epoch_number: None,
                certificate_index: None,
                certificate_id: *certificate_id,
                new_local_exit_root: [0; 32].into(),
                metadata: [0; 32].into(),
                status: CertificateStatus::Pending,
            }))
        });
    certifier
        .expect_certify()
        .once()
        .with(always(), eq(network_id), eq(0))
        .return_once(move |new_state, network_id, _height| {
            Ok(Box::pin(async move {
                let result = crate::CertifierOutput {
                    certificate,
                    height: 0,
                    new_state,
                    network: network_id,
                };

                Ok(result)
            }))
        });

    certifier
        .expect_certify()
        .never()
        .with(always(), eq(network_id), eq(1))
        .return_once(move |new_state, network_id, _height| {
            Ok(Box::pin(async move {
                let result = crate::CertifierOutput {
                    certificate: certificate2,
                    height: 1,
                    new_state,
                    network: network_id,
                };

                Ok(result)
            }))
        });
    pending
        .expect_set_latest_proven_certificate_per_network()
        .once()
        .with(eq(network_id), eq(0), eq(certificate_id))
        .returning(|_, _, _| Ok(()));
    state
        .expect_update_certificate_header_status()
        .once()
        .with(eq(certificate_id), eq(CertificateStatus::Proven))
        .returning(|_, _| Ok(()));

    let mut task = NetworkTask::new(
        Arc::new(pending),
        Arc::new(state),
        Arc::new(certifier),
        certification_notifier,
        clock_ref,
        network_id,
        certificate_stream,
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
    let mut next_expected_height = 0;

    let (reply0_sx, reply0_rx) = oneshot::channel();
    sender
        .send((Certificate::new_for_test(network_id, 0), reply0_sx))
        .await
        .expect("Failed to send the certificate");

    let (reply1_sx, mut reply1_rx) = oneshot::channel();
    sender
        .send((Certificate::new_for_test(network_id, 1), reply1_sx))
        .await
        .expect("Failed to send the certificate");

    tokio::spawn(async move {
        let (sender, cert) = receiver.recv().await.unwrap();

        sender
            .send(SettledCertificate(cert.0, cert.2, 0, 0))
            .expect("Failed to send");
    });

    task.make_progress(&mut epochs, &mut next_expected_height)
        .await
        .unwrap();

    assert_eq!(next_expected_height, 1);

    tokio::time::timeout(
        Duration::from_millis(100),
        task.make_progress(&mut epochs, &mut next_expected_height),
    )
    .await
    .expect_err("Should have timed out");

    assert_eq!(next_expected_height, 1);

    assert!(reply0_rx.await.unwrap().is_ok());
    assert!(reply1_rx.try_recv().is_err());
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(1))]
async fn changing_epoch_triggers_certify() {
    let mut pending = MockPendingStore::new();
    let mut state = MockStateStore::new();
    let mut certifier = MockCertifier::new();
    let (certification_notifier, mut receiver) = mpsc::channel(1);
    let clock_ref = clock();
    let network_id = 1.into();
    let (sender, certificate_stream) = mpsc::channel(100);

    let certificate = Certificate::new_for_test(network_id, 0);
    let certificate2 = Certificate::new_for_test(network_id, 1);
    let certificate_id = certificate.hash();
    let certificate_id2 = certificate2.hash();

    state
        .expect_get_certificate_header_by_cursor()
        .once()
        .with(eq(network_id), eq(0))
        .returning(|_, _| Ok(None));

    state
        .expect_insert_certificate_header()
        .once()
        .returning(|_, _| Ok(()));

    pending
        .expect_insert_pending_certificate()
        .once()
        .returning(|_, _, _| Ok(()));

    pending
        .expect_get_certificate()
        .once()
        .with(eq(network_id), eq(0))
        .returning(|network_id, height| Ok(Some(Certificate::new_for_test(network_id, height))));

    pending
        .expect_get_certificate()
        .once()
        .with(eq(network_id), eq(1))
        .returning(|network_id, height| Ok(Some(Certificate::new_for_test(network_id, height))));

    state
        .expect_read_local_network_state()
        .returning(|_| Ok(Default::default()));

    state
        .expect_write_local_network_state()
        .returning(|_, _, _| Ok(()));

    state
        .expect_get_certificate_header()
        .once()
        .with(eq(certificate_id))
        .returning(|certificate_id| {
            Ok(Some(agglayer_types::CertificateHeader {
                network_id: 1.into(),
                height: 0,
                epoch_number: None,
                certificate_index: None,
                certificate_id: *certificate_id,
                new_local_exit_root: [0; 32].into(),
                metadata: [0; 32].into(),
                status: CertificateStatus::Pending,
            }))
        });

    state
        .expect_get_certificate_header()
        .once()
        .with(eq(certificate_id2))
        .returning(|certificate_id| {
            Ok(Some(agglayer_types::CertificateHeader {
                network_id: 1.into(),
                height: 1,
                epoch_number: None,
                certificate_index: None,
                certificate_id: *certificate_id,
                new_local_exit_root: [0; 32].into(),
                metadata: [0; 32].into(),
                status: CertificateStatus::Pending,
            }))
        });
    certifier
        .expect_certify()
        .once()
        .with(always(), eq(network_id), eq(0))
        .return_once(move |new_state, network_id, _height| {
            Ok(Box::pin(async move {
                let result = crate::CertifierOutput {
                    certificate,
                    height: 0,
                    new_state,
                    network: network_id,
                };

                Ok(result)
            }))
        });

    certifier
        .expect_certify()
        .once()
        .with(always(), eq(network_id), eq(1))
        .return_once(move |new_state, network_id, _height| {
            Ok(Box::pin(async move {
                let result = crate::CertifierOutput {
                    certificate: certificate2,
                    height: 1,
                    new_state,
                    network: network_id,
                };

                Ok(result)
            }))
        });

    pending
        .expect_set_latest_proven_certificate_per_network()
        .once()
        .with(eq(network_id), eq(0), eq(certificate_id))
        .returning(|_, _, _| Ok(()));
    pending
        .expect_set_latest_proven_certificate_per_network()
        .once()
        .with(eq(network_id), eq(1), eq(certificate_id2))
        .returning(|_, _, _| Ok(()));

    state
        .expect_update_certificate_header_status()
        .once()
        .with(eq(certificate_id), eq(CertificateStatus::Proven))
        .returning(|_, _| Ok(()));

    state
        .expect_update_certificate_header_status()
        .once()
        .with(eq(certificate_id2), eq(CertificateStatus::Proven))
        .returning(|_, _| Ok(()));

    let mut task = NetworkTask::new(
        Arc::new(pending),
        Arc::new(state),
        Arc::new(certifier),
        certification_notifier,
        clock_ref.clone(),
        network_id,
        certificate_stream,
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
    let mut next_expected_height = 0;

    let (reply0_sx, reply0_rx) = oneshot::channel();
    sender
        .send((Certificate::new_for_test(network_id, 0), reply0_sx))
        .await
        .expect("Failed to send the certificate");

    let (reply1_sx, mut reply1_rx) = oneshot::channel();
    sender
        .send((Certificate::new_for_test(network_id, 1), reply1_sx))
        .await
        .expect("Failed to send the certificate");

    tokio::spawn(async move {
        let (sender, cert) = receiver.recv().await.unwrap();

        sender
            .send(SettledCertificate(cert.0, cert.2, 0, 0))
            .expect("Failed to send");

        let (sender, cert) = receiver.recv().await.unwrap();

        sender
            .send(SettledCertificate(cert.0, cert.2, 1, 0))
            .expect("Failed to send");
    });

    task.make_progress(&mut epochs, &mut next_expected_height)
        .await
        .unwrap();

    assert_eq!(next_expected_height, 1);

    tokio::time::timeout(
        Duration::from_millis(100),
        task.make_progress(&mut epochs, &mut next_expected_height),
    )
    .await
    .expect_err("Should have timed out");

    assert_eq!(next_expected_height, 1);

    clock_ref
        .get_sender()
        .send(agglayer_clock::Event::EpochEnded(0))
        .expect("Failed to send");

    task.make_progress(&mut epochs, &mut next_expected_height)
        .await
        .unwrap();

    assert_eq!(next_expected_height, 2);

    assert!(reply0_rx.await.unwrap().is_ok());
    assert!(reply1_rx.try_recv().is_err());
}

const fn dummy_cert_header(
    network_id: NetworkId,
    height: u64,
    status: CertificateStatus,
) -> CertificateHeader {
    CertificateHeader {
        network_id,
        height,
        epoch_number: None,
        certificate_index: None,
        certificate_id: agglayer_types::Hash([0xab; 32]),
        new_local_exit_root: agglayer_types::Hash([0xcd; 32]),
        metadata: agglayer_types::Hash([0xef; 32]),
        status,
    }
}

#[rstest]
#[case(None)]
#[case(Some(dummy_cert_header(1.into(), 0, CertificateStatus::InError {
    error: CertificateStatusError::TrustedSequencerNotFound(1.into())
})))]
#[case(Some(dummy_cert_header(1.into(), 0, CertificateStatus::InError {
    error: CertificateStatusError::ProofVerificationFailed(
        agglayer_types::ProofVerificationError::InvalidPublicValues
    )
})))]
#[tokio::test]
#[timeout(Duration::from_secs(1))]
async fn process_certificate_ok(#[case] existing_cert_header: Option<CertificateHeader>) {
    let mut pending = MockPendingStore::new();
    let mut state = MockStateStore::new();
    let certifier = MockCertifier::new();
    let (certification_notifier, mut _receiver) = mpsc::channel(1);
    let clock_ref = clock();
    let network_id = 1.into();
    let (_sender, certificate_stream) = mpsc::channel(100);

    state
        .expect_get_certificate_header_by_cursor()
        .once()
        .with(eq(network_id), eq(0))
        .returning(move |_network_id, _height| Ok(existing_cert_header.clone()));

    state
        .expect_read_local_network_state()
        .returning(|_| Ok(Default::default()));

    state
        .expect_insert_certificate_header()
        .once()
        .returning(|_cert, _status| Ok(()));

    pending
        .expect_insert_pending_certificate()
        .once()
        .returning(|_net, _ht, _cert| Ok(()));

    state
        .expect_write_local_network_state()
        .returning(|_, _, _| Ok(()));

    let mut task = NetworkTask::new(
        Arc::new(pending),
        Arc::new(state),
        Arc::new(certifier),
        certification_notifier,
        clock_ref.clone(),
        network_id,
        certificate_stream,
    )
    .expect("Failed to create a new network task");

    let certificate = Certificate::new_for_test(network_id, 0);
    let result = task.process_certificate(&certificate, 0);
    assert!(result.is_ok());
}

#[rstest]
#[case(CertificateStatus::Proven)]
#[case(CertificateStatus::Pending)]
#[case(CertificateStatus::Candidate)]
#[case(CertificateStatus::Settled)]
#[tokio::test]
#[timeout(Duration::from_secs(1))]
async fn replace_certificate_illegally(#[case] cur_cert_status: CertificateStatus) {
    let pending = MockPendingStore::new();
    let mut state = MockStateStore::new();
    let certifier = MockCertifier::new();
    let (certification_notifier, mut _receiver) = mpsc::channel(1);
    let clock_ref = clock();
    let network_id = 1.into();
    let (_sender, certificate_stream) = mpsc::channel(100);

    state
        .expect_get_certificate_header_by_cursor()
        .once()
        .with(eq(network_id), eq(0))
        .returning(move |network_id, height| {
            Ok(Some(dummy_cert_header(
                network_id,
                height,
                cur_cert_status.clone(),
            )))
        });

    state
        .expect_read_local_network_state()
        .returning(|_| Ok(Default::default()));

    let mut task = NetworkTask::new(
        Arc::new(pending),
        Arc::new(state),
        Arc::new(certifier),
        certification_notifier,
        clock_ref.clone(),
        network_id,
        certificate_stream,
    )
    .expect("Failed to create a new network task");

    let certificate = Certificate::new_for_test(network_id, 0);
    let result = task.process_certificate(&certificate, 0);
    assert!(matches!(
        result.unwrap_err(),
        InitialCheckError::IllegalReplacement { .. }
    ));
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(1))]
async fn timeout_certifier() {
    let mut pending = MockPendingStore::new();
    let mut state = MockStateStore::new();
    let mut certifier = MockCertifier::new();
    let (certification_notifier, mut receiver) = mpsc::channel(1);
    let clock_ref = clock();
    let network_id = 1.into();
    let (sender, certificate_stream) = mpsc::channel(100);

    let certificate = Certificate::new_for_test(network_id, 0);
    let certificate_id = certificate.hash();

    state
        .expect_get_certificate_header_by_cursor()
        .once()
        .with(eq(network_id), eq(0))
        .returning(move |_network_id, _height| Ok(None));

    state
        .expect_insert_certificate_header()
        .once()
        .returning(|_cert, _status| Ok(()));

    pending
        .expect_insert_pending_certificate()
        .once()
        .returning(|_net, _ht, _cert| Ok(()));

    pending
        .expect_get_certificate()
        .once()
        .with(eq(network_id), eq(0))
        .returning(|network_id, height| Ok(Some(Certificate::new_for_test(network_id, height))));

    state
        .expect_get_certificate_header()
        .once()
        .with(eq(certificate_id))
        .returning(|certificate_id| {
            Ok(Some(agglayer_types::CertificateHeader {
                network_id: 1.into(),
                height: 0,
                epoch_number: None,
                certificate_index: None,
                certificate_id: *certificate_id,
                new_local_exit_root: [0; 32].into(),
                metadata: [0; 32].into(),
                status: CertificateStatus::Pending,
            }))
        });

    certifier
        .expect_certify()
        .once()
        .with(always(), eq(network_id), eq(0))
        .return_once(move |_new_state, _network_id, _height| {
            Ok(Box::pin(async move {
                Err(CertificationError::InternalError("TimedOut".to_string()))
            }))
        });

    let expected_error = "Internal error happened in the certification process of \
                          0x911464a00ada773e3659bc570bfecf0819466498cd534f770dc97a6a59774d28: \
                          TimedOut"
        .to_string();

    state
        .expect_update_certificate_header_status()
        .once()
        .with(
            eq(certificate_id),
            eq(CertificateStatus::InError {
                error: CertificateStatusError::InternalError(expected_error),
            }),
        )
        .returning(|_, _| Ok(()));

    state
        .expect_read_local_network_state()
        .returning(|_| Ok(Default::default()));

    let mut task = NetworkTask::new(
        Arc::new(pending),
        Arc::new(state),
        Arc::new(certifier),
        certification_notifier,
        clock_ref.clone(),
        network_id,
        certificate_stream,
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
    let mut next_expected_height = 0;

    let (reply_sx, reply_rx) = oneshot::channel();
    sender
        .send((certificate, reply_sx))
        .await
        .expect("Failed to send the certificate");

    tokio::spawn(async move {
        let (sender, cert) = receiver.recv().await.unwrap();

        sender
            .send(SettledCertificate(cert.0, cert.2, 0, 0))
            .expect("Failed to send");
    });

    task.make_progress(&mut epochs, &mut next_expected_height)
        .await
        .unwrap();

    assert_eq!(next_expected_height, 0);
    assert!(reply_rx.await.unwrap().is_ok());
}
