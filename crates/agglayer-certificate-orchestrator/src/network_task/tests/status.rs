use std::time::Duration;

use agglayer_settlement_service::{
    MockSettlementServiceTrait,
    testutils::mock_settlement_success,
};
use agglayer_storage::{
    stores::{
        PendingCertificateReader, PendingCertificateWriter, StateWriter,
        UpdateEvenIfAlreadyPresent, UpdateStatusToCandidate,
    },
    tests::TempDBDir,
};
use agglayer_test_suite::{new_storage, sample_data::USDC, Forest};
use agglayer_types::{aggchain_data::CertificateAggchainDataCtx, L1WitnessCtx};
use mockall::predicate::{always, eq};
use pessimistic_proof::{
    core::{commitment::PessimisticRootCommitmentVersion, generate_pessimistic_proof},
    LocalNetworkState,
};
use rstest::rstest;
use tokio_util::sync::CancellationToken;

use super::*;
use crate::{
    settlement_client::MockSettlementClient,
    tests::{clock, mocks::MockCertifier},
};

const SETTLEMENT_TX_HASH_TEST: SettlementTxHash = SettlementTxHash::new(Digest([1; 32]));

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

            let state_commitment = new_state.get_roots();
            let pp_commitment_values =
                pessimistic_proof::core::commitment::PessimisticRootCommitmentValues {
                    height: height.as_u64(),
                    origin_network: network_id,
                    ler_leaf_count: state_commitment.ler_leaf_count,
                    balance_root: state_commitment.balance_root.into(),
                    nullifier_root: state_commitment.nullifier_root.into(),
                };
            let pp_root =
                pp_commitment_values.compute_pp_root(PessimisticRootCommitmentVersion::V3);

            Ok(CertifierOutput {
                certificate,
                height,
                new_state,
                network,
                new_pp_root: pp_root,
            })
        });

    let settlement_client = MockSettlementClient::new();

    let mut settlement_service = MockSettlementServiceTrait::new();
    settlement_service
        .expect_request_new_settlement()
        .returning(|_| Ok(mock_settlement_success(SETTLEMENT_TX_HASH_TEST)));

    let mut task = NetworkTask::new(
        Arc::clone(&storage.pending),
        Arc::clone(&storage.state),
        Arc::new(certifier),
        Arc::new(settlement_client),
        clock_ref.clone(),
        network_id,
        certificate_stream,
        Arc::new(settlement_service),
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
    let mut next_expected_height = Height::ZERO;
    let mut first_run = true;
    task.make_progress(
        &mut epochs,
        &mut next_expected_height,
        &mut first_run,
        &CancellationToken::new(),
    )
    .await
    .unwrap();

    assert_eq!(next_expected_height, Height::new(1));

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
            let state_commitment = new_state.get_roots();
            let pp_commitment_values =
                pessimistic_proof::core::commitment::PessimisticRootCommitmentValues {
                    height: height.as_u64(),
                    origin_network: network_id,
                    ler_leaf_count: state_commitment.ler_leaf_count,
                    balance_root: state_commitment.balance_root.into(),
                    nullifier_root: state_commitment.nullifier_root.into(),
                };
            let pp_root =
                pp_commitment_values.compute_pp_root(PessimisticRootCommitmentVersion::V3);

            Ok(CertifierOutput {
                certificate,
                height,
                new_state,
                network,
                new_pp_root: pp_root,
            })
        });

    let settlement_client = MockSettlementClient::new();

    let mut settlement_service = MockSettlementServiceTrait::new();
    settlement_service
        .expect_request_new_settlement()
        .returning(|_| Ok(mock_settlement_success(SETTLEMENT_TX_HASH_TEST)));

    let mut task = NetworkTask::new(
        Arc::clone(&storage.pending),
        Arc::clone(&storage.state),
        Arc::new(certifier),
        Arc::new(settlement_client),
        clock_ref.clone(),
        network_id,
        certificate_stream,
        Arc::new(settlement_service),
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
    let mut next_expected_height = Height::ZERO;
    let mut first_run = true;
    task.make_progress(
        &mut epochs,
        &mut next_expected_height,
        &mut first_run,
        &CancellationToken::new(),
    )
    .await
    .unwrap();

    assert_eq!(next_expected_height, Height::new(1));

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
        .update_settlement_tx_hash(
            &certificate_id,
            SettlementTxHash::for_tests(),
            UpdateEvenIfAlreadyPresent::No,
            UpdateStatusToCandidate::Yes,
        )
        .unwrap();

    certifier.expect_certify().never();
    certifier
        .expect_witness_generation()
        .withf(move |c, _, _| c.hash() == certificate_id)
        .once()
        .returning(move |cert, state, _tx_hash| {
            let initial = LocalNetworkState::from(state.clone());
            let l1_info_root = cert.l1_info_root().unwrap().unwrap_or_default();

            let batch = state
                .apply_certificate(
                    cert,
                    L1WitnessCtx {
                        l1_info_root,
                        prev_pessimistic_root: PessimisticRootInput::Computed(
                            PessimisticRootCommitmentVersion::V2,
                        ),
                        aggchain_data_ctx: CertificateAggchainDataCtx::LegacyEcdsa { signer },
                    },
                )
                .unwrap();

            let (pv, _) = generate_pessimistic_proof(initial.clone().into(), &batch).unwrap();
            Ok((batch, initial, pv))
        });

    let mut settlement_client = MockSettlementClient::new();
    settlement_client
        .expect_fetch_settlement_receipt_status()
        .with(eq(SettlementTxHash::for_tests()))
        .times(1)
        .returning(|_| Ok(crate::TxReceiptStatus::TxSuccessful));

    let mut settlement_service = MockSettlementServiceTrait::new();
    // Candidate status needs both request_new_settlement and retrieve_settlement_result
    settlement_service
        .expect_retrieve_settlement_result()
        .returning(|_| {
            Err(eyre::eyre!("Job not found"))
        });

    let mut task = NetworkTask::new(
        Arc::clone(&storage.pending),
        Arc::clone(&storage.state),
        Arc::new(certifier),
        Arc::new(settlement_client),
        clock_ref.clone(),
        network_id,
        certificate_stream,
        Arc::new(settlement_service),
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
    let mut next_expected_height = Height::ZERO;
    let mut first_run = true;
    task.make_progress(
        &mut epochs,
        &mut next_expected_height,
        &mut first_run,
        &CancellationToken::new(),
    )
    .await
    .unwrap();

    assert_eq!(next_expected_height, Height::new(1));

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

    let settlement_service = MockSettlementServiceTrait::new();
    let mut task = NetworkTask::new(
        Arc::clone(&storage.pending),
        Arc::clone(&storage.state),
        Arc::new(certifier),
        Arc::new(settlement_client),
        clock_ref.clone(),
        network_id,
        certificate_stream,
        Arc::new(settlement_service),
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
    let mut next_expected_height = Height::new(1);
    let mut first_run = true;
    task.make_progress(
        &mut epochs,
        &mut next_expected_height,
        &mut first_run,
        &CancellationToken::new(),
    )
    .await
    .unwrap();

    assert_eq!(next_expected_height, Height::new(1));

    let header = storage
        .state
        .get_certificate_header(&certificate_id)
        .unwrap()
        .unwrap();

    assert!(header.status == CertificateStatus::Settled);
}
