use std::time::Duration;

use agglayer_storage::{
    stores::{PendingCertificateReader, PendingCertificateWriter, StateWriter},
    tests::TempDBDir,
};
use agglayer_test_suite::{new_storage, sample_data::USDC, Forest};
use ethers::types::H256;
use mockall::predicate::{always, eq};
use pessimistic_proof::LocalNetworkState;
use rstest::rstest;

use super::*;
use crate::{
    epoch_packer::MockEpochPacker,
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
        .insert_pending_certificate(network_id, 0, &certificate)
        .expect("unable to insert certificate in pending");

    storage
        .state
        .insert_certificate_header(&certificate, CertificateStatus::Pending)
        .expect("Failed to insert certificate header");

    let pending_store = storage.pending.clone();
    certifier
        .expect_certify()
        .times(1)
        .with(always(), eq(network_id), eq(0))
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
    let state_store = Arc::clone(&storage.state);
    packer
        .expect_settle_certificate()
        .once()
        .withf(move |i| *i == certificate_id)
        .returning(move |c| {
            state_store
                .update_settlement_tx_hash(&c, Digest::ZERO)
                .unwrap();
            state_store
                .update_certificate_header_status(&c, &CertificateStatus::Settled)
                .unwrap();

            Ok((network_id, SettledCertificate(c, 0, 0, 0)))
        });

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
    let mut first_run = true;
    task.make_progress(&mut epochs, &mut next_expected_height, &mut first_run)
        .await
        .unwrap();

    assert_eq!(next_expected_height, 1);

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
async fn from_proven_to_settle() {
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
        .insert_pending_certificate(network_id, 0, &certificate)
        .expect("unable to insert certificate in pending");

    storage
        .state
        .insert_certificate_header(&certificate, CertificateStatus::Proven)
        .expect("Failed to insert certificate header");

    let pending_store = storage.pending.clone();
    certifier
        .expect_certify()
        .times(1)
        .with(always(), eq(network_id), eq(0))
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
    let state_store = Arc::clone(&storage.state);
    packer
        .expect_settle_certificate()
        .once()
        .withf(move |i| *i == certificate_id)
        .returning(move |c| {
            state_store
                .update_settlement_tx_hash(&c, Digest::ZERO)
                .unwrap();
            state_store
                .update_certificate_header_status(&c, &CertificateStatus::Settled)
                .unwrap();

            Ok((network_id, SettledCertificate(c, 0, 0, 0)))
        });

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
    let mut first_run = true;
    task.make_progress(&mut epochs, &mut next_expected_height, &mut first_run)
        .await
        .unwrap();

    assert_eq!(next_expected_height, 1);

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
        .insert_pending_certificate(network_id, 0, &certificate)
        .expect("unable to insert certificate in pending");

    storage
        .state
        .insert_certificate_header(&certificate, CertificateStatus::Candidate)
        .expect("Failed to insert certificate header");

    storage
        .state
        .update_settlement_tx_hash(&certificate_id, Digest::ZERO)
        .unwrap();

    certifier.expect_certify().never();
    certifier
        .expect_witness_execution()
        .withf(move |c, _| c.hash() == certificate_id)
        .once()
        .returning(move |cert, state| {
            let initial = LocalNetworkState::from(state.clone());
            let l1_info_root = cert.l1_info_root().unwrap().unwrap_or_default();

            let batch = state.apply_certificate(cert, signer, l1_info_root).unwrap();

            Ok((batch, initial))
        });

    let state_store = storage.state.clone();
    let mut packer = MockEpochPacker::new();
    packer.expect_settle_certificate().never();
    packer
        .expect_recover_settlement()
        .with(eq(H256::zero()), eq(certificate_id), eq(network_id), eq(0))
        .once()
        .returning(move |_, certificate_id, network_id, height| {
            state_store
                .update_certificate_header_status(&certificate_id, &CertificateStatus::Settled)
                .unwrap();

            Ok((network_id, SettledCertificate(certificate_id, height, 0, 0)))
        });
    packer
        .expect_transaction_exists()
        .once()
        .with(eq(H256::zero()))
        .returning(|_| Ok(true));

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
    let mut first_run = true;
    task.make_progress(&mut epochs, &mut next_expected_height, &mut first_run)
        .await
        .unwrap();

    assert_eq!(next_expected_height, 1);

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
    certifier.expect_witness_execution().never();

    let mut packer = MockEpochPacker::new();
    packer.expect_settle_certificate().never();
    packer.expect_recover_settlement().never();
    packer.expect_transaction_exists().never();

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
    let mut next_expected_height = 1;
    let mut first_run = true;
    task.make_progress(&mut epochs, &mut next_expected_height, &mut first_run)
        .await
        .unwrap();

    assert_eq!(next_expected_height, 1);

    let header = storage
        .state
        .get_certificate_header(&certificate_id)
        .unwrap()
        .unwrap();

    assert!(header.status == CertificateStatus::Settled);
}
