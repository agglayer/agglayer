use std::time::Duration;

use agglayer_storage::tests::TempDBDir;
use agglayer_types::{CertificateHeader, CertificateId, CertificateStatus};
use fail::FailScenario;
use jsonrpsee::{core::client::ClientT, rpc_params};
use pessimistic_proof_test_suite::forest::Forest;
use rstest::rstest;

mod common;

use common::agglayer_setup::{get_signer, setup_agglayer};

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
async fn successfully_push_certificate() {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    // L1 is a RAII guard
    let (_handle, _l1, client) = setup_agglayer(&tmp_dir.path).await;
    let signer = get_signer(0);

    let state = Forest::default().with_signer(signer);

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let mut status;
    loop {
        let response: CertificateHeader = client
            .request("interop_getCertificateHeader", rpc_params![certificate_id])
            .await
            .unwrap();

        status = response.status;

        match status {
            CertificateStatus::InError { .. } | CertificateStatus::Settled => {
                break;
            }
            _ => {
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
    }

    assert_eq!(status, CertificateStatus::Settled);

    scenario.teardown();
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
async fn failure_on_settlement_transaction_failed_status_0() {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::status_0",
        "return",
    )
    .expect("Failed to configure failpoint");

    // L1 is a RAII guard
    let (_handle, _l1, client) = setup_agglayer(&tmp_dir.path).await;
    let signer = get_signer(0);

    let state = Forest::default().with_signer(signer);

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let mut status;

    loop {
        let response: CertificateHeader = client
            .request("interop_getCertificateHeader", rpc_params![certificate_id])
            .await
            .unwrap();

        status = response.status;
        match status {
            CertificateStatus::InError { .. } | CertificateStatus::Settled => {
                break;
            }
            _ => {
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
    }

    assert!(matches!(status, CertificateStatus::InError { .. }));

    scenario.teardown();
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
async fn failure_on_settlement_transaction_failed_no_receipt() {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::no_receipt",
        "return",
    )
    .expect("Failed to configure failpoint");

    // L1 is a RAII guard
    let (_handle, _l1, client) = setup_agglayer(&tmp_dir.path).await;
    let signer = get_signer(0);

    let state = Forest::default().with_signer(signer);

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let mut status;

    loop {
        let response: CertificateHeader = client
            .request("interop_getCertificateHeader", rpc_params![certificate_id])
            .await
            .unwrap();

        status = response.status;
        match status {
            CertificateStatus::InError { .. } | CertificateStatus::Settled => {
                break;
            }
            _ => {
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
    }

    assert!(matches!(status, CertificateStatus::InError { .. }));

    scenario.teardown();
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
async fn retry_on_error() {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::no_receipt",
        "return",
    )
    .expect("Failed to configure failpoint");

    // L1 is a RAII guard
    let (_handle, _l1, client) = setup_agglayer(&tmp_dir.path).await;
    let signer = get_signer(0);

    let state = Forest::default().with_signer(signer);

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate.clone()])
        .await
        .unwrap();

    let mut status;

    loop {
        let response: CertificateHeader = client
            .request("interop_getCertificateHeader", rpc_params![certificate_id])
            .await
            .unwrap();

        status = response.status;
        match status {
            CertificateStatus::InError { .. } | CertificateStatus::Settled => {
                break;
            }
            _ => {
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
    }

    assert!(matches!(status, CertificateStatus::InError { .. }));

    fail::cfg(
        "notifier::packer::settle_certificate::receipt_future_ended::no_receipt",
        "off",
    )
    .expect("Failed to configure failpoint");
    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let mut status;

    loop {
        let response: CertificateHeader = client
            .request("interop_getCertificateHeader", rpc_params![certificate_id])
            .await
            .unwrap();

        status = response.status;
        match status {
            CertificateStatus::InError { .. } | CertificateStatus::Settled => {
                break;
            }
            _ => {
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
    }

    assert!(matches!(status, CertificateStatus::Settled));

    scenario.teardown();
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
async fn schedule_two_certs() {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    // L1 is a RAII guard
    let (_handle, _l1, client) = setup_agglayer(&tmp_dir.path).await;
    let signer = get_signer(0);

    let mut state = Forest::default().with_signer(signer);

    let withdrawals = vec![];

    let certificate_one = state.apply_events(&[], &withdrawals);
    let mut certificate_two = state.apply_events(&[], &withdrawals);
    certificate_two.height = 1;

    let certificate_one_id: CertificateId = client
        .request(
            "interop_sendCertificate",
            rpc_params![certificate_one.clone()],
        )
        .await
        .unwrap();

    let certificate_two_id: CertificateId = client
        .request(
            "interop_sendCertificate",
            rpc_params![certificate_two.clone()],
        )
        .await
        .unwrap();
    let mut status;

    loop {
        let response: CertificateHeader = client
            .request(
                "interop_getCertificateHeader",
                rpc_params![certificate_two_id],
            )
            .await
            .unwrap();

        status = response.status;
        match status {
            CertificateStatus::InError { .. } | CertificateStatus::Settled => {
                break;
            }
            _ => {
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
    }

    assert!(matches!(status, CertificateStatus::Settled));

    let response_one: CertificateHeader = client
        .request(
            "interop_getCertificateHeader",
            rpc_params![certificate_one_id],
        )
        .await
        .unwrap();

    assert!(matches!(response_one.status, CertificateStatus::Settled));
    let epoch_number = response_one.epoch_number.unwrap();

    let response_two: CertificateHeader = client
        .request(
            "interop_getCertificateHeader",
            rpc_params![certificate_two_id],
        )
        .await
        .unwrap();

    println!("one: {:?}", response_one);
    println!("two: {:?}", response_two);

    assert!(
        matches!(response_two.epoch_number, Some(epoch_number_two) if epoch_number < epoch_number_two)
    );

    scenario.teardown();
}
