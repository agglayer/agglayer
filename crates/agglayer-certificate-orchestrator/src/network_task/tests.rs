use std::{collections::VecDeque, sync::Mutex, time::Duration};

use agglayer_storage::{
    stores::{PendingCertificateReader, PendingCertificateWriter, StateWriter},
    tests::{
        mocks::{MockPendingStore, MockStateStore},
        TempDBDir,
    },
};
use agglayer_test_suite::{new_storage, sample_data::USDC, Forest};
use agglayer_types::{
    aggchain_data::CertificateAggchainDataCtx, Certificate, CertificateStatus, L1WitnessCtx,
    Metadata, PessimisticRootInput,
};
use mockall::predicate::{always, eq};
use pessimistic_proof::core::commitment::PessimisticRootCommitmentVersion;
use rstest::rstest;

use super::*;
use crate::{
    settlement_client::MockSettlementClient,
    tests::{clock, mocks::MockCertifier},
    CertificationError, CertifierOutput,
};

mod status;

const SETTLEMENT_TX_HASH_1: SettlementTxHash = SettlementTxHash::new(Digest([1; 32]));
const SETTLEMENT_TX_HASH_2: SettlementTxHash = SettlementTxHash::new(Digest([2; 32]));

// Helper functions to reduce test duplication

fn create_test_certificate(forest: &mut Forest, height: Height) -> Certificate {
    if height == Height::ZERO {
        forest.apply_events(
            &[(USDC, 10.try_into().unwrap())],
            &[(USDC, 1.try_into().unwrap())],
        )
    } else {
        let mut cert = forest.apply_events(&[], &[(USDC, 1.try_into().unwrap())]);
        cert.height = height;
        cert
    }
}

fn setup_certifier_mock(
    certifier: &mut MockCertifier,
    pending_store: Arc<impl PendingCertificateReader + 'static>,
    network_id: NetworkId,
    times: usize,
    specific_height: Option<Height>,
) {
    let mut expectation = certifier.expect_certify();

    if times == 1 {
        expectation = expectation.once();
    } else {
        expectation = expectation.times(times);
    }

    if let Some(height) = specific_height {
        expectation = expectation.with(always(), eq(network_id), eq(height));
    } else {
        expectation = expectation.with(always(), eq(network_id), always());
    }

    expectation.returning(move |mut new_state, network, height| {
        let certificate = pending_store
            .get_certificate(network, height)
            .expect("Failed to get certificate")
            .expect("Certificate not found");

        let signer = agglayer_types::Address::new([0; 20]);
        let ctx_from_l1 = L1WitnessCtx {
            l1_info_root: certificate
                .l1_info_root()
                .expect("Failed to get L1 info root")
                .unwrap_or_default(),
            prev_pessimistic_root: PessimisticRootInput::Computed(
                PessimisticRootCommitmentVersion::V2,
            ),
            aggchain_data_ctx: CertificateAggchainDataCtx::LegacyEcdsa { signer },
        };

        let _ = new_state
            .apply_certificate(&certificate, ctx_from_l1)
            .expect("Failed to apply certificate");

        Ok(CertifierOutput {
            certificate,
            height,
            new_state,
            network,
            new_pp_root: Digest::ZERO,
        })
    });
}

