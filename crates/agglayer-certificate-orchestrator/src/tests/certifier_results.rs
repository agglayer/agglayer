use std::time::Duration;

use agglayer_storage::stores::PendingCertificateReader;
use agglayer_storage::stores::PendingCertificateWriter;
use agglayer_storage::stores::StateReader;
use agglayer_storage::stores::StateWriter;
use agglayer_types::Certificate;
use agglayer_types::CertificateHeader;
use agglayer_types::CertificateStatus;
use agglayer_types::LocalNetworkStateData;
use agglayer_types::Proof;
use pessimistic_proof::Signature;
use pessimistic_proof::U256;
use rstest::rstest;
use tokio::time::sleep;

use crate::tests::create_orchestrator;

#[rstest]
#[tokio::test]
#[timeout(Duration::from_millis(100))]
async fn certifier_results_for_unknown_network_with_height_zero() {
    let (_data_sender, mut receiver, mut orchestrator) = create_orchestrator::default();

    let certificate = Certificate {
        network_id: 1.into(),
        height: 0,
        prev_local_exit_root: [0; 32],
        new_local_exit_root: [0; 32],
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        signature: Signature {
            r: U256::ZERO,
            s: U256::ZERO,
            odd_y_parity: false,
        },
    };
    let certificate2 = Certificate {
        network_id: 1.into(),
        height: 1,
        prev_local_exit_root: [0; 32],
        new_local_exit_root: [0; 32],
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        signature: Signature {
            r: U256::ZERO,
            s: U256::ZERO,
            odd_y_parity: false,
        },
    };
    let certificate_id = certificate.hash();
    let proof = Proof::new_for_test();

    orchestrator
        .pending_store
        .insert_pending_certificate(1.into(), 1, &certificate2)
        .unwrap();

    orchestrator
        .pending_store
        .insert_generated_proof(&certificate_id, &proof)
        .unwrap();

    orchestrator
        .state_store
        .insert_certificate_header(&certificate, CertificateStatus::Proven)
        .unwrap();

    orchestrator
        .state_store
        .insert_certificate_header(&certificate2, CertificateStatus::Pending)
        .unwrap();

    let header = orchestrator
        .state_store
        .get_certificate_header_by_cursor(1.into(), 1)
        .unwrap();

    assert!(matches!(
        header,
        Some(CertificateHeader {
            status: CertificateStatus::Pending,
            ..
        })
    ));

    assert!(orchestrator
        .state_store
        .get_certificate_header_by_cursor(1.into(), 2)
        .unwrap()
        .is_none());

    let state = LocalNetworkStateData::default();
    let result = orchestrator.handle_certifier_result(Ok(crate::CertifierOutput {
        certificate,
        height: 0,
        new_state: state,
        network: 1.into(),
    }));

    assert!(result.is_ok());
    assert!(matches!(
        orchestrator
            .state_store
            .get_certificate_header_by_cursor(1.into(), 1)
            .unwrap(),
        Some(CertificateHeader {
            status: CertificateStatus::Pending,
            ..
        })
    ));

    let result = receiver.recv().await.expect("output not present");

    let result = orchestrator.handle_certifier_result(Ok(result));

    assert!(result.is_ok());

    let header = orchestrator
        .state_store
        .get_certificate_header_by_cursor(1.into(), 1);

    assert!(
        matches!(
            header,
            Ok(Some(CertificateHeader {
                network_id,
                height: 1,
                epoch_number: None,
                certificate_index: None,
                ..
            })) if network_id == 1.into()

        ),
        "Certificate header not match {:?}",
        header
    );

    assert!(orchestrator
        .pending_store
        .get_certificate(1.into(), 1)
        .unwrap()
        .is_none());

    assert!(orchestrator.global_state.contains_key(&1.into()));
}

#[tokio::test]
async fn certifier_results_for_unknown_network_with_height_not_zero() {
    let (_data_sender, mut receiver, mut orchestrator) = create_orchestrator::default();

    let certificate = Certificate {
        network_id: 1.into(),
        height: 1,
        prev_local_exit_root: [0; 32],
        new_local_exit_root: [0; 32],
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        signature: Signature {
            r: U256::ZERO,
            s: U256::ZERO,
            odd_y_parity: false,
        },
    };

    let state = LocalNetworkStateData::default();
    let result = orchestrator.handle_certifier_result(Ok(crate::CertifierOutput {
        certificate,
        height: 1,
        new_state: state,
        network: 1.into(),
    }));

    assert!(result.is_err());
    assert!(!orchestrator.cursors.contains_key(&1.into()));
    assert!(!orchestrator.global_state.contains_key(&1.into()));
    sleep(Duration::from_millis(10)).await;

    assert!(receiver.try_recv().is_err());
}

