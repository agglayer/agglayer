use std::time::Duration;

use agglayer_storage::stores::PendingCertificateWriter;
use agglayer_storage::stores::StateWriter;
use agglayer_types::Certificate;
use agglayer_types::CertificateStatus;
use pessimistic_proof::Signature;
use pessimistic_proof::U256;
use rstest::rstest;
use tokio::time::sleep;

use crate::tests::create_orchestrator;

#[rstest]
#[tokio::test]
#[timeout(Duration::from_millis(100))]
async fn receive_certificate_with_height_zero() {
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
        metadata: Default::default(),
    };

    let certificate_id = certificate.hash();

    orchestrator
        .pending_store
        .insert_pending_certificate(1.into(), 0, &certificate)
        .unwrap();

    let result = orchestrator.receive_certificates(&[(1.into(), 0, certificate_id)]);

    assert!(result.is_ok());
    assert!(matches!(
        orchestrator.proving_cursors.get(&1.into()),
        Some(0)
    ));
    assert!(orchestrator.global_state.contains_key(&1.into()));
    assert!(receiver.recv().await.is_some());
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_millis(100))]
async fn receive_certificate_with_previous_proven() {
    let (_data_sender, mut receiver, mut orchestrator) = create_orchestrator::default();

    let previous = Certificate {
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
        metadata: Default::default(),
    };

    orchestrator
        .state_store
        .insert_certificate_header(&previous, CertificateStatus::Pending)
        .unwrap();

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
        metadata: Default::default(),
    };

    orchestrator
        .pending_store
        .insert_pending_certificate(1.into(), 1, &certificate)
        .unwrap();

    orchestrator.proving_cursors.insert(1.into(), 0);

    let result = orchestrator.receive_certificates(&[(1.into(), 1, [0; 32].into())]);

    assert!(result.is_ok());
    assert!(matches!(
        orchestrator.proving_cursors.get(&1.into()),
        Some(1)
    ));
    assert!(orchestrator.global_state.contains_key(&1.into()));
    assert!(receiver.recv().await.is_some());
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_millis(100))]
async fn receive_certificate_with_previous_pending() {
    let (_data_sender, mut receiver, mut orchestrator) = create_orchestrator::default();

    let previous = Certificate {
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
        metadata: Default::default(),
    };

    orchestrator
        .pending_store
        .insert_pending_certificate(1.into(), 0, &previous)
        .unwrap();

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
        metadata: Default::default(),
    };

    orchestrator
        .pending_store
        .insert_pending_certificate(1.into(), 1, &certificate)
        .unwrap();

    let result = orchestrator.receive_certificates(&[(1.into(), 1, [0; 32].into())]);

    assert!(result.is_ok());
    assert!(!orchestrator.proving_cursors.contains_key(&1.into()));
    sleep(Duration::from_millis(10)).await;

    assert!(receiver.try_recv().is_err());
}
