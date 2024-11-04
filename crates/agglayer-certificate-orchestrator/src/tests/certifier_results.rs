use std::time::Duration;

use agglayer_storage::tests::mocks::MockPendingStore;
use agglayer_storage::tests::mocks::MockPerEpochStore;
use agglayer_storage::tests::mocks::MockStateStore;
use agglayer_types::Certificate;
use agglayer_types::CertificateHeader;
use agglayer_types::CertificateStatus;
use agglayer_types::CertificateStatusError;
use agglayer_types::LocalNetworkStateData;
use agglayer_types::NetworkId;
use mockall::predicate::always;
use mockall::predicate::eq;
use mockall::predicate::le;
use mockall::Sequence;
use rstest::rstest;
use tokio::time::sleep;

use crate::tests::create_orchestrator_mock;
use crate::tests::mocks::MockCertifier;
use crate::tests::MockOrchestrator;

#[rstest]
#[tokio::test]
#[timeout(Duration::from_millis(100))]
async fn certifier_results_for_unknown_network_with_height_zero() {
    let network_id: NetworkId = 1.into();

    let certificate = Certificate::new_for_test(network_id, 0);
    let certificate2 = Certificate::new_for_test(network_id, 1);

    let certificate_id = certificate.hash();
    let certificate_id2 = certificate2.hash();

    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
    let mut seq = Sequence::new();

    let mut certifier_mock = MockCertifier::new();

    let mut state_store = MockStateStore::new();
    let mut pending_store = MockPendingStore::new();
    let mut current_epoch_store = MockPerEpochStore::new();

    pending_store
        .expect_get_current_proven_height_for_network()
        .once()
        .in_sequence(&mut seq)
        .with(eq(network_id))
        .returning(|_| Ok(None));

    // We expect one call to the pending store to set the latest proven certificate.
    // Having this executed means that the expected certificate ID was set as
    // the latest proven certificate for this network.
    pending_store
        .expect_set_latest_proven_certificate_per_network()
        .once()
        .in_sequence(&mut seq)
        .with(eq(network_id), eq(0), eq(certificate_id))
        .returning(|_, _, _| Ok(()));

    // We expect one call to the state store to update the certificate header status
    // to proven. This will change as the `add_certificate` will handle the
    // decision of move the certificate to the current epoch or not.
    state_store
        .expect_update_certificate_header_status()
        .with(eq(certificate_id), eq(&CertificateStatus::Proven))
        .once()
        .in_sequence(&mut seq)
        .returning(|_, _| Ok(()));

    // We expect one call to the current epoch store to add the certificate to the
    // current epoch.
    current_epoch_store
        .expect_add_certificate()
        .once()
        .in_sequence(&mut seq)
        .with(eq(network_id), eq(0))
        .returning(|_, _| Ok((0, 0)));

    // We expect one call to the certifier for the certificate2 as we forge the
    // certificate one result.
    certifier_mock
        .expect_certify()
        // Don't check the state as it doesn't implement eq
        .with(always(), eq(network_id), eq(1))
        .once()
        .in_sequence(&mut seq)
        .return_once(move |new_state, network_id, _height| {
            Ok(Box::pin(async move {
                let result = crate::CertifierOutput {
                    certificate: certificate2,
                    height: 1,
                    new_state,
                    network: network_id,
                };
                sender.send(result.clone()).await.unwrap();
                Ok(result)
            }))
        });

    pending_store
        .expect_get_current_proven_height_for_network()
        .once()
        .in_sequence(&mut seq)
        .with(eq(network_id))
        .returning(|_| Ok(Some(0)));

    // This one is expected as we spawn the second certificate.
    // And it validates every steps of the process.
    pending_store
        .expect_set_latest_proven_certificate_per_network()
        .once()
        .in_sequence(&mut seq)
        .with(eq(network_id), eq(1), eq(certificate_id2))
        .returning(|_, _, _| Ok(()));

    // This status change is expected but as mention above, it'll be removed at some
    // point.
    state_store
        .expect_update_certificate_header_status()
        .with(eq(certificate_id2), eq(&CertificateStatus::Proven))
        .once()
        .in_sequence(&mut seq)
        .returning(|_, _| Ok(()));

    current_epoch_store
        .expect_add_certificate()
        .once()
        .in_sequence(&mut seq)
        .with(eq(network_id), eq(1))
        .returning(|network_id, height| {
            Err(agglayer_storage::error::Error::CertificateCandidateError(
                agglayer_storage::error::CertificateCandidateError::UnexpectedHeight(
                    network_id, height, 0,
                ),
            ))
        });

    let builder = MockOrchestrator::builder()
        .certifier(certifier_mock)
        .pending_store(pending_store)
        .state_store(state_store)
        .current_epoch(current_epoch_store)
        .build();

    let (_data_sender, mut orchestrator) = create_orchestrator_mock::partial_1(builder);

    let state = LocalNetworkStateData::default();
    let result = orchestrator.handle_certifier_result(Ok(crate::CertifierOutput {
        certificate,
        height: 0,
        new_state: state,
        network: network_id,
    }));

    assert!(result.is_ok());

    let result = receiver.recv().await.expect("output not present");

    let result = orchestrator.handle_certifier_result(Ok(result));

    assert!(result.is_ok());
}

