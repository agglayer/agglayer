use std::time::Duration;

use agglayer_storage::tests::TempDBDir;
use agglayer_types::{CertificateHeader, CertificateId, CertificateStatus};
use fail::FailScenario;
use integrations::{
    agglayer_setup::{get_signer, setup_network},
    wait_for_settlement_or_error,
};
use jsonrpsee::core::client::ClientT as _;
use jsonrpsee::rpc_params;
use pessimistic_proof_test_suite::forest::Forest;
use rstest::rstest;

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
async fn schedule_two_certs() {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    // L1 is a RAII guard
    let (_handle, _l1, client) = setup_network(&tmp_dir.path).await;
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

    let result = wait_for_settlement_or_error!(client, certificate_two_id).await;

    assert!(matches!(result.status, CertificateStatus::Settled));

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

    assert!(
        matches!(response_two.epoch_number, Some(epoch_number_two) if
epoch_number < epoch_number_two)
    );

    scenario.teardown();
}
