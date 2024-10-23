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

    let mut certifier_mock = MockCertifier::new();

    // We expect one call to the certifier for the certificate2 as we forge the
    // certificate one result.
    certifier_mock
        .expect_certify()
        // Don't check the state as it doesn't implement eq
        .with(always(), eq(network_id), eq(1))
        .once()
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

    let mut pending_store = MockPendingStore::new();

    // We expect one call to the pending store to get the current proven height
    // it is used when starting the Orchestrator.
    pending_store
        .expect_get_current_proven_height()
        .returning(|| Ok(vec![]));

    // We expect one call to the pending store to set the latest proven certificate.
    // Having this execute means that the expected certificate ID as been put as the
    // latest proven certificate for this network.
    pending_store
        .expect_set_latest_proven_certificate_per_network()
        .once()
        .with(eq(network_id), eq(0), eq(certificate_id))
        .returning(|_, _, _| Ok(()));

    // This one is expected as we spawn the second certificate.
    // And it validates every steps of the process.
    pending_store
        .expect_set_latest_proven_certificate_per_network()
        .once()
        .with(eq(network_id), eq(1), eq(certificate_id2))
        .returning(|_, _, _| Ok(()));

    // We expect one call to the pending store to remove the first pending
    // certificate. We do not expect the second one to be removed as it is not
    // part of an epoch yet.
    pending_store
        .expect_remove_pending_certificate()
        .once()
        .with(eq(network_id), eq(0))
        .returning(|_, _| Ok(()));

    let mut state_store = MockStateStore::new();

    // We expect one call to the state store to update the certificate header status
    // to proven. This will change as the `add_certificate` will handle the
    // decision of move the certificate to the current epoch or not.
    state_store
        .expect_update_certificate_header_status()
        .with(eq(certificate_id), eq(&CertificateStatus::Proven))
        .once()
        .returning(|_, _| Ok(()));

    // This status change is expected but as mention above, it'll be removed at some
    // point.
    state_store
        .expect_update_certificate_header_status()
        .with(eq(certificate_id2), eq(&CertificateStatus::Proven))
        .once()
        .returning(|_, _| Ok(()));

    let mut current_epoch_store = MockPerEpochStore::new();
    // We expect one call to the current epoch store to add the certificate to the
    // current epoch.
    current_epoch_store
        .expect_add_certificate()
        .once()
        .with(eq(network_id), eq(0))
        .returning(|_, _| Ok((0, 0)));

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
    let mut certifier_mock = MockCertifier::new();
    let mut pending_store = MockPendingStore::new();

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

    // We expect one call to the pending store to get the current proven height
    // it is used when starting the Orchestrator.
    pending_store
        .expect_get_current_proven_height()
        .returning(|| Ok(vec![]));

    pending_store
        .expect_remove_pending_certificate()
        .once()
        .with(eq(network_id), eq(1))
        .returning(|_, _| Ok(()));

    pending_store
        .expect_remove_generated_proof()
        .once()
        .with(eq(certificate_id))
        .returning(|_| Ok(()));

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
    assert!(!orchestrator.proving_cursors.contains_key(&network_id));
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
    assert!(!orchestrator.proving_cursors.contains_key(&network_id));
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
        .expect_get_current_proven_height()
        .returning(|| Ok(vec![]));

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