#[tokio::test]
async fn certifier_results_for_unknown_network_with_height_not_zero() {
    let network_id: NetworkId = 1.into();

    let certificate = Certificate::new_for_test(network_id, 1);
    let certificate_id = certificate.hash();

    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
    let mut seq = Sequence::new();

    let mut certifier_mock = MockCertifier::new();
    let mut pending_store = MockPendingStore::new();

    pending_store
        .expect_get_current_proven_height_for_network()
        .once()
        .in_sequence(&mut seq)
        .with(eq(network_id))
        .returning(|_| Ok(None));

    pending_store
        .expect_remove_pending_certificate()
        .once()
        .in_sequence(&mut seq)
        .with(eq(network_id), eq(1))
        .returning(|_, _| Ok(()));

    pending_store
        .expect_remove_generated_proof()
        .once()
        .in_sequence(&mut seq)
        .with(eq(certificate_id))
        .returning(|_| Ok(()));

    // We do not expect a call to the certifier.
    certifier_mock
        .expect_certify()
        .never()
        .return_once(move |new_state, network_id, _height| {
            Ok(Box::pin(async move {
                let result = crate::CertifierOutput {
                    certificate: Certificate::new_for_test(network_id, 1),
                    height: 1,
                    new_state,
                    network: network_id,
                };
                sender.send(result.clone()).await.unwrap();
                Ok(result)
            }))
        });

    let builder = MockOrchestrator::builder()
        .certifier(certifier_mock)
        .pending_store(pending_store)
        .build();

    let (_data_sender, mut orchestrator) = create_orchestrator_mock::partial_1(builder);

    let state = LocalNetworkStateData::default();
    let result = orchestrator.handle_certifier_result(Ok(crate::CertifierOutput {
        certificate,
        height: 1,
        new_state: state,
        network: network_id,
    }));

    assert!(result.is_err());
    assert!(!orchestrator.global_state.contains_key(&network_id));

    sleep(Duration::from_millis(10)).await;
    assert!(receiver.try_recv().is_err());
}

