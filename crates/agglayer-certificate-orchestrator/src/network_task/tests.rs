use std::{collections::VecDeque, sync::Mutex, time::Duration};

use agglayer_storage::{
    stores::{PendingCertificateReader, PendingCertificateWriter, StateWriter},
    tests::{
        mocks::{MockPendingStore, MockStateStore},
        TempDBDir,
    },
};
use agglayer_test_suite::{new_storage, sample_data::USDC, Forest};
use mockall::predicate::{always, eq, in_iter};
use rstest::rstest;

use super::*;
use crate::{
    epoch_packer::MockEpochPacker,
    tests::{clock, mocks::MockCertifier},
};

mod status;

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(1))]
async fn start_from_zero() {
    let mut pending = MockPendingStore::new();
    let mut state = MockStateStore::new();
    let mut certifier = MockCertifier::new();
    let clock_ref = clock();
    let network_id = 1.into();
    let (sender, certificate_stream) = mpsc::channel(1);

    let certificate = Certificate::new_for_test(network_id, 0);
    let certificate_id = certificate.hash();
    pending
        .expect_get_certificate()
        .once()
        .with(eq(network_id), eq(0))
        .returning(|network_id, height| {
            let certificate = Certificate::new_for_test(network_id, height);
            Ok(Some(certificate))
        });

    state
        .expect_get_latest_settled_certificate_per_network()
        .once()
        .with(eq(network_id))
        .returning(|_| Ok(None));

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
                prev_local_exit_root: [1; 32].into(),
                new_local_exit_root: [0; 32].into(),
                metadata: [0; 32].into(),
                status: CertificateStatus::Pending,
                settlement_tx_hash: None,
            }))
        });

    certifier
        .expect_certify()
        .once()
        .with(always(), eq(network_id), eq(0))
        .return_once(move |new_state, network_id, _height| {
            let result = crate::CertifierOutput {
                certificate,
                height: 0,
                new_state,
                network: network_id,
            };

            Ok(result)
        });

    state
        .expect_read_local_network_state()
        .returning(|_| Ok(Default::default()));

    state
        .expect_write_local_network_state()
        .returning(|_, _, _| Ok(()));

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

    let mut packer = MockEpochPacker::new();
    packer
        .expect_settle_certificate()
        .once()
        .withf(move |i| *i == certificate_id)
        .returning(move |c| Ok((network_id, SettledCertificate(c, 0, 0, 0))));

    let mut task = NetworkTask::new(
        Arc::new(pending),
        Arc::new(state),
        Arc::new(certifier),
        Arc::new(packer),
        clock_ref,
        network_id,
        certificate_stream,
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
    let mut next_expected_height = 0;

    let _ = sender
        .send(NewCertificate {
            certificate_id,
            height: 0,
        })
        .await;

    let mut first_run = true;
    task.make_progress(&mut epochs, &mut next_expected_height, &mut first_run)
        .await
        .unwrap();

    assert_eq!(next_expected_height, 1);
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(1))]
async fn one_per_epoch() {
    let mut pending = MockPendingStore::new();
    let mut state = MockStateStore::new();
    let mut certifier = MockCertifier::new();
    let clock_ref = clock();
    let network_id = 1.into();
    let (sender, certificate_stream) = mpsc::channel(100);

    let certificate = Certificate::new_for_test(network_id, 0);
    let certificate2 = Certificate::new_for_test(network_id, 1);
    let certificate_id = certificate.hash();
    let certificate_id2 = certificate2.hash();

    pending
        .expect_get_certificate()
        .once()
        .with(eq(network_id), eq(0))
        .returning(|network_id, height| {
            let c = Certificate::new_for_test(network_id, height);

            Ok(Some(c))
        });

    pending
        .expect_get_certificate()
        .never()
        .with(eq(network_id), eq(1))
        .returning(|network_id, height| {
            let c = Certificate::new_for_test(network_id, height);

            Ok(Some(c))
        });

    state
        .expect_get_latest_settled_certificate_per_network()
        .once()
        .with(eq(network_id))
        .returning(|_| Ok(None));

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
                prev_local_exit_root: [1; 32].into(),
                new_local_exit_root: [0; 32].into(),
                metadata: [0; 32].into(),
                status: CertificateStatus::Pending,
                settlement_tx_hash: None,
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
                prev_local_exit_root: [1; 32].into(),
                new_local_exit_root: [0; 32].into(),
                metadata: [0; 32].into(),
                status: CertificateStatus::Pending,
                settlement_tx_hash: None,
            }))
        });
    certifier
        .expect_certify()
        .once()
        .with(always(), eq(network_id), eq(0))
        .return_once(move |new_state, network_id, _height| {
            let result = crate::CertifierOutput {
                certificate,
                height: 0,
                new_state,
                network: network_id,
            };

            Ok(result)
        });

    state
        .expect_read_local_network_state()
        .returning(|_| Ok(Default::default()));

    state
        .expect_write_local_network_state()
        .returning(|_, _, _| Ok(()));

    certifier
        .expect_certify()
        .never()
        .with(always(), eq(network_id), eq(1))
        .return_once(move |new_state, network_id, _height| {
            let result = crate::CertifierOutput {
                certificate: certificate2,
                height: 1,
                new_state,
                network: network_id,
            };

            Ok(result)
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

    let mut packer = MockEpochPacker::new();
    packer
        .expect_settle_certificate()
        .once()
        .withf(move |i| *i == certificate_id)
        .returning(move |c| Ok((network_id, SettledCertificate(c, 0, 0, 0))));

    let mut task = NetworkTask::new(
        Arc::new(pending),
        Arc::new(state),
        Arc::new(certifier),
        Arc::new(packer),
        clock_ref,
        network_id,
        certificate_stream,
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
    let mut next_expected_height = 0;

    sender
        .send(NewCertificate {
            certificate_id,
            height: 0,
        })
        .await
        .expect("Failed to send the certificate");

    sender
        .send(NewCertificate {
            certificate_id: certificate_id2,
            height: 1,
        })
        .await
        .expect("Failed to send the certificate");

    let mut first_run = true;
    task.make_progress(&mut epochs, &mut next_expected_height, &mut first_run)
        .await
        .unwrap();

    assert_eq!(next_expected_height, 1);
    tokio::time::timeout(
        Duration::from_millis(100),
        task.make_progress(&mut epochs, &mut next_expected_height, &mut first_run),
    )
    .await
    .expect_err("Should have timed out");

    assert_eq!(next_expected_height, 1);
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_secs(1))]
async fn retries() {
    let mut pending = MockPendingStore::new();
    let mut state = MockStateStore::new();
    let mut certifier = MockCertifier::new();
    let clock_ref = clock();
    let network_id = 1.into();
    let (sender, certificate_stream) = mpsc::channel(100);

    let certificate = Certificate::new_for_test(network_id, 0);
    let mut certificate2 = Certificate::new_for_test(network_id, 0);
    certificate2.new_local_exit_root = [2u8; 32].into();

    let certificate_id = certificate.hash();
    let certificate_id2 = certificate2.hash();

    let mut certs = VecDeque::new();
    certs.push_back(certificate.clone());
    certs.push_back(certificate2.clone());
    let certs = Arc::new(Mutex::new(certs));

    pending
        .expect_get_certificate()
        .times(2)
        .with(eq(network_id), eq(0))
        .returning(move |_network_id, _height| {
            let cert = certs.lock().unwrap().pop_front().unwrap();
            Ok(Some(cert))
        });

    pending
        .expect_get_certificate()
        .never()
        .with(eq(network_id), eq(1))
        .returning(|network_id, height| {
            let c = Certificate::new_for_test(network_id, height);
            Ok(Some(c))
        });

    state
        .expect_get_latest_settled_certificate_per_network()
        .once()
        .with(eq(network_id))
        .returning(|_| Ok(None));

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
                prev_local_exit_root: [1; 32].into(),
                new_local_exit_root: [0; 32].into(),
                metadata: [0; 32].into(),
                status: CertificateStatus::Pending,
                settlement_tx_hash: None,
            }))
        });

    state
        .expect_get_certificate_header()
        .once()
        .with(eq(certificate_id2))
        .returning(|certificate_id| {
            Ok(Some(agglayer_types::CertificateHeader {
                network_id: 1.into(),
                height: 0,
                epoch_number: None,
                certificate_index: None,
                certificate_id: *certificate_id,
                prev_local_exit_root: [1; 32].into(),
                new_local_exit_root: [2; 32].into(),
                metadata: [0; 32].into(),
                status: CertificateStatus::Pending,
                settlement_tx_hash: None,
            }))
        });

    let mut responses = VecDeque::new();
    responses.push_back(crate::CertifierOutput {
        certificate: certificate.clone(),
        height: 0,
        new_state: LocalNetworkStateData::default(),
        network: network_id,
    });
    responses.push_back(crate::CertifierOutput {
        certificate: certificate2.clone(),
        height: 0,
        new_state: LocalNetworkStateData::default(),
        network: network_id,
    });
    let response_certifier = Arc::new(Mutex::new(responses));
    certifier
        .expect_certify()
        .times(2)
        .with(always(), eq(network_id), eq(0))
        .returning(move |_new_state, _network_id, _height| {
            let res = response_certifier.lock().unwrap().pop_front().unwrap();
            Ok(res)
        });

    state
        .expect_read_local_network_state()
        .returning(|_| Ok(Default::default()));

    state
        .expect_write_local_network_state()
        .returning(|_, _, _| Ok(()));

    certifier
        .expect_certify()
        .never()
        .with(always(), eq(network_id), eq(1))
        .return_once(move |new_state, network_id, _height| {
            let result = crate::CertifierOutput {
                certificate: certificate2,
                height: 1,
                new_state,
                network: network_id,
            };

            Ok(result)
        });
    pending
        .expect_set_latest_proven_certificate_per_network()
        .once()
        .with(eq(network_id), eq(0), eq(certificate_id))
        .returning(|_, _, _| Ok(()));
    pending
        .expect_set_latest_proven_certificate_per_network()
        .once()
        .with(eq(network_id), eq(0), eq(certificate_id2))
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
    state
        .expect_update_certificate_header_status()
        .once()
        .with(
            eq(certificate_id),
            eq(CertificateStatus::InError {
                error: CertificateStatusError::InternalError(String::new()),
            }),
        )
        .returning(|_, _| Ok(()));

    let mut packer = MockEpochPacker::new();
    // First certificate is failing
    packer
        .expect_settle_certificate()
        .once()
        .withf(move |i| *i == certificate_id)
        .returning(|_| Err(Error::InternalError(String::new())));

    // Second one (retry) is passing
    packer
        .expect_settle_certificate()
        .once()
        .withf(move |i| *i == certificate_id2)
        .returning(move |c| Ok((network_id, SettledCertificate(c, 0, 0, 0))));

    let mut task = NetworkTask::new(
        Arc::new(pending),
        Arc::new(state),
        Arc::new(certifier),
        Arc::new(packer),
        clock_ref,
        network_id,
        certificate_stream,
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
    let mut next_expected_height = 0;

    sender
        .send(NewCertificate {
            certificate_id,
            height: 0,
        })
        .await
        .expect("Failed to send the certificate");

    sender
        .send(NewCertificate {
            certificate_id: certificate_id2,
            height: 0,
        })
        .await
        .expect("Failed to send the certificate");
    let mut first_run = true;
    task.make_progress(&mut epochs, &mut next_expected_height, &mut first_run)
        .await
        .unwrap();

    assert_eq!(next_expected_height, 0);

    task.make_progress(&mut epochs, &mut next_expected_height, &mut first_run)
        .await
        .unwrap();

    assert_eq!(next_expected_height, 1);
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_secs(1))]
async fn changing_epoch_triggers_certify() {
    let mut pending = MockPendingStore::new();
    let mut state = MockStateStore::new();
    let mut certifier = MockCertifier::new();
    let clock_ref = clock();
    let network_id = 1.into();
    let (sender, certificate_stream) = mpsc::channel(100);

    let certificate = Certificate::new_for_test(network_id, 0);
    let certificate2 = Certificate::new_for_test(network_id, 1);
    let certificate_id = certificate.hash();
    let certificate_id2 = certificate2.hash();

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
        .expect_get_latest_settled_certificate_per_network()
        .once()
        .with(eq(network_id))
        .returning(|_| Ok(None));

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
                prev_local_exit_root: [1; 32].into(),
                metadata: [0; 32].into(),
                status: CertificateStatus::Pending,
                settlement_tx_hash: None,
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
                prev_local_exit_root: [1; 32].into(),
                new_local_exit_root: [0; 32].into(),
                metadata: [0; 32].into(),
                status: CertificateStatus::Pending,
                settlement_tx_hash: None,
            }))
        });
    certifier
        .expect_certify()
        .once()
        .with(always(), eq(network_id), eq(0))
        .return_once(move |new_state, network_id, _height| {
            let result = crate::CertifierOutput {
                certificate,
                height: 0,
                new_state,
                network: network_id,
            };

            Ok(result)
        });

    certifier
        .expect_certify()
        .once()
        .with(always(), eq(network_id), eq(1))
        .return_once(move |new_state, network_id, _height| {
            let result = crate::CertifierOutput {
                certificate: certificate2,
                height: 1,
                new_state,
                network: network_id,
            };

            Ok(result)
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

    let mut packer = MockEpochPacker::new();
    packer
        .expect_settle_certificate()
        .once()
        .withf(move |i| *i == certificate_id)
        .returning(move |c| Ok((network_id, SettledCertificate(c, 0, 0, 0))));

    packer
        .expect_settle_certificate()
        .once()
        .withf(move |i| *i == certificate_id2)
        .returning(move |c| Ok((network_id, SettledCertificate(c, 1, 1, 0))));

    let mut task = NetworkTask::new(
        Arc::new(pending),
        Arc::new(state),
        Arc::new(certifier),
        Arc::new(packer),
        clock_ref.clone(),
        network_id,
        certificate_stream,
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
    let mut next_expected_height = 0;

    sender
        .send(NewCertificate {
            certificate_id,
            height: 0,
        })
        .await
        .expect("Failed to send the certificate");

    sender
        .send(NewCertificate {
            certificate_id: certificate_id2,
            height: 1,
        })
        .await
        .expect("Failed to send the certificate");
    let mut first_run = true;
    task.make_progress(&mut epochs, &mut next_expected_height, &mut first_run)
        .await
        .unwrap();

    assert_eq!(next_expected_height, 1);

    tokio::time::timeout(
        Duration::from_millis(100),
        task.make_progress(&mut epochs, &mut next_expected_height, &mut first_run),
    )
    .await
    .expect_err("Should have timed out");

    assert_eq!(next_expected_height, 1);

    clock_ref.update_block_height(2);

    clock_ref
        .get_sender()
        .send(agglayer_clock::Event::EpochEnded(0))
        .expect("Failed to send");
    let mut first_run = true;
    task.make_progress(&mut epochs, &mut next_expected_height, &mut first_run)
        .await
        .unwrap();

    assert_eq!(next_expected_height, 2);
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(1))]
async fn timeout_certifier() {
    let mut pending = MockPendingStore::new();
    let mut state = MockStateStore::new();
    let mut certifier = MockCertifier::new();
    let clock_ref = clock();
    let network_id = 1.into();
    let (sender, certificate_stream) = mpsc::channel(100);

    let certificate = Certificate::new_for_test(network_id, 0);
    let certificate_id = certificate.hash();

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
                prev_local_exit_root: [1; 32].into(),
                new_local_exit_root: [0; 32].into(),
                metadata: [0; 32].into(),
                status: CertificateStatus::Pending,
                settlement_tx_hash: None,
            }))
        });

    certifier
        .expect_certify()
        .once()
        .with(always(), eq(network_id), eq(0))
        .return_once(move |_new_state, _network_id, _height| {
            Err(CertificationError::InternalError("TimedOut".to_string()))
        });

    let expected_error = format!(
        "Internal error happened in the certification process of {}: TimedOut",
        certificate_id
    );

    state
        .expect_get_latest_settled_certificate_per_network()
        .once()
        .with(eq(network_id))
        .returning(|_| Ok(None));

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
        Arc::new(MockEpochPacker::new()),
        clock_ref.clone(),
        network_id,
        certificate_stream,
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
    let mut next_expected_height = 0;

    sender
        .send(NewCertificate {
            certificate_id,
            height: 0,
        })
        .await
        .expect("Failed to send the certificate");
    let mut first_run = true;
    task.make_progress(&mut epochs, &mut next_expected_height, &mut first_run)
        .await
        .unwrap();

    assert_eq!(next_expected_height, 0);
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_secs(2))]
async fn process_next_certificate() {
    let tmp = TempDBDir::new();
    let storage = new_storage(&tmp.path);

    let mut certifier = MockCertifier::new();
    let clock_ref = clock();
    let clock_sender = clock_ref.get_sender();
    let network_id = 1.into();
    let (sender, certificate_stream) = mpsc::channel(100);

    let mut forest = Forest::default();

    let certificate = forest.apply_events(
        &[(USDC, 10.try_into().unwrap())],
        &[(USDC, 1.try_into().unwrap())],
    );
    let certificate_id = certificate.hash();
    storage
        .pending
        .insert_pending_certificate(network_id, 0, &certificate)
        .expect("unable to insert certificate in pending");

    storage
        .state
        .insert_certificate_header(&certificate, CertificateStatus::Pending)
        .expect("Failed to insert certificate header");

    let mut certificate = forest.apply_events(&[], &[(USDC, 1.try_into().unwrap())]);
    certificate.height = 1;
    let certificate_id2 = certificate.hash();

    storage
        .pending
        .insert_pending_certificate(network_id, 1, &certificate)
        .expect("unable to insert certificate in pending");
    storage
        .state
        .insert_certificate_header(&certificate, CertificateStatus::Pending)
        .expect("Failed to insert certificate header");

    let pending_store = storage.pending.clone();
    certifier
        .expect_certify()
        .times(2)
        .with(always(), eq(network_id), in_iter(vec![0, 1]))
        .returning(move |mut new_state, network, height| {
            let certificate = pending_store
                .get_certificate(network, height)
                .expect("Failed to get certificate")
                .expect("Certificate not found");

            let signer = agglayer_types::Address::new([0; 20]);
            let _ = new_state
                .apply_certificate(
                    &certificate,
                    signer,
                    certificate
                        .l1_info_root()
                        .expect("Failed to get L1 info root")
                        .unwrap_or_default(),
                )
                .expect("Failed to apply certificate");

            Ok(CertifierOutput {
                certificate,
                height,
                new_state,
                network,
            })
        });

    let mut packer = MockEpochPacker::new();
    packer
        .expect_settle_certificate()
        .once()
        .withf(move |i| *i == certificate_id)
        .returning(move |c| Ok((network_id, SettledCertificate(c, 0, 0, 0))));
    packer
        .expect_settle_certificate()
        .once()
        .withf(move |i| *i == certificate_id2)
        .returning(move |c| Ok((network_id, SettledCertificate(c, 1, 1, 0))));

    let mut task = NetworkTask::new(
        Arc::clone(&storage.pending),
        Arc::clone(&storage.state),
        Arc::new(certifier),
        Arc::new(packer),
        clock_ref.clone(),
        network_id,
        certificate_stream,
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
    let mut next_expected_height = 0;

    sender
        .send(NewCertificate {
            certificate_id,
            height: 0,
        })
        .await
        .expect("Failed to send the certificate");
    let mut first_run = true;
    task.make_progress(&mut epochs, &mut next_expected_height, &mut first_run)
        .await
        .unwrap();

    assert_eq!(next_expected_height, 1);
    clock_ref.update_block_height(2);
    _ = clock_sender.send(agglayer_clock::Event::EpochEnded(0));

    task.make_progress(&mut epochs, &mut next_expected_height, &mut first_run)
        .await
        .unwrap();
    assert_eq!(next_expected_height, 2);
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_secs(1))]
async fn epoch_jammed(#[values(false, true)] at_capacity: bool) {
    let mut pending = MockPendingStore::new();
    let mut state = MockStateStore::new();
    let certifier = MockCertifier::new();
    let packer = MockEpochPacker::new();
    let clock_ref = clock();
    let epoch_sender = clock_ref.get_sender();
    let network_id = 1.into();
    let (_sender, certificate_stream) = mpsc::channel(1);

    state
        .expect_read_local_network_state()
        .returning(|_| Ok(Default::default()));

    state
        .expect_get_latest_settled_certificate_per_network()
        .once()
        .with(eq(network_id))
        .returning(|_| Ok(None));

    pending.expect_get_certificate().returning(|_, _| Ok(None));

    let mut task = NetworkTask::new(
        Arc::new(pending),
        Arc::new(state),
        Arc::new(certifier),
        Arc::new(packer),
        clock_ref.clone(),
        network_id,
        certificate_stream,
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
    let mut next_expected_height = 0;

    // Jam the epoch channel with a bunch of epoch events.
    for epoch_no in 1..=105 {
        epoch_sender
            .send(agglayer_clock::Event::EpochEnded(epoch_no))
            .unwrap();
    }

    // Just make sure it does not panic or time out when epoch events are skipped.
    let mut first_run = false;
    task.at_capacity_for_epoch = at_capacity;
    task.make_progress(&mut epochs, &mut next_expected_height, &mut first_run)
        .await
        .unwrap();
    assert_eq!(task.at_capacity_for_epoch, at_capacity);

    // Taking the next item from the channel should advance the epoch.
    task.make_progress(&mut epochs, &mut next_expected_height, &mut first_run)
        .await
        .unwrap();
    assert!(!task.at_capacity_for_epoch);
}