#[rstest]
#[case::unknown_network_height_0_no_header(0, CertificateStatus::Candidate, false)]
#[case::unknown_network_height_0_candidate(0, CertificateStatus::Candidate, false)]
#[case::unknown_network_height_0_pending(0, CertificateStatus::Pending, false)]
#[case::unknown_network_height_0_proven(0, CertificateStatus::Proven, false)]
#[case::unknown_network_height_0_settled(0, CertificateStatus::Settled, false)]
#[case::unknown_network_height_0_in_error(0, CertificateStatus::InError {
    error: CertificateStatusError::TypeConversionError(
        agglayer_types::Error::MultipleL1InfoRoot,
)}, false)]
#[case::known_network_height_not_0_candidate(1, CertificateStatus::Candidate, true)]
#[case::known_network_height_not_0_pending(1, CertificateStatus::Pending, true)]
#[case::known_network_height_not_0_proven(1, CertificateStatus::Proven, true)]
#[case::known_network_height_not_0_settled(1, CertificateStatus::Settled, true)]
#[case::known_network_height_not_0_in_error(1, CertificateStatus::InError {
    error: CertificateStatusError::TypeConversionError(
        agglayer_types::Error::MultipleL1InfoRoot,
)}, true)]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_millis(100))]
async fn certifier_error_proof_already_exists(
    #[case] height: u64,
    #[case] status: CertificateStatus,
    #[case] known_network: bool,
    #[values(true, false)] return_header: bool,
) {
    let network_id: NetworkId = 1.into();
    let certificate = Certificate::new_for_test(network_id, height);
    let certificate_id = certificate.hash();

    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);

    let mut certifier_mock = MockCertifier::new();
    let mut pending_store = MockPendingStore::new();
    let mut state_store = MockStateStore::new();

    pending_store
        .expect_get_current_proven_height()
        .returning(|| Ok(vec![]));

    if return_header {
        // We expect one call to the certifier for the certificate at height 2 as we
        // forge the certificate one result.
        certifier_mock
            .expect_certify()
            // Don't check the state as it doesn't implement eq
            .with(always(), eq(network_id), eq(height + 1))
            .once()
            .return_once(|new_state, network, height| {
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

        match status {
            CertificateStatus::Candidate | CertificateStatus::Settled => {
                pending_store
                    .expect_remove_pending_certificate()
                    .once()
                    .returning(|_, _| Ok(()));

                pending_store
                    .expect_remove_generated_proof()
                    .once()
                    .returning(|_| Ok(()));
            }
            CertificateStatus::InError { .. } | CertificateStatus::Pending => {
                state_store
                    .expect_update_certificate_header_status()
                    .once()
                    .with(eq(certificate_id), eq(CertificateStatus::Proven))
                    .returning(|_, _| Ok(()));
            }
            _ => {}
        }
    } else {
        pending_store
            .expect_remove_pending_certificate()
            .once()
            .returning(|_, _| Ok(()));

        pending_store
            .expect_remove_generated_proof()
            .once()
            .returning(|_| Ok(()));
    }

    let certificate_header = if return_header {
        Ok(Some(CertificateHeader {
            certificate_id,
            network_id,
            height,
            epoch_number: None,
            certificate_index: None,
            new_local_exit_root: [0; 32].into(),
            status,
        }))
    } else {
        Ok(None)
    };

    state_store
        .expect_get_certificate_header_by_cursor()
        .once()
        .with(eq(network_id), eq(height))
        .return_once(move |_, _| certificate_header);

    let builder = MockOrchestrator::builder()
        .pending_store(pending_store)
        .state_store(state_store)
        .certifier(certifier_mock)
        .build();

    let (_data_sender, mut orchestrator) = create_orchestrator_mock::partial_1(builder);

    if known_network {
        *orchestrator.proving_cursors.entry(network_id).or_default() = height;
    }
    let result = orchestrator.handle_certifier_result(Err(crate::Error::ProofAlreadyExists(
        network_id,
        height,
        certificate_id,
    )));

    assert!(result.is_ok());

    if return_header {
        assert!(receiver.recv().await.is_some());
    } else {
        sleep(Duration::from_millis(10)).await;
        assert!(receiver.try_recv().is_err());
    }
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_millis(100))]
async fn certifier_error_proof_already_exists_with_previous_pending() {
    let network_id: NetworkId = 1.into();
    let height = 1;
    let certificate = Certificate::new_for_test(network_id, height);
    let certificate_id = certificate.hash();

    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);

    let mut certifier_mock = MockCertifier::new();
    let mut pending_store = MockPendingStore::new();
    let mut state_store = MockStateStore::new();

    pending_store
        .expect_get_current_proven_height()
        .returning(|| Ok(vec![]));

    // We expect one call to the certifier for the certificate at height 2 as we
    // forge the certificate one result.
    certifier_mock
        .expect_certify()
        // Don't check the state as it doesn't implement eq
        .with(always(), eq(network_id), eq(height + 1))
        .never()
        .return_once(|new_state, network, height| {
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
        .expect_remove_generated_proof()
        .once()
        .returning(|_| Ok(()));

    state_store
        .expect_update_certificate_header_status()
        .never()
        .returning(|_, _| Ok(()));
    state_store
        .expect_get_certificate_header_by_cursor()
        .never()
        .return_once(move |_, _| Ok(None));

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

    sleep(Duration::from_millis(10)).await;
    assert!(receiver.try_recv().is_err());
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_millis(100))]
async fn certifier_error_proof_already_exists_but_wrong_height() {
    let network_id: NetworkId = 1.into();
    let height = 4;
    let certificate = Certificate::new_for_test(network_id, height);
    let certificate_id = certificate.hash();

    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);

    let mut certifier_mock = MockCertifier::new();
    let mut pending_store = MockPendingStore::new();
    let mut state_store = MockStateStore::new();

    pending_store
        .expect_get_current_proven_height()
        .returning(|| Ok(vec![]));

    // We expect one call to the certifier for the certificate at height 2 as we
    // forge the certificate one result.
    certifier_mock
        .expect_certify()
        // Don't check the state as it doesn't implement eq
        .with(always(), eq(network_id), eq(height + 1))
        .never()
        .return_once(|new_state, network, height| {
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
        .expect_remove_generated_proof()
        .once()
        .returning(|_| Ok(()));

    pending_store
        .expect_remove_pending_certificate()
        .once()
        .returning(|_, _| Ok(()));

    state_store
        .expect_update_certificate_header_status()
        .never()
        .returning(|_, _| Ok(()));
    state_store
        .expect_get_certificate_header_by_cursor()
        .never()
        .return_once(move |_, _| Ok(None));

    let builder = MockOrchestrator::builder()
        .pending_store(pending_store)
        .state_store(state_store)
        .certifier(certifier_mock)
        .build();

    let (_data_sender, mut orchestrator) = create_orchestrator_mock::partial_1(builder);

    orchestrator.proving_cursors.insert(1.into(), 1);
    orchestrator
        .handle_certifier_result(Err(crate::Error::ProofAlreadyExists(
            network_id,
            height,
            certificate_id,
        )))
        .unwrap();

    sleep(Duration::from_millis(10)).await;

    assert!(receiver.try_recv().is_err());
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
        .expect_get_current_proven_height()
        .returning(|| Ok(vec![]));

    pending_store
        .expect_set_latest_proven_certificate_per_network()
        .once()
        .with(eq(network_id), eq(0), eq(certificate_id))
        .returning(|_, _, _| Ok(()));

    pending_store
        .expect_set_latest_proven_certificate_per_network()
        .once()
        .with(eq(network_id), eq(1), eq(certificate_id2))
        .returning(|_, _, _| Ok(()));

    pending_store
        .expect_remove_pending_certificate()
        .once()
        .with(eq(network_id), eq(0))
        .returning(|_, _| Ok(()));

    pending_store
        .expect_remove_pending_certificate()
        .never()
        .with(eq(network_id), eq(1))
        .returning(|_, _| Ok(()));

    state_store
        .expect_update_certificate_header_status()
        .once()
        .with(eq(certificate_id), eq(&CertificateStatus::Proven))
        .returning(|_, _| Ok(()));

    state_store
        .expect_update_certificate_header_status()
        .once()
        .with(eq(certificate_id2), eq(&CertificateStatus::Proven))
        .returning(|_, _| Ok(()));

    let mut current_epoch_store = MockPerEpochStore::new();

    current_epoch_store
        .expect_add_certificate()
        .once()
        .with(eq(network_id), eq(0))
        .returning(|_, _| Ok((0, 0)));

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