#[test_log::test(tokio::test)]
async fn certifier_error_certificate_does_not_exists() {
    let network_id: NetworkId = 1.into();
    let mut certifier_mock = MockCertifier::new();

    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
    // We do not expect a call to the certifier.
    certifier_mock
        .expect_certify()
        .never()
        .return_once(move |new_state, network_id, _height| {
            Ok(Box::pin(async move {
                let result = crate::CertifierOutput {
                    certificate: Certificate::new_for_test(network_id, 1),
                    height: 1,
                    new_state,
                    network: network_id,
                };
                sender.send(result.clone()).await.unwrap();
                Ok(result)
            }))
        });

    let builder = MockOrchestrator::builder()
        .certifier(certifier_mock)
        .build();

    let (_data_sender, mut orchestrator) = create_orchestrator_mock::partial_1(builder);

    let result =
        orchestrator.handle_certifier_result(Err(crate::Error::CertificateNotFound(network_id, 0)));

    assert!(result.is_ok());
    assert!(!orchestrator.global_state.contains_key(&network_id));

    sleep(Duration::from_millis(10)).await;
    assert!(receiver.try_recv().is_err());
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_millis(100))]
async fn certifier_error_proof_already_exists_unknown_network_at_height_0_but_no_header() {
    let network_id: NetworkId = 1.into();
    let height = 0;
    let certificate = Certificate::new_for_test(network_id, height);
    let certificate_id = certificate.hash();

    let (_sender, mut receiver) = tokio::sync::mpsc::channel::<()>(1);

    let certifier_mock = MockCertifier::new();
    let mut pending_store = MockPendingStore::new();

    pending_store
        .expect_get_current_proven_height_for_network()
        .once()
        .with(eq(network_id))
        .returning(|_| Ok(None));
    pending_store
        .expect_remove_pending_certificate()
        .once()
        .returning(|_, _| Ok(()));
    pending_store
        .expect_remove_generated_proof()
        .once()
        .returning(|_| Ok(()));

    let mut state_store = MockStateStore::new();

    let network_id: NetworkId = 1.into();
    state_store
        .expect_get_certificate_header_by_cursor()
        .once()
        .with(eq(network_id), eq(0))
        .return_once(move |_, _| Ok(None));

    let builder = MockOrchestrator::builder()
        .pending_store(pending_store)
        .state_store(state_store)
        .certifier(certifier_mock)
        .build();

    let (_data_sender, mut orchestrator) = create_orchestrator_mock::partial_1(builder);

    let result = orchestrator.handle_certifier_result(Err(crate::Error::ProofAlreadyExists(
        network_id,
        0,
        certificate_id,
    )));

    assert!(result.is_ok());

    sleep(Duration::from_millis(10)).await;
    assert!(receiver.try_recv().is_err());
}

