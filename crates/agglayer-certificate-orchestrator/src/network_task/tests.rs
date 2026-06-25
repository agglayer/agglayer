use std::{collections::VecDeque, sync::Mutex, time::Duration};

use agglayer_config::settlement_service::SettlementTransactionConfig;
use agglayer_settlement_service::MockSettlementServiceTrait;
use agglayer_storage::{
    stores::{PendingCertificateReader, PendingCertificateWriter, SettlementWriter, StateWriter},
    tests::{
        mocks::{MockPendingStore, MockPerEpochStore, MockStateStore},
        TempDBDir,
    },
};
use agglayer_test_suite::{new_storage, sample_data::USDC, Forest};
use agglayer_types::{
    aggchain_data::CertificateAggchainDataCtx, Certificate, CertificateIndex, CertificateStatus,
    ContractCallOutcome, ContractCallResult, EpochNumber, L1WitnessCtx, Metadata, Nonce,
    PessimisticRootInput, SettlementAttemptNumber, SettlementJobId, SettlementJobResult,
    SettlementTxHash, B256,
};
use arc_swap::ArcSwap;
use mockall::predicate::{always, eq};
use pessimistic_proof::core::commitment::PessimisticRootCommitmentVersion;
use rstest::rstest;
use tokio_util::sync::CancellationToken;

use super::*;
use crate::{
    tests::{clock, mocks::MockCertifier},
    CertificationError, CertifierOutput,
};

fn mock_current_epoch() -> Arc<ArcSwap<MockPerEpochStore>> {
    let mut mock_epoch = MockPerEpochStore::new();
    mock_epoch
        .expect_add_certificate()
        .returning(|_, _| Ok((EpochNumber::ZERO, CertificateIndex::ZERO)));
    mock_epoch.expect_is_epoch_packed().returning(|| false);
    Arc::new(ArcSwap::new(Arc::new(mock_epoch)))
}

fn default_settlement_config() -> Arc<SettlementTransactionConfig> {
    Arc::new(SettlementTransactionConfig::default())
}

fn settlement_result(
    tx_hash: SettlementTxHash,
    outcome: ContractCallOutcome,
) -> SettlementJobResult {
    SettlementJobResult {
        wallet: agglayer_types::Address::from([0u8; 20]),
        nonce: Nonce(0),
        attempt_number: SettlementAttemptNumber(0),
        contract_call_result: ContractCallResult {
            outcome,
            metadata: Default::default(),
            block_hash: B256::ZERO,
            block_number: 0,
            tx_hash,
        },
    }
}

fn job_id(seed: u128) -> SettlementJobId {
    SettlementJobId::from(ulid::Ulid::from(seed))
}

/// A placeholder settlement job, used by tests that need a job persisted in the
/// store (e.g. to set up the certificate↔job-id link for a Candidate
/// certificate recovered after a reboot).
fn dummy_settlement_job() -> agglayer_types::SettlementJob {
    agglayer_types::SettlementJob {
        contract_address: agglayer_types::Address::ZERO,
        calldata: Default::default(),
        eth_value: agglayer_types::U256::ZERO,
        gas_limit: 0,
    }
}

/// Wire the settlement-service mock to behave like the real service against a
/// real store: persist each submitted settlement job (so storage permits the
/// certificate↔job-id link), then resolve every job to the same terminal
/// `outcome`/`tx_hash`. A fresh job id is minted per call so multi-certificate
/// tests get distinct ids.
fn mock_settlement_persisting<S>(
    settlement_service: &mut MockSettlementServiceTrait,
    store: Arc<S>,
    tx_hash: SettlementTxHash,
    outcome: ContractCallOutcome,
) where
    S: SettlementWriter + StateWriter + Send + Sync + 'static,
{
    let next_id = Arc::new(std::sync::atomic::AtomicU64::new(1));
    settlement_service
        .expect_submit_settlement_job()
        .returning(move |certificate_id, job| {
            let id = job_id(next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst) as u128);
            store
                .insert_settlement_job(&id, &job)
                .map_err(|error| eyre::eyre!("{error}"))?;
            // The real service records the certificate <-> job-id link as part of
            // creating the job; mirror it so process_from_candidate can resume.
            store
                .insert_certificate_settlement_job_id(&certificate_id, &id)
                .map_err(|error| eyre::eyre!("{error}"))?;
            Ok(id)
        });
    settlement_service
        .expect_wait_for_settlement()
        .returning(move |_| Ok(settlement_result(tx_hash, outcome.clone())));
}

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

