use std::time::Duration;

use agglayer_storage::{
    stores::{PendingCertificateReader, PendingCertificateWriter, StateWriter},
    tests::TempDBDir,
};
use agglayer_test_suite::{new_storage, sample_data::USDC, Forest};
use mockall::predicate::{always, eq};
use pessimistic_proof::{
    core::generate_pessimistic_proof, LocalNetworkState, PessimisticProofOutput,
};
use rstest::rstest;

use super::*;
use crate::{
    settlement_client::MockSettlementClient,
    tests::{clock, mocks::MockCertifier},
};

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_secs(2))]
async fn from_pending_to_settle() {
    let tmp = TempDBDir::new();
    let storage = new_storage(&tmp.path);

    let mut certifier = MockCertifier::new();
    let clock_ref = clock();
    let network_id = 1.into();
    let (_sender, certificate_stream) = mpsc::channel(100);

    let mut forest = Forest::default();

    let certificate = forest.apply_events(
        &[(USDC, 10.try_into().unwrap())],
        &[(USDC, 1.try_into().unwrap())],
    );
    let certificate_id = certificate.hash();
    storage
        .pending
        .insert_pending_certificate(network_id, Height::ZERO, &certificate)
        .expect("unable to insert certificate in pending");

    storage
        .state
        .insert_certificate_header(&certificate, CertificateStatus::Pending)
        .expect("Failed to insert certificate header");

    let pending_store = storage.pending.clone();
    certifier
        .expect_certify()
        .times(1)
        .with(always(), eq(network_id), eq(Height::ZERO))
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
                    PessimisticRootInput::Computed(CommitmentVersion::V2),
                    None,
                )
                .expect("Failed to apply certificate");

            Ok(CertifierOutput {
                certificate,
                height,
                new_state,
                network,
            })
        });

    let mut settlement_client = MockSettlementClient::new();
    settlement_client
        .expect_submit_certificate_settlement()
        .once()
        .withf(move |i| *i == certificate_id)
        .returning(move |_| Ok(SettlementTxHash::for_tests()));
    settlement_client
        .expect_wait_for_settlement()
        .once()
        .withf(move |t, i| *t == SettlementTxHash::for_tests() && *i == certificate_id)
        .returning(move |_, _| Ok((EpochNumber::ZERO, CertificateIndex::ZERO)));

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

    let mut epochs = task.clock_ref.subscribe().unwrap();
    let mut next_expected_height = Height::ZERO;
    let mut first_run = true;
    task.make_progress(&mut epochs, &mut next_expected_height, &mut first_run)
        .await
        .unwrap();

    assert_eq!(next_expected_height, Height::ONE);

    let header = storage
        .state
        .get_certificate_header(&certificate_id)
        .unwrap()
        .unwrap();

    assert!(header.status == CertificateStatus::Settled);
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_secs(2))]
async fn from_proven_to_settled() {
    let tmp = TempDBDir::new();
    let storage = new_storage(&tmp.path);

    let mut certifier = MockCertifier::new();
    let clock_ref = clock();
    let network_id = 1.into();
    let (_sender, certificate_stream) = mpsc::channel(100);

    let mut forest = Forest::default();

    let certificate = forest.apply_events(
        &[(USDC, 10.try_into().unwrap())],
        &[(USDC, 1.try_into().unwrap())],
    );
    let certificate_id = certificate.hash();
    storage
        .pending
        .insert_pending_certificate(network_id, Height::ZERO, &certificate)
        .expect("unable to insert certificate in pending");

    storage
        .state
        .insert_certificate_header(&certificate, CertificateStatus::Proven)
        .expect("Failed to insert certificate header");

    certifier
        .expect_witness_generation()
        .times(1)
        .with(always(), always())
        .returning(move |certificate, new_state| {
            let signer = agglayer_types::Address::new([0; 20]);
            let initial_state = LocalNetworkState::from(new_state.clone());
            let multi_batch_header = new_state
                .apply_certificate(
                    certificate,
                    signer,
                    certificate
                        .l1_info_root()
                        .expect("Failed to get L1 info root")
                        .unwrap_or_default(),
                    PessimisticRootInput::Computed(CommitmentVersion::V2),
                    None,
                )
                .expect("Failed to apply certificate");
            // This proof is obviously wrong, but it's actually unused in this test
            let proof = PessimisticProofOutput {
                prev_local_exit_root: Default::default(),
                prev_pessimistic_root: Default::default(),
                l1_info_root: Default::default(),
                origin_network: NetworkId::ETH_L1,
                aggchain_hash: Default::default(),
                new_local_exit_root: Default::default(),
                new_pessimistic_root: Default::default(),
            };
            Ok((multi_batch_header, initial_state, proof))
        });

    let mut settlement_client = MockSettlementClient::new();
    settlement_client
        .expect_submit_certificate_settlement()
        .once()
        .withf(move |i| *i == certificate_id)
        .returning(move |_| Ok(SettlementTxHash::for_tests()));
    settlement_client
        .expect_wait_for_settlement()
        .once()
        .withf(move |t, i| *t == SettlementTxHash::for_tests() && *i == certificate_id)
        .returning(move |_, _| Ok((EpochNumber::ZERO, CertificateIndex::ZERO)));

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

    let mut epochs = task.clock_ref.subscribe().unwrap();
    let mut next_expected_height = Height::ZERO;
    let mut first_run = true;
    task.make_progress(&mut epochs, &mut next_expected_height, &mut first_run)
        .await
        .unwrap();

    assert_eq!(next_expected_height, Height::ONE);

    let header = storage
        .state
        .get_certificate_header(&certificate_id)
        .unwrap()
        .unwrap();

    assert!(header.status == CertificateStatus::Settled);
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_secs(2))]
async fn from_candidate_to_settle() {
    let tmp = TempDBDir::new();
    let storage = new_storage(&tmp.path);

    let mut certifier = MockCertifier::new();
    let clock_ref = clock();
    let network_id = 1.into();
    let (_sender, certificate_stream) = mpsc::channel(100);

    let mut forest = Forest::default();
    let signer = forest.get_signer();

    let certificate = forest.apply_events(
        &[(USDC, 10.try_into().unwrap())],
        &[(USDC, 1.try_into().unwrap())],
    );
    let certificate_id = certificate.hash();
    storage
        .pending
        .insert_pending_certificate(network_id, Height::ZERO, &certificate)
        .expect("unable to insert certificate in pending");

    storage
        .state
        .insert_certificate_header(&certificate, CertificateStatus::Candidate)
        .expect("Failed to insert certificate header");

    storage
        .state
        .update_settlement_tx_hash(&certificate_id, SettlementTxHash::for_tests())
        .unwrap();

    certifier.expect_certify().never();
    certifier
        .expect_witness_generation()
        .withf(move |c, _| c.hash() == certificate_id)
        .once()
        .returning(move |cert, state| {
            let initial = LocalNetworkState::from(state.clone());
            let l1_info_root = cert.l1_info_root().unwrap().unwrap_or_default();

            let batch = state
                .apply_certificate(
                    cert,
                    signer,
                    l1_info_root,
                    PessimisticRootInput::Computed(CommitmentVersion::V2),
                    None,
                )
                .unwrap();

            let (pv, _) = generate_pessimistic_proof(initial.clone().into(), &batch).unwrap();
            Ok((batch, initial, pv))
        });

    let mut settlement_client = MockSettlementClient::new();
    settlement_client
        .expect_submit_certificate_settlement()
        .never();
    settlement_client
        .expect_wait_for_settlement()
        .with(eq(SettlementTxHash::for_tests()), eq(certificate_id))
        .once()
        .returning(move |_, _| Ok((EpochNumber::ZERO, CertificateIndex::ZERO)));

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

    let mut epochs = task.clock_ref.subscribe().unwrap();
    let mut next_expected_height = Height::ZERO;
    let mut first_run = true;
    task.make_progress(&mut epochs, &mut next_expected_height, &mut first_run)
        .await
        .unwrap();

    assert_eq!(next_expected_height, Height::ONE);

    let header = storage
        .state
        .get_certificate_header(&certificate_id)
        .unwrap()
        .unwrap();

    assert!(header.status == CertificateStatus::Settled);
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_secs(2))]
async fn from_settle_to_settle() {
    let tmp = TempDBDir::new();
    let storage = new_storage(&tmp.path);

    let mut certifier = MockCertifier::new();
    let clock_ref = clock();
    let network_id = 1.into();
    let (_sender, certificate_stream) = mpsc::channel(100);

    let mut forest = Forest::default();

    let certificate = forest.apply_events(
        &[(USDC, 10.try_into().unwrap())],
        &[(USDC, 1.try_into().unwrap())],
    );
    let certificate_id = certificate.hash();
    storage
        .state
        .insert_certificate_header(&certificate, CertificateStatus::Settled)
        .expect("Failed to insert certificate header");

    certifier.expect_certify().never();
    certifier.expect_witness_generation().never();

    let mut settlement_client = MockSettlementClient::new();
    settlement_client
        .expect_submit_certificate_settlement()
        .never();
    settlement_client.expect_wait_for_settlement().never();

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

    let mut epochs = task.clock_ref.subscribe().unwrap();
    let mut next_expected_height = Height::ONE;
    let mut first_run = true;
    task.make_progress(&mut epochs, &mut next_expected_height, &mut first_run)
        .await
        .unwrap();

    assert_eq!(next_expected_height, Height::ONE);

    let header = storage
        .state
        .get_certificate_header(&certificate_id)
        .unwrap()
        .unwrap();

    assert!(header.status == CertificateStatus::Settled);
}
