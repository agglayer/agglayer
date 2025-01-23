use std::{sync::Arc, time::Duration};

use agglayer_storage::tests::mocks::{MockPendingStore, MockStateStore};
use agglayer_types::{
    Certificate, CertificateHeader, CertificateStatus, CertificateStatusError, Digest, NetworkId,
};
use mockall::predicate::eq;
use rstest::rstest;
use tokio::sync::mpsc;

use crate::{
    epoch_packer::MockEpochPacker,
    network_task::NetworkTask,
    tests::{clock, mocks::MockCertifier},
    InitialCheckError,
};

const fn dummy_cert_header(
    network_id: NetworkId,
    height: u64,
    status: CertificateStatus,
) -> CertificateHeader {
    CertificateHeader {
        network_id,
        height,
        epoch_number: None,
        certificate_index: None,
        certificate_id: Digest([0xab; 32]),
        prev_local_exit_root: Digest([0xbc; 32]),
        new_local_exit_root: Digest([0xcd; 32]),
        metadata: Digest([0xef; 32]),
        settlement_tx_hash: None,
        status,
    }
}

#[rstest]
#[case(None)]
#[case(Some(dummy_cert_header(1.into(), 0, CertificateStatus::InError {
        error: CertificateStatusError::TrustedSequencerNotFound(1.into())
    })))]
#[case(Some(dummy_cert_header(1.into(), 0, CertificateStatus::InError {
        error: CertificateStatusError::ProofVerificationFailed(
            agglayer_types::ProofVerificationError::InvalidPublicValues
        )
    })))]
#[tokio::test]
#[timeout(Duration::from_secs(1))]
async fn process_certificate_ok(#[case] existing_cert_header: Option<CertificateHeader>) {
    let mut pending = MockPendingStore::new();
    let mut state = MockStateStore::new();
    let certifier = MockCertifier::new();
    let packer = MockEpochPacker::new();
    let clock_ref = clock();
    let network_id = 1.into();
    let (_sender, certificate_stream) = mpsc::channel(100);

    state
        .expect_get_latest_settled_certificate_per_network()
        .once()
        .with(eq(network_id))
        .returning(|_| Ok(None));

    state
        .expect_get_certificate_header_by_cursor()
        .once()
        .with(eq(network_id), eq(0))
        .returning(move |_network_id, _height| Ok(existing_cert_header.clone()));

    state
        .expect_read_local_network_state()
        .returning(|_| Ok(Default::default()));

    state
        .expect_insert_certificate_header()
        .once()
        .returning(|_cert, _status| Ok(()));

    pending
        .expect_insert_pending_certificate()
        .once()
        .returning(|_net, _ht, _cert| Ok(()));

    state
        .expect_write_local_network_state()
        .returning(|_, _, _| Ok(()));

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

    let certificate = Certificate::new_for_test(network_id, 0);
    let result = task.run_prechecks(&certificate, 0);
    assert!(result.is_ok());
}

#[rstest]
#[case(CertificateStatus::Proven)]
#[case(CertificateStatus::Pending)]
#[case(CertificateStatus::Candidate)]
#[case(CertificateStatus::Settled)]
#[tokio::test]
#[timeout(Duration::from_secs(1))]
async fn replace_certificate_illegally(#[case] cur_cert_status: CertificateStatus) {
    let pending = MockPendingStore::new();
    let mut state = MockStateStore::new();
    let certifier = MockCertifier::new();
    let packer = MockEpochPacker::new();
    let clock_ref = clock();
    let network_id = 1.into();
    let (_sender, certificate_stream) = mpsc::channel(100);

    state
        .expect_get_latest_settled_certificate_per_network()
        .once()
        .with(eq(network_id))
        .returning(|_| Ok(None));

    state
        .expect_get_certificate_header_by_cursor()
        .once()
        .with(eq(network_id), eq(0))
        .returning(move |network_id, height| {
            Ok(Some(dummy_cert_header(
                network_id,
                height,
                cur_cert_status.clone(),
            )))
        });

    state
        .expect_read_local_network_state()
        .returning(|_| Ok(Default::default()));

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

    let certificate = Certificate::new_for_test(network_id, 0);
    let result = task.run_prechecks(&certificate, 0);
    assert!(matches!(
        result.unwrap_err(),
        InitialCheckError::IllegalReplacement { .. }
    ));
}