fn setup_settlement_mock(
    settlement_client: &mut MockSettlementClient,
    certificate_id: CertificateId,
    settlement_hash: SettlementTxHash,
    nonce: u64,
    epoch: EpochNumber,
    index: CertificateIndex,
) {
    settlement_client
        .expect_submit_certificate_settlement()
        .once()
        .withf(move |i, _| *i == certificate_id)
        .returning(move |_, _| Ok(settlement_hash));

    settlement_client
        .expect_fetch_settlement_nonce()
        .once()
        .with(eq(settlement_hash))
        .returning(move |_| {
            Ok(Some(NonceInfo {
                nonce,
                previous_max_fee_per_gas: 0,
                previous_max_priority_fee_per_gas: None,
            }))
        });

    settlement_client
        .expect_wait_for_settlement()
        .once()
        .withf(move |t, i| *t == settlement_hash && *i == certificate_id)
        .returning(move |_, _| Ok((epoch, index)));
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(1))]
async fn start_from_zero() {
    let mut pending = MockPendingStore::new();
    let mut state = MockStateStore::new();
    let mut certifier = MockCertifier::new();
    let mut settlement_client = MockSettlementClient::new();
    let clock_ref = clock();
    let network_id = 1.into();
    let (sender, certificate_stream) = mpsc::channel(1);

    let certificate = Certificate::new_for_test(network_id, Height::ZERO);
    let certificate_id = certificate.hash();

    pending
        .expect_get_certificate()
        .once()
        .with(eq(network_id), eq(Height::ZERO))
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
                height: Height::ZERO,
                epoch_number: None,
                certificate_index: None,
                certificate_id: *certificate_id,
                prev_local_exit_root: [1; 32].into(),
                new_local_exit_root: [0; 32].into(),
                metadata: Metadata::ZERO,
                status: CertificateStatus::Pending,
                settlement_tx_hash: None,
            }))
        });

    certifier
        .expect_certify()
        .once()
        .with(always(), eq(network_id), eq(Height::ZERO))
        .return_once(move |new_state, network_id, _height| {
            let result = crate::CertifierOutput {
                certificate,
                height: Height::ZERO,
                new_state,
                network: network_id,
                new_pp_root: Digest::ZERO,
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
        .with(eq(network_id), eq(Height::ZERO), eq(certificate_id))
        .returning(|_, _, _| Ok(()));

    state
        .expect_update_certificate_header_status()
        .once()
        .with(eq(certificate_id), eq(CertificateStatus::Proven))
        .returning(|_, _| Ok(()));

    settlement_client
        .expect_submit_certificate_settlement()
        .once()
        .withf(move |i, _| *i == certificate_id)
        .returning(move |_, _| Ok(SettlementTxHash::for_tests()));

    settlement_client
        .expect_fetch_settlement_nonce()
        .once()
        .with(eq(SettlementTxHash::for_tests()))
        .returning(|_| {
            Ok(Some(NonceInfo {
                nonce: 1,
                previous_max_fee_per_gas: 0,
                previous_max_priority_fee_per_gas: None,
            }))
        });

    state
        .expect_update_settlement_tx_hash()
        .once()
        .withf(move |i, t, _f, _| *i == certificate_id && *t == SettlementTxHash::for_tests())
        .returning(|_, _, _, _| Ok(()));

    settlement_client
        .expect_wait_for_settlement()
        .once()
        .withf(move |t, i| *t == SettlementTxHash::for_tests() && *i == certificate_id)
        .returning(move |_, _| Ok((EpochNumber::ZERO, CertificateIndex::ZERO)));

    state
        .expect_update_certificate_header_status()
        .once()
        .with(eq(certificate_id), eq(CertificateStatus::Settled))
        .returning(|_, _| Ok(()));

    state
        .expect_set_latest_settled_certificate_for_network()
        .once()
        .with(
            eq(network_id),
            eq(Height::ZERO),
            eq(certificate_id),
            eq(EpochNumber::ZERO),
            eq(CertificateIndex::ZERO),
        )
        .returning(|_, _, _, _, _| Ok(()));

    let mut task = NetworkTask::new(
        Arc::new(pending),
        Arc::new(state),
        Arc::new(certifier),
        Arc::new(settlement_client),
        clock_ref,
        network_id,
        certificate_stream,
    )
    .expect("Failed to create a new network task");

    let mut next_expected_height = Height::ZERO;

    let _ = sender
        .send(NewCertificate {
            certificate_id,
            height: Height::ZERO,
        })
        .await;

    let mut first_run = true;
    task.make_progress(
        &mut next_expected_height,
        &mut first_run,
        &CancellationToken::new(),
    )
    .await
    .unwrap();

    assert_eq!(next_expected_height, Height::new(1));
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_secs(1))]
async fn retries() {
    let mut pending = MockPendingStore::new();
    let mut state = MockStateStore::new();
    let mut certifier = MockCertifier::new();
    let mut settlement_client = MockSettlementClient::new();
    let clock_ref = clock();
    let network_id = 1.into();
    let (sender, certificate_stream) = mpsc::channel(100);

    let certificate = Certificate::new_for_test(network_id, Height::ZERO);
    let mut certificate2 = Certificate::new_for_test(network_id, Height::ZERO);
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
        .with(eq(network_id), eq(Height::ZERO))
        .returning(move |_network_id, _height| {
            let cert = certs.lock().unwrap().pop_front().unwrap();
            Ok(Some(cert))
        });

    pending
        .expect_get_certificate()
        .never()
        .with(eq(network_id), eq(Height::new(1)))
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
                height: Height::ZERO,
                epoch_number: None,
                certificate_index: None,
                certificate_id: *certificate_id,
                prev_local_exit_root: [1; 32].into(),
                new_local_exit_root: [0; 32].into(),
                metadata: Metadata::ZERO,
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
                height: Height::ZERO,
                epoch_number: None,
                certificate_index: None,
                certificate_id: *certificate_id,
                prev_local_exit_root: [1; 32].into(),
                new_local_exit_root: [2; 32].into(),
                metadata: Metadata::ZERO,
                status: CertificateStatus::Pending,
                settlement_tx_hash: None,
            }))
        });

    let mut responses = VecDeque::new();
    responses.push_back(crate::CertifierOutput {
        certificate: certificate.clone(),
        height: Height::ZERO,
        new_state: LocalNetworkStateData::default(),
        network: network_id,
        new_pp_root: Digest::ZERO,
    });
    responses.push_back(crate::CertifierOutput {
        certificate: certificate2.clone(),
        height: Height::ZERO,
        new_state: LocalNetworkStateData::default(),
        network: network_id,
        new_pp_root: Digest::ZERO,
    });
    let response_certifier = Arc::new(Mutex::new(responses));

    certifier
        .expect_certify()
        .times(2)
        .with(always(), eq(network_id), eq(Height::ZERO))
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
        .with(always(), eq(network_id), eq(Height::new(1)))
        .return_once(move |new_state, network_id, _height| {
            let result = crate::CertifierOutput {
                certificate: certificate2,
                height: Height::new(1),
                new_state,
                network: network_id,
                new_pp_root: Digest::ZERO,
            };

            Ok(result)
        });

    pending
        .expect_set_latest_proven_certificate_per_network()
        .once()
        .with(eq(network_id), eq(Height::ZERO), eq(certificate_id))
        .returning(|_, _, _| Ok(()));

    pending
        .expect_set_latest_proven_certificate_per_network()
        .once()
        .with(eq(network_id), eq(Height::ZERO), eq(certificate_id2))
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
            eq(CertificateStatus::error(
                CertificateStatusError::InternalError(String::new()),
            )),
        )
        .returning(|_, _| Ok(()));

    // First certificate is failing
    settlement_client
        .expect_submit_certificate_settlement()
        .once()
        .withf(move |i, _| *i == certificate_id)
        .returning(move |_, _| Err(Error::InternalError(String::new())));

    // Mock fetch_last_settled_pp_root for the first certificate (retry scenario)
    settlement_client
        .expect_fetch_last_settled_pp_root()
        .once()
        .with(eq(network_id))
        .returning(|_| Ok(None));

    // Second one (retry) is passing
    settlement_client
        .expect_submit_certificate_settlement()
        .once()
        .withf(move |i, _| *i == certificate_id2)
        .returning(|_, _| Ok(SettlementTxHash::for_tests()));

    settlement_client
        .expect_fetch_settlement_nonce()
        .once()
        .with(eq(SettlementTxHash::for_tests()))
        .returning(|_| {
            Ok(Some(NonceInfo {
                nonce: 1,
                previous_max_fee_per_gas: 0,
                previous_max_priority_fee_per_gas: None,
            }))
        });

    state
        .expect_update_settlement_tx_hash()
        .once()
        .withf(move |i, t, _f, _| *i == certificate_id2 && *t == SettlementTxHash::for_tests())
        .returning(|_, _, _, _| Ok(()));

    settlement_client
        .expect_wait_for_settlement()
        .once()
        .withf(move |t, i| *t == SettlementTxHash::for_tests() && *i == certificate_id2)
        .returning(move |_, _| Ok((EpochNumber::ZERO, CertificateIndex::ZERO)));

    state
        .expect_update_certificate_header_status()
        .once()
        .with(eq(certificate_id2), eq(CertificateStatus::Settled))
        .returning(|_, _| Ok(()));

    state
        .expect_set_latest_settled_certificate_for_network()
        .once()
        .with(
            eq(network_id),
            eq(Height::ZERO),
            eq(certificate_id2),
            eq(EpochNumber::ZERO),
            eq(CertificateIndex::ZERO),
        )
        .returning(|_, _, _, _, _| Ok(()));

    let mut task = NetworkTask::new(
        Arc::new(pending),
        Arc::new(state),
        Arc::new(certifier),
        Arc::new(settlement_client),
        clock_ref,
        network_id,
        certificate_stream,
    )
    .expect("Failed to create a new network task");

    let mut next_expected_height = Height::ZERO;

    sender
        .send(NewCertificate {
            certificate_id,
            height: Height::ZERO,
        })
        .await
        .expect("Failed to send the certificate");

    sender
        .send(NewCertificate {
            certificate_id: certificate_id2,
            height: Height::ZERO,
        })
        .await
        .expect("Failed to send the certificate");

    let mut first_run = true;
    task.make_progress(
        &mut next_expected_height,
        &mut first_run,
        &CancellationToken::new(),
    )
    .await
    .unwrap();

    assert_eq!(next_expected_height, Height::ZERO);

    task.make_progress(
        &mut next_expected_height,
        &mut first_run,
        &CancellationToken::new(),
    )
    .await
    .unwrap();

    assert_eq!(next_expected_height, Height::new(1));
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

    let certificate = Certificate::new_for_test(network_id, Height::ZERO);
    let certificate_id = certificate.hash();

    pending
        .expect_get_certificate()
        .once()
        .with(eq(network_id), eq(Height::ZERO))
        .returning(|network_id, height| Ok(Some(Certificate::new_for_test(network_id, height))));

    state
        .expect_get_certificate_header()
        .once()
        .with(eq(certificate_id))
        .returning(|certificate_id| {
            Ok(Some(agglayer_types::CertificateHeader {
                network_id: 1.into(),
                height: Height::ZERO,
                epoch_number: None,
                certificate_index: None,
                certificate_id: *certificate_id,
                prev_local_exit_root: [1; 32].into(),
                new_local_exit_root: [0; 32].into(),
                metadata: Metadata::ZERO,
                status: CertificateStatus::Pending,
                settlement_tx_hash: None,
            }))
        });

    certifier
        .expect_certify()
        .once()
        .with(always(), eq(network_id), eq(Height::ZERO))
        .return_once(move |_new_state, _network_id, _height| {
            Err(CertificationError::InternalError("TimedOut".to_string()))
        });

    let expected_error = String::from("Internal error: TimedOut");

    state
        .expect_get_latest_settled_certificate_per_network()
        .once()
        .with(eq(network_id))
        .returning(|_| Ok(None));

    state
        .expect_update_certificate_header_status()
        .once()
        .withf(move |id, status| {
            if *id != certificate_id {
                return false;
            }
            let CertificateStatus::InError { error } = status else {
                return false;
            };
            let CertificateStatusError::InternalError(error) = &**error else {
                return false;
            };
            error.starts_with(&expected_error)
        })
        .returning(|_, _| Ok(()));

    state
        .expect_read_local_network_state()
        .returning(|_| Ok(Default::default()));

    let mut task = NetworkTask::new(
        Arc::new(pending),
        Arc::new(state),
        Arc::new(certifier),
        Arc::new(MockSettlementClient::new()),
        clock_ref.clone(),
        network_id,
        certificate_stream,
    )
    .expect("Failed to create a new network task");

    let mut next_expected_height = Height::ZERO;

    sender
        .send(NewCertificate {
            certificate_id,
            height: Height::ZERO,
        })
        .await
        .expect("Failed to send the certificate");
    let mut first_run = true;
    task.make_progress(
        &mut next_expected_height,
        &mut first_run,
        &CancellationToken::new(),
    )
    .await
    .unwrap();

    assert_eq!(next_expected_height, Height::ZERO);
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_secs(2))]
async fn process_next_certificate() {
    let tmp = TempDBDir::new();
    let storage = new_storage(&tmp.path);
    let mut settlement_client = MockSettlementClient::new();
    let mut certifier = MockCertifier::new();
    let clock_ref = clock();
    let network_id = 1.into();
    let (sender, certificate_stream) = mpsc::channel(100);

    let mut forest = Forest::default();

    let certificate = create_test_certificate(&mut forest, Height::ZERO);
    let certificate_id = certificate.hash();
    storage
        .pending
        .insert_pending_certificate(network_id, Height::ZERO, &certificate)
        .expect("unable to insert certificate in pending");

    storage
        .state
        .insert_certificate_header(&certificate, CertificateStatus::Pending)
        .expect("Failed to insert certificate header");

    let certificate2 = create_test_certificate(&mut forest, Height::new(1));
    let certificate_id2 = certificate2.hash();

    storage
        .pending
        .insert_pending_certificate(network_id, Height::new(1), &certificate2)
        .expect("unable to insert certificate in pending");
    storage
        .state
        .insert_certificate_header(&certificate2, CertificateStatus::Pending)
        .expect("Failed to insert certificate header");

    setup_certifier_mock(
        &mut certifier,
        Arc::clone(&storage.pending),
        network_id,
        2,
        None,
    );

    setup_settlement_mock(
        &mut settlement_client,
        certificate_id,
        SETTLEMENT_TX_HASH_1,
        1,
        EpochNumber::ZERO,
        CertificateIndex::ZERO,
    );

    setup_settlement_mock(
        &mut settlement_client,
        certificate_id2,
        SETTLEMENT_TX_HASH_2,
        2,
        EpochNumber::new(1),
        CertificateIndex::ZERO,
    );

    let mut task = NetworkTask::new(
        Arc::clone(&storage.pending),
        Arc::clone(&storage.state),
        Arc::new(certifier),
        Arc::new(settlement_client),
        clock_ref.clone(),
        network_id,
        certificate_stream,
    )
    .expect("Failed to create a new network task");

    let mut next_expected_height = Height::ZERO;
    let mut first_run = false; // Set to false since we're testing certificate processing, not initialization

    // Send both certificate events
    sender
        .send(NewCertificate {
            certificate_id,
            height: Height::ZERO,
        })
        .await
        .expect("Failed to send first certificate");

    sender
        .send(NewCertificate {
            certificate_id: certificate_id2,
            height: Height::new(1),
        })
        .await
        .expect("Failed to send second certificate");

    // Process first certificate
    task.make_progress(
        &mut next_expected_height,
        &mut first_run,
        &CancellationToken::new(),
    )
    .await
    .unwrap();

    assert_eq!(next_expected_height, Height::new(1));

    // Update clock for epoch transition
    clock_ref.update_block_height(2);

    // Process second certificate
    task.make_progress(
        &mut next_expected_height,
        &mut first_run,
        &CancellationToken::new(),
    )
    .await
    .unwrap();

    assert_eq!(next_expected_height, Height::new(2));
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_secs(2))]
async fn multiple_certificates_per_epoch_sequential() {
    let tmp = TempDBDir::new();
    let storage = new_storage(&tmp.path);
    let mut settlement_client = MockSettlementClient::new();
    let mut certifier = MockCertifier::new();
    let clock_ref = clock();
    let network_id = 1.into();
    let (sender, certificate_stream) = mpsc::channel(100);

    let num_certificates = 5;
    let mut forest = Forest::default();
    let mut certificate_ids = Vec::new();

    // Create and store multiple certificates
    for i in 0..num_certificates {
        let certificate = create_test_certificate(&mut forest, Height::new(i));
        let certificate_id = certificate.hash();
        certificate_ids.push(certificate_id);

        storage
            .pending
            .insert_pending_certificate(network_id, Height::new(i), &certificate)
            .expect("unable to insert certificate in pending");

        storage
            .state
            .insert_certificate_header(&certificate, CertificateStatus::Pending)
            .expect("Failed to insert certificate header");
    }

    // Mock certifier to prove ALL certificates
    setup_certifier_mock(
        &mut certifier,
        Arc::clone(&storage.pending),
        network_id,
        num_certificates as usize,
        None,
    );

    // Mock settlement for ALL certificates (now that at_capacity_for_epoch is
    // removed)
    for i in 0..num_certificates {
        let settlement_hash = SettlementTxHash::new(Digest([i as u8; 32]));
        setup_settlement_mock(
            &mut settlement_client,
            certificate_ids[i as usize],
            settlement_hash,
            i + 1,
            EpochNumber::ZERO,
            CertificateIndex::new(i),
        );
    }

    let mut task = NetworkTask::new(
        Arc::clone(&storage.pending),
        Arc::clone(&storage.state),
        Arc::new(certifier),
        Arc::new(settlement_client),
        clock_ref.clone(),
        network_id,
        certificate_stream,
    )
    .expect("Failed to create a new network task");

    let mut next_expected_height = Height::ZERO;

    // Send all certificates
    for i in 0..num_certificates {
        sender
            .send(NewCertificate {
                certificate_id: certificate_ids[i as usize],
                height: Height::new(i),
            })
            .await
            .expect("Failed to send the certificate");
    }

    // Process all certificates - they should all be proven and settled
    let mut first_run = false; // Set to false to process certificates immediately
    for i in 0..num_certificates {
        task.make_progress(
            &mut next_expected_height,
            &mut first_run,
            &CancellationToken::new(),
        )
        .await
        .unwrap();

        // After settling each certificate, next_expected_height should increment
        assert_eq!(next_expected_height, Height::new(i + 1));

        // Verify certificate is settled
        let header = storage
            .state
            .get_certificate_header(&certificate_ids[i as usize])
            .expect("Failed to get certificate header")
            .expect("Certificate header not found");

        assert_eq!(
            header.status,
            CertificateStatus::Settled,
            "Certificate {} should be settled",
            i
        );
    }

    // This test demonstrates that multiple certificates can be processed and
    // settled sequentially in the same epoch (at_capacity_for_epoch removed)
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(1))]
async fn reject_non_sequential_certificates() {
    let mut pending = MockPendingStore::new();
    let mut state = MockStateStore::new();
    let certifier = MockCertifier::new();
    let settlement_client = MockSettlementClient::new();
    let clock_ref = clock();
    let network_id = 1.into();
    let (sender, certificate_stream) = mpsc::channel(100);

    let certificate_height_2 = Certificate::new_for_test(network_id, Height::new(2));
    let certificate_id_2 = certificate_height_2.hash();

    state
        .expect_get_latest_settled_certificate_per_network()
        .once()
        .with(eq(network_id))
        .returning(|network_id| {
            // Network last settled at height 0
            Ok(Some((
                *network_id,
                agglayer_storage::columns::latest_settled_certificate_per_network::SettledCertificate(
                    CertificateId::default(),
                    Height::ZERO,
                    EpochNumber::ZERO,
                    CertificateIndex::ZERO,
                ),
            )))
        });

    state
        .expect_read_local_network_state()
        .returning(|_| Ok(Default::default()));

    // Certificate at height 2 should never be processed (expecting height 1)
    pending
        .expect_get_certificate()
        .never()
        .with(eq(network_id), eq(Height::new(2)))
        .returning(|network_id, height| Ok(Some(Certificate::new_for_test(network_id, height))));

    let mut task = NetworkTask::new(
        Arc::new(pending),
        Arc::new(state),
        Arc::new(certifier),
        Arc::new(settlement_client),
        clock_ref.clone(),
        network_id,
        certificate_stream,
    )
    .expect("Failed to create a new network task");

    let mut next_expected_height = Height::new(1); // Expecting height 1 after settling height 0

    // Try to send certificate at height 2 (skipping height 1)
    sender
        .send(NewCertificate {
            certificate_id: certificate_id_2,
            height: Height::new(2),
        })
        .await
        .expect("Failed to send the certificate");

    // The certificate at height 2 should be rejected because next_expected_height
    // is 1 When make_progress receives a certificate with wrong height, it logs
    // a warning and returns Ok(()) without processing it

    let mut first_run = false;

    // This should complete immediately and return Ok(()) because the certificate
    // height doesn't match next_expected_height
    let result = task
        .make_progress(
            &mut next_expected_height,
            &mut first_run,
            &CancellationToken::new(),
        )
        .await;

    // Should return Ok(()) after rejecting the wrong-height certificate
    assert!(
        result.is_ok(),
        "Should return Ok after rejecting wrong-height certificate"
    );

    // next_expected_height should remain unchanged
    assert_eq!(next_expected_height, Height::new(1));
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(1))]
async fn accept_sequential_certificates_in_order() {
    let tmp = TempDBDir::new();
    let storage = new_storage(&tmp.path);
    let mut certifier = MockCertifier::new();
    let mut settlement_client = MockSettlementClient::new();
    let clock_ref = clock();
    let network_id = 1.into();
    let (sender, certificate_stream) = mpsc::channel(100);

    let mut forest = Forest::default();

    // Create certificates at heights 0 and 1.
    let cert0 = create_test_certificate(&mut forest, Height::ZERO);
    let cert_id_0 = cert0.hash();

    let cert1 = create_test_certificate(&mut forest, Height::new(1));
    let cert_id_1 = cert1.hash();

    storage
        .pending
        .insert_pending_certificate(network_id, Height::ZERO, &cert0)
        .expect("unable to insert certificate");
    storage
        .state
        .insert_certificate_header(&cert0, CertificateStatus::Pending)
        .expect("Failed to insert certificate header");

    storage
        .pending
        .insert_pending_certificate(network_id, Height::new(1), &cert1)
        .expect("unable to insert certificate");
    storage
        .state
        .insert_certificate_header(&cert1, CertificateStatus::Pending)
        .expect("Failed to insert certificate header");

    setup_certifier_mock(
        &mut certifier,
        Arc::clone(&storage.pending),
        network_id,
        2,
        None,
    );

    setup_settlement_mock(
        &mut settlement_client,
        cert_id_0,
        SETTLEMENT_TX_HASH_1,
        1,
        EpochNumber::ZERO,
        CertificateIndex::ZERO,
    );

    setup_settlement_mock(
        &mut settlement_client,
        cert_id_1,
        SETTLEMENT_TX_HASH_2,
        2,
        EpochNumber::ZERO,
        CertificateIndex::new(1),
    );

    let mut task = NetworkTask::new(
        Arc::clone(&storage.pending),
        Arc::clone(&storage.state),
        Arc::new(certifier),
        Arc::new(settlement_client),
        clock_ref.clone(),
        network_id,
        certificate_stream,
    )
    .expect("Failed to create a new network task");

    let mut next_expected_height = Height::ZERO;
    let mut first_run = false;

    // Send certificate at height 1 first; it should be ignored while height 0 is
    // expected.
    sender
        .send(NewCertificate {
            certificate_id: cert_id_1,
            height: Height::new(1),
        })
        .await
        .expect("Failed to send certificate");

    task.make_progress(
        &mut next_expected_height,
        &mut first_run,
        &CancellationToken::new(),
    )
    .await
    .expect("Failed to process out-of-order certificate");

    assert_eq!(next_expected_height, Height::ZERO);
    let header_1 = storage
        .state
        .get_certificate_header(&cert_id_1)
        .expect("Failed to get certificate header")
        .expect("Certificate header not found");
    assert_eq!(header_1.status, CertificateStatus::Pending);

    // Process height 0.
    sender
        .send(NewCertificate {
            certificate_id: cert_id_0,
            height: Height::ZERO,
        })
        .await
        .expect("Failed to send certificate");

    task.make_progress(
        &mut next_expected_height,
        &mut first_run,
        &CancellationToken::new(),
    )
    .await
    .expect("Failed to process first certificate");

    assert_eq!(next_expected_height, Height::new(1));
    let header_0 = storage
        .state
        .get_certificate_header(&cert_id_0)
        .expect("Failed to get certificate header")
        .expect("Certificate header not found");
    assert_eq!(header_0.status, CertificateStatus::Settled);

    // Re-send height 1 now that it is expected.
    sender
        .send(NewCertificate {
            certificate_id: cert_id_1,
            height: Height::new(1),
        })
        .await
        .expect("Failed to send certificate");

    task.make_progress(
        &mut next_expected_height,
        &mut first_run,
        &CancellationToken::new(),
    )
    .await
    .expect("Failed to process second certificate");

    assert_eq!(next_expected_height, Height::new(2));
    let header_1 = storage
        .state
        .get_certificate_header(&cert_id_1)
        .expect("Failed to get certificate header")
        .expect("Certificate header not found");
    assert_eq!(header_1.status, CertificateStatus::Settled);
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_secs(3))]
async fn process_multiple_certificates_across_epochs_from_pending() {
    let tmp = TempDBDir::new();
    let storage = new_storage(&tmp.path);
    let mut settlement_client = MockSettlementClient::new();
    let mut certifier = MockCertifier::new();
    let clock_ref = clock();
    let network_id = 1.into();
    let (sender, certificate_stream) = mpsc::channel(100);

    let num_certificates = 5;
    let mut forest = Forest::default();
    let mut certificate_ids = Vec::new();

    // Create and store ALL certificates in pending store upfront
    for i in 0..num_certificates {
        let certificate = create_test_certificate(&mut forest, Height::new(i));
        let certificate_id = certificate.hash();
        certificate_ids.push(certificate_id);

        storage
            .pending
            .insert_pending_certificate(network_id, Height::new(i), &certificate)
            .expect("unable to insert certificate in pending");

        storage
            .state
            .insert_certificate_header(&certificate, CertificateStatus::Pending)
            .expect("Failed to insert certificate header");
    }

    // Mock certifier to prove ALL certificates
    setup_certifier_mock(
        &mut certifier,
        Arc::clone(&storage.pending),
        network_id,
        num_certificates as usize,
        None,
    );

    // Mock settlement for ALL certificates
    for i in 0..num_certificates {
        let cert_id = certificate_ids[i as usize];
        let settlement_hash = SettlementTxHash::new(Digest([i as u8; 32]));
        let epoch = EpochNumber::new(i / 2); // 2 certificates per epoch
        let index = CertificateIndex::new(i % 2);

        setup_settlement_mock(
            &mut settlement_client,
            cert_id,
            settlement_hash,
            i + 1,
            epoch,
            index,
        );
    }

    let mut task = NetworkTask::new(
        Arc::clone(&storage.pending),
        Arc::clone(&storage.state),
        Arc::new(certifier),
        Arc::new(settlement_client),
        clock_ref.clone(),
        network_id,
        certificate_stream,
    )
    .expect("Failed to create a new network task");

    let mut next_expected_height = Height::ZERO;
    let mut first_run = false; // Set to false to process certificates immediately

    // Process certificates one by one, triggering epoch transitions
    for i in 0..num_certificates {
        // Send the certificate event
        sender
            .send(NewCertificate {
                certificate_id: certificate_ids[i as usize],
                height: Height::new(i),
            })
            .await
            .expect("Failed to send certificate");

        // Process the certificate
        task.make_progress(
            &mut next_expected_height,
            &mut first_run,
            &CancellationToken::new(),
        )
        .await
        .expect("Failed to process certificate");

        // Verify certificate is settled
        let header = storage
            .state
            .get_certificate_header(&certificate_ids[i as usize])
            .expect("Failed to get certificate header")
            .expect("Certificate header not found");
        assert_eq!(
            header.status,
            CertificateStatus::Settled,
            "Certificate {} should be settled",
            i
        );

        // After every 2 certificates, trigger epoch transition
        if i > 0 && i % 2 == 1 {
            clock_ref.update_block_height((i + 1) * 2);
        }

        // Next expected height should increment
        assert_eq!(next_expected_height, Height::new(i + 1));
    }

    // Verify ALL certificates were settled
    for i in 0..num_certificates {
        let header = storage
            .state
            .get_certificate_header(&certificate_ids[i as usize])
            .expect("Failed to get certificate header")
            .expect("Certificate header not found");
        assert_eq!(
            header.status,
            CertificateStatus::Settled,
            "Certificate {} should be settled",
            i
        );
    }

    // This test demonstrates:
    // 1. Multiple certificates can be processed sequentially
    // 2. Epoch transitions don't block certificate processing
    // 3. Certificates are automatically picked from pending store
    // 4. All certificates settle successfully across multiple epochs
}