/// Settlement L1-context expectations a certifier mock needs now that the
/// orchestrator builds real settlement calldata from the certificate proof.
fn expect_settlement_l1_context(certifier: &mut MockCertifier) {
    certifier
        .expect_verifier_type()
        .returning(|_| Ok(agglayer_contracts::rollup::VerifierType::Pessimistic));
    certifier
        .expect_rollup_manager_address()
        .returning(|| agglayer_types::Address::new([0; 20]));
    certifier
        .expect_default_l1_info_tree_leaf_count()
        .returning(|| 0);
}

fn setup_certifier_mock(
    certifier: &mut MockCertifier,
    pending_store: Arc<impl PendingCertificateReader + PendingCertificateWriter + 'static>,
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
        pending_store
            .insert_generated_proof(
                &certificate.hash(),
                &agglayer_test_suite::dummy_settlement_proof(),
            )
            .ok();

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

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(1))]
async fn start_from_zero() {
    let mut pending = MockPendingStore::new();
    pending
        .expect_get_proof()
        .returning(|_| Ok(Some(agglayer_test_suite::dummy_settlement_proof())));
    let mut state = MockStateStore::new();
    let mut certifier = MockCertifier::new();
    expect_settlement_l1_context(&mut certifier);
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

    state
        .expect_update_settlement_tx_hash()
        .returning(|_, _, _, _| Ok(()));

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

    state
        .expect_insert_certificate_settlement_job_id()
        .returning(|_, _| Ok(()));
    state
        .expect_get_certificate_settlement_job_id()
        .returning(|_| Ok(Some(job_id(1))));
    state
        .expect_update_certificate_header_status()
        .once()
        .with(eq(certificate_id), eq(CertificateStatus::Candidate))
        .returning(|_, _| Ok(()));

    let mut settlement_service = MockSettlementServiceTrait::new();
    settlement_service
        .expect_submit_settlement_job()
        .returning(|_, _| Ok(job_id(1)));
    settlement_service
        .expect_wait_for_settlement()
        .returning(move |_| {
            Ok(settlement_result(
                SETTLEMENT_TX_HASH_1,
                ContractCallOutcome::Success,
            ))
        });

    let mut task = NetworkTask::new(
        Arc::new(pending),
        Arc::new(state),
        Arc::new(certifier),
        clock_ref,
        network_id,
        certificate_stream,
        Arc::new(settlement_service),
        mock_current_epoch(),
        default_settlement_config(),
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
    let mut next_expected_height = Height::ZERO;

    let _ = sender
        .send(NewCertificate {
            certificate_id,
            height: Height::ZERO,
        })
        .await;

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
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_secs(1))]
async fn retries() {
    let mut pending = MockPendingStore::new();
    pending
        .expect_get_proof()
        .returning(|_| Ok(Some(agglayer_test_suite::dummy_settlement_proof())));
    let mut state = MockStateStore::new();
    let mut certifier = MockCertifier::new();
    expect_settlement_l1_context(&mut certifier);
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

    // First certificate fails, expect error status update
    state
        .expect_update_certificate_header_status()
        .once()
        .withf(move |id, status| {
            *id == certificate_id && matches!(status, CertificateStatus::InError { .. })
        })
        .returning(|_, _| Ok(()));

    state
        .expect_update_settlement_tx_hash()
        .returning(|_, _, _, _| Ok(()));

    state
        .expect_update_certificate_header_status()
        .once()
        .with(eq(certificate_id2), eq(CertificateStatus::Settled))
        .returning(|_, _| Ok(()));

    // Both certificates transition through Candidate before settlement.
    state
        .expect_update_certificate_header_status()
        .times(2)
        .withf(|_, status| matches!(status, CertificateStatus::Candidate))
        .returning(|_, _| Ok(()));

    state
        .expect_insert_certificate_settlement_job_id()
        .returning(|_, _| Ok(()));
    state
        .expect_get_certificate_settlement_job_id()
        .returning(|_| Ok(Some(job_id(1))));

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

    let mut settlement_service = MockSettlementServiceTrait::new();
    settlement_service
        .expect_submit_settlement_job()
        .returning(|_, _| Ok(job_id(1)));
    // First certificate's settlement fails; second succeeds.
    settlement_service
        .expect_wait_for_settlement()
        .times(1)
        .return_once(|_| Err(eyre::eyre!("Simulated failure")));
    settlement_service
        .expect_wait_for_settlement()
        .times(1)
        .return_once(|_| {
            Ok(settlement_result(
                SETTLEMENT_TX_HASH_2,
                ContractCallOutcome::Success,
            ))
        });

    let mut task = NetworkTask::new(
        Arc::new(pending),
        Arc::new(state),
        Arc::new(certifier),
        clock_ref,
        network_id,
        certificate_stream,
        Arc::new(settlement_service),
        mock_current_epoch(),
        default_settlement_config(),
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
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
        &mut epochs,
        &mut next_expected_height,
        &mut first_run,
        &CancellationToken::new(),
    )
    .await
    .unwrap();

    // First certificate should fail - height doesn't advance
    assert_eq!(next_expected_height, Height::ZERO);

    task.make_progress(
        &mut epochs,
        &mut next_expected_height,
        &mut first_run,
        &CancellationToken::new(),
    )
    .await
    .unwrap();

    // Second certificate should succeed - height advances
    assert_eq!(next_expected_height, Height::new(1));
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(1))]
async fn timeout_certifier() {
    let mut pending = MockPendingStore::new();
    pending
        .expect_get_proof()
        .returning(|_| Ok(Some(agglayer_test_suite::dummy_settlement_proof())));
    let mut state = MockStateStore::new();
    let mut certifier = MockCertifier::new();
    expect_settlement_l1_context(&mut certifier);
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

    let settlement_service = MockSettlementServiceTrait::new();
    let mut task = NetworkTask::new(
        Arc::new(pending),
        Arc::new(state),
        Arc::new(certifier),
        clock_ref.clone(),
        network_id,
        certificate_stream,
        Arc::new(settlement_service),
        mock_current_epoch(),
        default_settlement_config(),
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
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
        &mut epochs,
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
    let mut certifier = MockCertifier::new();
    expect_settlement_l1_context(&mut certifier);
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

    let mut settlement_service = MockSettlementServiceTrait::new();
    mock_settlement_persisting(
        &mut settlement_service,
        Arc::clone(&storage.state),
        SETTLEMENT_TX_HASH_1,
        ContractCallOutcome::Success,
    );
    let mut task = NetworkTask::new(
        Arc::clone(&storage.pending),
        Arc::clone(&storage.state),
        Arc::new(certifier),
        clock_ref.clone(),
        network_id,
        certificate_stream,
        Arc::new(settlement_service),
        mock_current_epoch(),
        default_settlement_config(),
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
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
        &mut epochs,
        &mut next_expected_height,
        &mut first_run,
        &CancellationToken::new(),
    )
    .await
    .unwrap();

    assert_eq!(next_expected_height, Height::new(1));

    // Advance the epoch and signal it so the per-epoch settlement backpressure
    // releases before the second certificate.
    clock_ref.update_block_height(2);
    clock_ref
        .get_sender()
        .send(agglayer_clock::Event::EpochEnded(EpochNumber::new(1)))
        .unwrap();

    // Process second certificate
    task.make_progress(
        &mut epochs,
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
    let mut certifier = MockCertifier::new();
    expect_settlement_l1_context(&mut certifier);
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

    let mut settlement_service = MockSettlementServiceTrait::new();
    mock_settlement_persisting(
        &mut settlement_service,
        Arc::clone(&storage.state),
        SETTLEMENT_TX_HASH_1,
        ContractCallOutcome::Success,
    );
    let mut task = NetworkTask::new(
        Arc::clone(&storage.pending),
        Arc::clone(&storage.state),
        Arc::new(certifier),
        clock_ref.clone(),
        network_id,
        certificate_stream,
        Arc::new(settlement_service),
        mock_current_epoch(),
        default_settlement_config(),
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
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

    // Process all certificates - they should all be proven and settled. Only one
    // certificate settles per epoch, so an epoch event must be signalled between
    // settlements to release the per-epoch backpressure.
    let mut first_run = false; // Set to false to process certificates immediately
    for i in 0..num_certificates {
        if i > 0 {
            clock_ref.update_block_height(i + 1);
            clock_ref
                .get_sender()
                .send(agglayer_clock::Event::EpochEnded(EpochNumber::new(i)))
                .unwrap();
        }

        task.make_progress(
            &mut epochs,
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
            "Certificate {i} should be settled"
        );
    }

    // This test demonstrates that multiple certificates can be processed and
    // settled sequentially, one per epoch, with epoch events releasing the
    // per-epoch settlement backpressure between certificates.
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(1))]
async fn reject_non_sequential_certificates() {
    let mut pending = MockPendingStore::new();
    pending
        .expect_get_proof()
        .returning(|_| Ok(Some(agglayer_test_suite::dummy_settlement_proof())));
    let mut state = MockStateStore::new();
    let certifier = MockCertifier::new();
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

    let settlement_service = MockSettlementServiceTrait::new();
    let mut task = NetworkTask::new(
        Arc::new(pending),
        Arc::new(state),
        Arc::new(certifier),
        clock_ref.clone(),
        network_id,
        certificate_stream,
        Arc::new(settlement_service),
        mock_current_epoch(),
        default_settlement_config(),
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
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
            &mut epochs,
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
    expect_settlement_l1_context(&mut certifier);
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

    let mut settlement_service = MockSettlementServiceTrait::new();
    mock_settlement_persisting(
        &mut settlement_service,
        Arc::clone(&storage.state),
        SETTLEMENT_TX_HASH_1,
        ContractCallOutcome::Success,
    );
    let mut task = NetworkTask::new(
        Arc::clone(&storage.pending),
        Arc::clone(&storage.state),
        Arc::new(certifier),
        clock_ref.clone(),
        network_id,
        certificate_stream,
        Arc::new(settlement_service),
        mock_current_epoch(),
        default_settlement_config(),
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
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
        &mut epochs,
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
        &mut epochs,
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

    // Height 0 settled, so the network is at capacity for the epoch. Advance the
    // epoch and signal it so height 1 can be processed.
    clock_ref.update_block_height(2);
    clock_ref
        .get_sender()
        .send(agglayer_clock::Event::EpochEnded(EpochNumber::new(1)))
        .unwrap();

    // Re-send height 1 now that it is expected.
    sender
        .send(NewCertificate {
            certificate_id: cert_id_1,
            height: Height::new(1),
        })
        .await
        .expect("Failed to send certificate");

    task.make_progress(
        &mut epochs,
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
#[ignore = "multi-epoch settlement needs a clock mock that advances current_epoch so the \
            one-per-epoch backpressure resets between certificates; revisit with the settlement \
            test-refactor follow-up"]
async fn process_multiple_certificates_across_epochs_from_pending() {
    let tmp = TempDBDir::new();
    let storage = new_storage(&tmp.path);
    let mut certifier = MockCertifier::new();
    expect_settlement_l1_context(&mut certifier);
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

    let mut settlement_service = MockSettlementServiceTrait::new();
    mock_settlement_persisting(
        &mut settlement_service,
        Arc::clone(&storage.state),
        SETTLEMENT_TX_HASH_1,
        ContractCallOutcome::Success,
    );
    let mut task = NetworkTask::new(
        Arc::clone(&storage.pending),
        Arc::clone(&storage.state),
        Arc::new(certifier),
        clock_ref.clone(),
        network_id,
        certificate_stream,
        Arc::new(settlement_service),
        mock_current_epoch(),
        default_settlement_config(),
    )
    .expect("Failed to create a new network task");

    let mut epochs = task.clock_ref.subscribe().unwrap();
    let mut next_expected_height = Height::ZERO;
    let mut first_run = false; // Set to false to process certificates immediately

    // Process certificates one by one, triggering epoch transitions
    for i in 0..num_certificates {
        // Only one certificate settles per epoch, so advance and signal an epoch
        // before each subsequent certificate to release the backpressure.
        if i > 0 {
            clock_ref.update_block_height((i + 1) * 2);
            clock_ref
                .get_sender()
                .send(agglayer_clock::Event::EpochEnded(EpochNumber::new(i)))
                .unwrap();
        }

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
            &mut epochs,
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
            "Certificate {i} should be settled"
        );

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
            "Certificate {i} should be settled"
        );
    }

    // This test demonstrates:
    // 1. Multiple certificates can be processed sequentially
    // 2. Epoch transitions don't block certificate processing
    // 3. Certificates are automatically picked from pending store
    // 4. All certificates settle successfully across multiple epochs
}