enum ExpectedAction {
    GetCertificateHeader(CertificateStatus),
    RemovePendingCertificate,
    RemoveGeneratedProof,
    UpdateCertificateHeaderStatus(CertificateStatus),
    SetLatestProvenCertificatePerNetwork,
    SpawnNextTask,
}
#[rstest]
// The state doesn't know the network and the height is not 0, we remove the proof.
#[case::unknown_network_height_not_0(1, None, vec![ExpectedAction::RemoveGeneratedProof])]
// The state doesn't know the network and the height is 0. We update the state if needed.
#[case::unknown_network_height_0_pending(0, None, vec![
    ExpectedAction::GetCertificateHeader(CertificateStatus::Pending),
    ExpectedAction::UpdateCertificateHeaderStatus(CertificateStatus::Proven),
    ExpectedAction::SetLatestProvenCertificatePerNetwork,
    ExpectedAction::SpawnNextTask,
])]
#[case::unknown_network_height_0_candidate(0, None, vec![
    ExpectedAction::GetCertificateHeader(CertificateStatus::Candidate),
    ExpectedAction::RemovePendingCertificate,
    ExpectedAction::RemoveGeneratedProof,
    ExpectedAction::SpawnNextTask,
])]
#[case::unknown_network_height_0_in_error(0, None, vec![
    ExpectedAction::GetCertificateHeader(CertificateStatus::InError {
        error: CertificateStatusError::TypeConversionError(agglayer_types::Error::MultipleL1InfoRoot)
    }),
    ExpectedAction::RemovePendingCertificate,
    ExpectedAction::RemoveGeneratedProof,
    ExpectedAction::SpawnNextTask,
])]
#[case::unknown_network_height_0_settled(0, None, vec![
    ExpectedAction::GetCertificateHeader(CertificateStatus::Settled),
    ExpectedAction::RemovePendingCertificate,
    ExpectedAction::RemoveGeneratedProof,
    ExpectedAction::SpawnNextTask,
])]
#[case::unknown_network_height_0_proven(0, None, vec![
    ExpectedAction::GetCertificateHeader(CertificateStatus::Proven),
    ExpectedAction::SpawnNextTask,
])]
#[case::known_network_height_1_pending(1, Some(0), vec![
    ExpectedAction::GetCertificateHeader(CertificateStatus::Pending),
    ExpectedAction::UpdateCertificateHeaderStatus(CertificateStatus::Proven),
    ExpectedAction::SetLatestProvenCertificatePerNetwork,
    ExpectedAction::SpawnNextTask,
])]
#[case::known_network_height_1_pending_but_not_last(1, Some(1), vec![
    ExpectedAction::GetCertificateHeader(CertificateStatus::Pending),
    ExpectedAction::UpdateCertificateHeaderStatus(CertificateStatus::Proven),
    ExpectedAction::SpawnNextTask,
])]
#[case::known_network_height_1_candidate(1, Some(0), vec![
    ExpectedAction::GetCertificateHeader(CertificateStatus::Candidate),
    ExpectedAction::RemovePendingCertificate,
    ExpectedAction::RemoveGeneratedProof,
    ExpectedAction::SpawnNextTask,
])]
#[case::known_network_height_1_settled(1, Some(0), vec![
    ExpectedAction::GetCertificateHeader(CertificateStatus::Settled),
    ExpectedAction::RemovePendingCertificate,
    ExpectedAction::RemoveGeneratedProof,
    ExpectedAction::SpawnNextTask,
])]
#[case::known_network_height_1_proven(1, Some(0), vec![
    ExpectedAction::GetCertificateHeader(CertificateStatus::Proven),
    ExpectedAction::SpawnNextTask,
])]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_millis(100))]
async fn certifier_error_proof_already_exists(
    #[case] height: u64,
    #[case] proven_height: Option<u64>,
    #[case] expected_actions: Vec<ExpectedAction>,
) {
    let network_id: NetworkId = 1.into();
    let certificate = Certificate::new_for_test(network_id, height);
    let certificate_id = certificate.hash();

    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);

    let mut certifier_mock = MockCertifier::new();
    let mut pending_store = MockPendingStore::new();
    let mut state_store = MockStateStore::new();

    let mut seq = Sequence::new();

    let mut trigger_next_certify = false;

    pending_store
        .expect_get_current_proven_height_for_network()
        .once()
        .in_sequence(&mut seq)
        .with(eq(network_id))
        .returning(move |_| Ok(proven_height));

    for action in expected_actions {
        match action {
            ExpectedAction::SpawnNextTask => {
                trigger_next_certify = true;
                let sender = sender.clone();
                certifier_mock
                    .expect_certify()
                    // Don't check the state as it doesn't implement eq
                    .with(always(), eq(network_id), eq(height + 1))
                    .once()
                    .return_once(move |new_state, network, height| {
                        Ok(Box::pin(async move {
                            let result = crate::CertifierOutput {
                                certificate: Certificate::new_for_test(network, height),
                                height,
                                new_state,
                                network,
                            };
                            sender.send(result.clone()).await.unwrap();
                            Ok(result)
                        }))
                    });
            }
            ExpectedAction::GetCertificateHeader(status) => {
                let certificate_header = Ok(Some(CertificateHeader {
                    certificate_id,
                    network_id,
                    height,
                    epoch_number: None,
                    certificate_index: None,
                    metadata: [0; 32].into(),
                    new_local_exit_root: [0; 32].into(),
                    status,
                }));

                state_store
                    .expect_get_certificate_header_by_cursor()
                    .once()
                    .in_sequence(&mut seq)
                    .with(eq(network_id), eq(height))
                    .return_once(move |_, _| certificate_header);
            }
            ExpectedAction::UpdateCertificateHeaderStatus(status) => {
                state_store
                    .expect_update_certificate_header_status()
                    .once()
                    .in_sequence(&mut seq)
                    .with(eq(certificate_id), eq(status))
                    .return_once(move |_, _| Ok(()));
            }
            ExpectedAction::SetLatestProvenCertificatePerNetwork => {
                pending_store
                    .expect_set_latest_proven_certificate_per_network()
                    .once()
                    .in_sequence(&mut seq)
                    .with(eq(network_id), eq(height), eq(certificate_id))
                    .returning(|_, _, _| Ok(()));
            }
            ExpectedAction::RemovePendingCertificate => {
                pending_store
                    .expect_remove_pending_certificate()
                    .once()
                    .in_sequence(&mut seq)
                    .with(eq(network_id), eq(height))
                    .returning(|_, _| Ok(()));
            }
            ExpectedAction::RemoveGeneratedProof => {
                pending_store
                    .expect_remove_generated_proof()
                    .once()
                    .in_sequence(&mut seq)
                    .with(eq(certificate_id))
                    .returning(|_| Ok(()));
            }
        }
    }

    let builder = MockOrchestrator::builder()
        .pending_store(pending_store)
        .state_store(state_store)
        .certifier(certifier_mock)
        .build();

    let (_data_sender, mut orchestrator) = create_orchestrator_mock::partial_1(builder);
    let result = orchestrator.handle_certifier_result(Err(crate::Error::ProofAlreadyExists(
        network_id,
        height,
        certificate_id,
    )));

    assert!(result.is_ok());

    if trigger_next_certify {
        assert!(receiver.recv().await.is_some());
    } else {
        sleep(Duration::from_millis(10)).await;
        assert!(receiver.try_recv().is_err());
    }
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_millis(100))]
async fn certifier_success_with_chain_of_certificates() {
    let network_id: NetworkId = 1.into();
    let height = 0;
    let certificate = Certificate::new_for_test(network_id, height);
    let certificate2 = Certificate::new_for_test(network_id, height + 1);
    let certificate_id = certificate.hash();
    let certificate_id2 = certificate2.hash();

    let (sender, mut receiver) = tokio::sync::mpsc::channel(2);

    let mut certifier_mock = MockCertifier::new();
    let mut pending_store = MockPendingStore::new();
    let mut state_store = MockStateStore::new();
    let mut current_epoch_store = MockPerEpochStore::new();
    let mut seq = Sequence::new();

    certifier_mock
        .expect_certify()
        // Don't check the state as it doesn't implement eq
        .with(always(), eq(network_id), le(height + 1))
        .times(2)
        .returning(move |new_state, network, height| {
            let sender = sender.clone();
            Ok(Box::pin(async move {
                let result = crate::CertifierOutput {
                    certificate: Certificate::new_for_test(network, height),
                    height,
                    new_state,
                    network,
                };
                sender.send(result.clone()).await.unwrap();
                Ok(result)
            }))
        });

    pending_store
        .expect_get_current_proven_height_for_network()
        .once()
        .in_sequence(&mut seq)
        .with(eq(network_id))
        .returning(|_| Ok(None));

    pending_store
        .expect_set_latest_proven_certificate_per_network()
        .once()
        .in_sequence(&mut seq)
        .with(eq(network_id), eq(0), eq(certificate_id))
        .returning(|_, _, _| Ok(()));

    state_store
        .expect_update_certificate_header_status()
        .once()
        .in_sequence(&mut seq)
        .with(eq(certificate_id), eq(&CertificateStatus::Proven))
        .returning(|_, _| Ok(()));

    current_epoch_store
        .expect_add_certificate()
        .once()
        .in_sequence(&mut seq)
        .with(eq(network_id), eq(0))
        .returning(|_, _| Ok((0, 0)));

    pending_store
        .expect_get_current_proven_height_for_network()
        .once()
        .in_sequence(&mut seq)
        .with(eq(network_id))
        .returning(|_| Ok(Some(0)));

    pending_store
        .expect_remove_pending_certificate()
        .never()
        .with(eq(network_id), eq(1))
        .returning(|_, _| Ok(()));

    pending_store
        .expect_set_latest_proven_certificate_per_network()
        .once()
        .in_sequence(&mut seq)
        .with(eq(network_id), eq(1), eq(certificate_id2))
        .returning(|_, _, _| Ok(()));

    state_store
        .expect_update_certificate_header_status()
        .once()
        .in_sequence(&mut seq)
        .with(eq(certificate_id2), eq(&CertificateStatus::Proven))
        .returning(|_, _| Ok(()));

    current_epoch_store
        .expect_add_certificate()
        .once()
        .in_sequence(&mut seq)
        .with(eq(network_id), eq(1))
        .returning(|network_id, height| {
            Err(agglayer_storage::error::Error::CertificateCandidateError(
                agglayer_storage::error::CertificateCandidateError::UnexpectedHeight(
                    network_id, height, 0,
                ),
            ))
        });

    let builder = MockOrchestrator::builder()
        .pending_store(pending_store)
        .current_epoch(current_epoch_store)
        .state_store(state_store)
        .certifier(certifier_mock)
        .build();

    let (_data_sender, mut orchestrator) = create_orchestrator_mock::partial_1(builder);
    orchestrator.spawn_certifier_task(network_id, height);

    let output = receiver.recv().await.expect("output not present");
    orchestrator.handle_certifier_result(Ok(output)).unwrap();

    let output = receiver.recv().await.expect("output not present");
    orchestrator.handle_certifier_result(Ok(output)).unwrap();
}