#[test_log::test(tokio::test)]
async fn certifier_error_certificate_does_not_exists() {
    let (_data_sender, mut receiver, mut orchestrator) = create_orchestrator::default();

    let result =
        orchestrator.handle_certifier_result(Err(crate::Error::CertificateNotFound(1.into(), 0)));

    assert!(result.is_ok());
    assert!(!orchestrator.cursors.contains_key(&1.into()));
    assert!(!orchestrator.global_state.contains_key(&1.into()));
    sleep(Duration::from_millis(10)).await;

    assert!(receiver.try_recv().is_err());
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(Duration::from_millis(100))]
async fn certifier_error_proof_already_exists_with_height_zero() {
    let (_data_sender, mut receiver, mut orchestrator) = create_orchestrator::default();
    let certificate = Certificate {
        network_id: 1.into(),
        height: 0,
        prev_local_exit_root: [0; 32],
        new_local_exit_root: [0; 32],
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        signature: Signature {
            r: U256::ZERO,
            s: U256::ZERO,
            odd_y_parity: false,
        },
    };
    let certificate2 = Certificate {
        network_id: 1.into(),
        height: 1,
        prev_local_exit_root: [0; 32],
        new_local_exit_root: [0; 32],
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        signature: Signature {
            r: U256::ZERO,
            s: U256::ZERO,
            odd_y_parity: false,
        },
    };
    let certificate_id = certificate.hash();
    let proof = Proof::new_for_test();

    orchestrator
        .pending_store
        .insert_pending_certificate(1.into(), 1, &certificate2)
        .unwrap();

    orchestrator
        .pending_store
        .insert_generated_proof(&certificate_id, &proof)
        .unwrap();

    orchestrator
        .state_store
        .insert_certificate_header(&certificate, CertificateStatus::Proven)
        .unwrap();

    orchestrator
        .state_store
        .insert_certificate_header(&certificate2, CertificateStatus::Pending)
        .unwrap();

    assert!(matches!(
        orchestrator
            .state_store
            .get_certificate_header_by_cursor(1.into(), 1)
            .unwrap(),
        Some(CertificateHeader {
            status: CertificateStatus::Pending,
            ..
        })
    ));

    let result = orchestrator.handle_certifier_result(Err(crate::Error::ProofAlreadyExists(
        1.into(),
        0,
        certificate_id,
    )));

    assert!(result.is_ok());
    assert!(matches!(orchestrator.cursors.get(&1.into()), Some(0)));
    assert!(orchestrator.global_state.contains_key(&1.into()));
    assert!(receiver.recv().await.is_some());
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_millis(100))]
async fn certifier_error_proof_already_exists_with_height_zero_still_pending() {
    let (_data_sender, mut receiver, mut orchestrator) = create_orchestrator::default();
    let certificate = Certificate {
        network_id: 1.into(),
        height: 0,
        prev_local_exit_root: [0; 32],
        new_local_exit_root: [0; 32],
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        signature: Signature {
            r: U256::ZERO,
            s: U256::ZERO,
            odd_y_parity: false,
        },
    };
    let certificate2 = Certificate {
        network_id: 1.into(),
        height: 1,
        prev_local_exit_root: [0; 32],
        new_local_exit_root: [0; 32],
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        signature: Signature {
            r: U256::ZERO,
            s: U256::ZERO,
            odd_y_parity: false,
        },
    };
    let certificate_id = certificate.hash();
    let proof = Proof::new_for_test();

    orchestrator
        .pending_store
        .insert_pending_certificate(1.into(), 0, &certificate)
        .unwrap();

    orchestrator
        .pending_store
        .insert_pending_certificate(1.into(), 1, &certificate2)
        .unwrap();

    orchestrator
        .state_store
        .insert_certificate_header(&certificate, CertificateStatus::Pending)
        .unwrap();

    orchestrator
        .pending_store
        .insert_generated_proof(&certificate_id, &proof)
        .unwrap();

    assert!(orchestrator
        .state_store
        .get_certificate_header_by_cursor(1.into(), 0)
        .unwrap()
        .is_some());

    assert!(orchestrator
        .state_store
        .get_certificate_header_by_cursor(1.into(), 1)
        .unwrap()
        .is_none());
    let result = orchestrator.handle_certifier_result(Err(crate::Error::ProofAlreadyExists(
        1.into(),
        0,
        certificate_id,
    )));

    assert!(result.is_ok());

    assert!(orchestrator
        .pending_store
        .get_certificate(1.into(), 0)
        .unwrap()
        .is_none());

    assert!(matches!(orchestrator.cursors.get(&1.into()), Some(0)));
    assert!(orchestrator.global_state.contains_key(&1.into()));
    assert!(receiver.recv().await.is_some());
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_millis(100))]
async fn certifier_error_proof_already_exists_with_height_zero_no_header() {
    let (_data_sender, mut receiver, mut orchestrator) = create_orchestrator::default();
    let certificate = Certificate {
        network_id: 1.into(),
        height: 0,
        prev_local_exit_root: [0; 32],
        new_local_exit_root: [0; 32],
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        signature: Signature {
            r: U256::ZERO,
            s: U256::ZERO,
            odd_y_parity: false,
        },
    };
    let certificate2 = Certificate {
        network_id: 1.into(),
        height: 1,
        prev_local_exit_root: [0; 32],
        new_local_exit_root: [0; 32],
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        signature: Signature {
            r: U256::ZERO,
            s: U256::ZERO,
            odd_y_parity: false,
        },
    };
    let certificate_id = certificate.hash();
    let proof = Proof::new_for_test();

    orchestrator
        .pending_store
        .insert_pending_certificate(1.into(), 0, &certificate)
        .unwrap();

    orchestrator
        .pending_store
        .insert_pending_certificate(1.into(), 1, &certificate2)
        .unwrap();

    orchestrator
        .pending_store
        .insert_generated_proof(&certificate_id, &proof)
        .unwrap();

    orchestrator
        .state_store
        .insert_certificate_header(&certificate, CertificateStatus::Pending)
        .unwrap();
    orchestrator
        .state_store
        .insert_certificate_header(&certificate2, CertificateStatus::Pending)
        .unwrap();

    assert!(orchestrator
        .state_store
        .get_certificate_header_by_cursor(1.into(), 0)
        .unwrap()
        .is_some());

    assert!(orchestrator
        .state_store
        .get_certificate_header_by_cursor(1.into(), 1)
        .unwrap()
        .is_some());
    let result = orchestrator.handle_certifier_result(Err(crate::Error::ProofAlreadyExists(
        1.into(),
        0,
        certificate_id,
    )));

    assert!(result.is_ok());

    assert!(orchestrator
        .state_store
        .get_certificate_header_by_cursor(1.into(), 0)
        .unwrap()
        .is_some());

    assert!(matches!(orchestrator.cursors.get(&1.into()), Some(0)));
    assert!(orchestrator.global_state.contains_key(&1.into()));
    assert!(receiver.recv().await.is_some());
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_millis(100))]
async fn certifier_error_proof_already_exists_with_height_zero_no_header_no_pending() {
    let (_data_sender, mut receiver, mut orchestrator) = create_orchestrator::default();
    let certificate = Certificate {
        network_id: 1.into(),
        height: 0,
        prev_local_exit_root: [0; 32],
        new_local_exit_root: [0; 32],
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        signature: Signature {
            r: U256::ZERO,
            s: U256::ZERO,
            odd_y_parity: false,
        },
    };
    let certificate2 = Certificate {
        network_id: 1.into(),
        height: 1,
        prev_local_exit_root: [0; 32],
        new_local_exit_root: [0; 32],
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        signature: Signature {
            r: U256::ZERO,
            s: U256::ZERO,
            odd_y_parity: false,
        },
    };
    let certificate_id = certificate.hash();
    let proof = Proof::new_for_test();

    orchestrator
        .pending_store
        .insert_pending_certificate(1.into(), 1, &certificate2)
        .unwrap();

    orchestrator
        .pending_store
        .insert_generated_proof(&certificate_id, &proof)
        .unwrap();

    assert!(orchestrator
        .state_store
        .get_certificate_header_by_cursor(1.into(), 0)
        .unwrap()
        .is_none());

    assert!(orchestrator
        .state_store
        .get_certificate_header_by_cursor(1.into(), 1)
        .unwrap()
        .is_none());

    let result = orchestrator.handle_certifier_result(Err(crate::Error::ProofAlreadyExists(
        1.into(),
        0,
        certificate_id,
    )));

    assert!(result.is_err());
    assert!(!orchestrator.cursors.contains_key(&1.into()));
    assert!(!orchestrator.global_state.contains_key(&1.into()));

    sleep(Duration::from_millis(10)).await;

    assert!(receiver.try_recv().is_err());
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_millis(100))]
async fn certifier_error_proof_already_exists_with_previous_proven() {
    let (_data_sender, mut receiver, mut orchestrator) = create_orchestrator::default();

    let certificate = Certificate {
        network_id: 1.into(),
        height: 2,
        prev_local_exit_root: [0; 32],
        new_local_exit_root: [0; 32],
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        signature: Signature {
            r: U256::ZERO,
            s: U256::ZERO,
            odd_y_parity: false,
        },
    };

    let certificate2 = Certificate {
        network_id: 1.into(),
        height: 3,
        prev_local_exit_root: [0; 32],
        new_local_exit_root: [0; 32],
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        signature: Signature {
            r: U256::ZERO,
            s: U256::ZERO,
            odd_y_parity: false,
        },
    };

    let certificate_id = certificate.hash();
    let proof = Proof::new_for_test();

    orchestrator
        .pending_store
        .insert_pending_certificate(1.into(), 2, &certificate)
        .unwrap();

    orchestrator
        .pending_store
        .insert_pending_certificate(1.into(), 3, &certificate2)
        .unwrap();
    orchestrator
        .pending_store
        .insert_generated_proof(&certificate_id, &proof)
        .unwrap();

    orchestrator.cursors.insert(1.into(), 1);
    let result = orchestrator.handle_certifier_result(Err(crate::Error::ProofAlreadyExists(
        1.into(),
        2,
        certificate_id,
    )));

    assert!(result.is_ok());
    assert!(matches!(orchestrator.cursors.get(&1.into()), Some(&2)));
    assert!(orchestrator.global_state.contains_key(&1.into()));
    assert!(receiver.recv().await.is_some());
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_millis(100))]
async fn certifier_error_proof_already_exists_with_previous_proven_no_header() {
    let (_data_sender, mut receiver, mut orchestrator) = create_orchestrator::default();

    let certificate = Certificate {
        network_id: 1.into(),
        height: 2,
        prev_local_exit_root: [0; 32],
        new_local_exit_root: [0; 32],
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        signature: Signature {
            r: U256::ZERO,
            s: U256::ZERO,
            odd_y_parity: false,
        },
    };

    let certificate2 = Certificate {
        network_id: 1.into(),
        height: 3,
        prev_local_exit_root: [0; 32],
        new_local_exit_root: [0; 32],
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        signature: Signature {
            r: U256::ZERO,
            s: U256::ZERO,
            odd_y_parity: false,
        },
    };

    let certificate_id = certificate.hash();
    let proof = Proof::new_for_test();

    orchestrator
        .pending_store
        .insert_pending_certificate(1.into(), 2, &certificate)
        .unwrap();

    orchestrator
        .pending_store
        .insert_pending_certificate(1.into(), 3, &certificate2)
        .unwrap();

    orchestrator
        .state_store
        .insert_certificate_header(&certificate, CertificateStatus::Proven)
        .unwrap();

    orchestrator
        .state_store
        .insert_certificate_header(&certificate2, CertificateStatus::Pending)
        .unwrap();

    orchestrator
        .pending_store
        .insert_generated_proof(&certificate_id, &proof)
        .unwrap();

    assert!(matches!(
        orchestrator
            .state_store
            .get_certificate_header_by_cursor(1.into(), 2)
            .unwrap(),
        Some(CertificateHeader {
            height: 2,
            epoch_number: None,
            certificate_index: None,
            status: CertificateStatus::Proven,
            ..
        })
    ));

    assert!(matches!(
        orchestrator
            .state_store
            .get_certificate_header_by_cursor(1.into(), 3)
            .unwrap(),
        Some(CertificateHeader {
            height: 3,
            epoch_number: None,
            certificate_index: None,
            status: CertificateStatus::Pending,
            ..
        })
    ));

    orchestrator.cursors.insert(1.into(), 1);
    let result = orchestrator.handle_certifier_result(Err(crate::Error::ProofAlreadyExists(
        1.into(),
        2,
        certificate_id,
    )));

    assert!(result.is_ok());

    assert!(matches!(
        orchestrator
            .state_store
            .get_certificate_header_by_cursor(1.into(), 2)
            .unwrap(),
        Some(CertificateHeader {
            status: CertificateStatus::Proven,
            ..
        })
    ));

    assert!(matches!(orchestrator.cursors.get(&1.into()), Some(&2)));
    assert!(orchestrator.global_state.contains_key(&1.into()));
    assert!(receiver.recv().await.is_some());
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_millis(100))]
async fn certifier_error_proof_already_exists_with_previous_pending() {
    let (_data_sender, mut receiver, mut orchestrator) = create_orchestrator::default();

    let previous = Certificate {
        network_id: 1.into(),
        height: 3,
        prev_local_exit_root: [0; 32],
        new_local_exit_root: [0; 32],
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        signature: Signature {
            r: U256::ZERO,
            s: U256::ZERO,
            odd_y_parity: false,
        },
    };
    let certificate = Certificate {
        network_id: 1.into(),
        height: 4,
        prev_local_exit_root: [0; 32],
        new_local_exit_root: [0; 32],
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        signature: Signature {
            r: U256::ZERO,
            s: U256::ZERO,
            odd_y_parity: false,
        },
    };

    let certificate_id = certificate.hash();
    let proof = Proof::new_for_test();

    orchestrator
        .pending_store
        .insert_pending_certificate(1.into(), 3, &previous)
        .unwrap();

    orchestrator
        .pending_store
        .insert_pending_certificate(1.into(), 4, &certificate)
        .unwrap();

    orchestrator
        .pending_store
        .insert_generated_proof(&certificate_id, &proof)
        .unwrap();

    assert!(orchestrator
        .state_store
        .get_certificate_header_by_cursor(1.into(), 2)
        .unwrap()
        .is_none());

    assert!(orchestrator
        .state_store
        .get_certificate_header_by_cursor(1.into(), 3)
        .unwrap()
        .is_none());

    orchestrator.cursors.insert(1.into(), 2);
    let result = orchestrator.handle_certifier_result(Err(crate::Error::ProofAlreadyExists(
        1.into(),
        4,
        certificate_id,
    )));

    assert!(result.is_err());
    assert!(matches!(orchestrator.cursors.get(&1.into()), Some(2)));

    assert!(orchestrator
        .state_store
        .get_certificate_header_by_cursor(1.into(), 2)
        .unwrap()
        .is_none());

    assert!(orchestrator
        .state_store
        .get_certificate_header_by_cursor(1.into(), 3)
        .unwrap()
        .is_none());

    assert!(orchestrator
        .pending_store
        .get_proof(certificate_id)
        .unwrap()
        .is_some());

    sleep(Duration::from_millis(10)).await;

    assert!(receiver.try_recv().is_err());
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_millis(100))]
async fn certifier_error_proof_already_exists_but_wrong_height() {
    let (_data_sender, mut receiver, mut orchestrator) = create_orchestrator::default();
    let certificate_id = [0; 32].into();

    orchestrator.cursors.insert(1.into(), 1);
    let result = orchestrator.handle_certifier_result(Err(crate::Error::ProofAlreadyExists(
        1.into(),
        4,
        certificate_id,
    )));

    assert!(result.is_err());
    assert!(matches!(orchestrator.cursors.get(&1.into()), Some(1)));

    sleep(Duration::from_millis(10)).await;

    assert!(receiver.try_recv().is_err());
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_millis(100))]
async fn certifier_success_with_chain_of_certificates() {
    let (_data_sender, mut receiver, mut orchestrator) = create_orchestrator::default();

    let certificate = Certificate {
        network_id: 1.into(),
        height: 0,
        prev_local_exit_root: [0; 32],
        new_local_exit_root: [0; 32],
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        signature: Signature {
            r: U256::ZERO,
            s: U256::ZERO,
            odd_y_parity: false,
        },
    };
    let certificate2 = Certificate {
        network_id: 1.into(),
        height: 1,
        prev_local_exit_root: [0; 32],
        new_local_exit_root: [0; 32],
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        signature: Signature {
            r: U256::ZERO,
            s: U256::ZERO,
            odd_y_parity: false,
        },
    };
    let certificate_id = certificate.hash();
    let certificate_id2 = certificate.hash();

    orchestrator
        .pending_store
        .insert_pending_certificate(1.into(), 0, &certificate)
        .unwrap();

    orchestrator
        .pending_store
        .insert_pending_certificate(1.into(), 1, &certificate2)
        .unwrap();

    _ = orchestrator
        .state_store
        .insert_certificate_header(&certificate, CertificateStatus::Pending);
    _ = orchestrator
        .state_store
        .insert_certificate_header(&certificate2, CertificateStatus::Pending);

    let header = orchestrator
        .state_store
        .get_certificate_header_by_cursor(1.into(), 0)
        .unwrap();

    assert!(matches!(
        header,
        Some(CertificateHeader {
            status: CertificateStatus::Pending,
            ..
        })
    ));

    assert!(matches!(
        orchestrator
            .state_store
            .get_certificate_header_by_cursor(1.into(), 1)
            .unwrap(),
        Some(CertificateHeader {
            status: CertificateStatus::Pending,
            ..
        })
    ));

    orchestrator.spawn_certifier_task(1.into(), 0);
    let output = receiver.recv().await.expect("output not present");

    let result = orchestrator.handle_certifier_result(Ok(output));

    assert!(result.is_ok());
    assert!(matches!(
        orchestrator
            .state_store
            .get_certificate_header_by_cursor(1.into(), 0)
            .unwrap(),
        Some(CertificateHeader {
            status: CertificateStatus::Proven,
            ..
        })
    ));

    assert!(orchestrator
        .pending_store
        .get_certificate(1.into(), 0)
        .unwrap()
        .is_none());

    assert!(orchestrator
        .pending_store
        .get_proof(certificate_id)
        .unwrap()
        .is_some());

    let output = receiver.recv().await.expect("output not present");

    let result = orchestrator.handle_certifier_result(Ok(output));
    assert!(result.is_ok());

    assert!(matches!(
        orchestrator
            .state_store
            .get_certificate_header_by_cursor(1.into(), 1),
        Ok(Some(CertificateHeader {
            network_id,
            height: 1,
            epoch_number: None,
            certificate_index: None,
            ..
        })) if network_id == 1.into()
    ));

    assert!(orchestrator
        .pending_store
        .get_certificate(1.into(), 1)
        .unwrap()
        .is_none());

    assert!(orchestrator
        .pending_store
        .get_proof(certificate_id2)
        .unwrap()
        .is_some());

    assert!(orchestrator.cursors.get(&1.into()) == Some(&1));
    assert!(orchestrator.global_state.contains_key(&1.into()));
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_millis(100))]
async fn certifier_success_with_not_chained_certificates() {
    let (_data_sender, mut receiver, mut orchestrator) = create_orchestrator::default();

    let certificate = Certificate {
        network_id: 1.into(),
        height: 2,
        prev_local_exit_root: [0; 32],
        new_local_exit_root: [0; 32],
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        signature: Signature {
            r: U256::ZERO,
            s: U256::ZERO,
            odd_y_parity: false,
        },
    };
    let certificate_id = certificate.hash();

    orchestrator.cursors.insert(1.into(), 0);
    orchestrator
        .pending_store
        .insert_pending_certificate(1.into(), 2, &certificate)
        .unwrap();

    assert!(orchestrator
        .state_store
        .get_certificate_header_by_cursor(1.into(), 2)
        .unwrap()
        .is_none());

    orchestrator.spawn_certifier_task(1.into(), 2);
    let output = receiver.recv().await.expect("output not present");

    let result = orchestrator.handle_certifier_result(Ok(output));

    assert!(result.is_err());
    assert!(orchestrator.cursors.get(&1.into()) == Some(&0));

    let x = orchestrator
        .state_store
        .get_certificate_header_by_cursor(1.into(), 2)
        .unwrap();

    assert!(x.is_none());

    assert!(orchestrator
        .pending_store
        .get_certificate(1.into(), 2)
        .unwrap()
        .is_some());

    assert!(orchestrator
        .pending_store
        .get_proof(certificate_id)
        .unwrap()
        .is_some());
}
