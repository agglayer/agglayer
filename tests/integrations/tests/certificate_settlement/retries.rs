use std::time::Duration;

use agglayer_storage::tests::TempDBDir;
use agglayer_types::{CertificateHeader, CertificateId, CertificateStatus};
use fail::FailScenario;
use integrations::{
    agglayer_setup::{setup_network, wait_for_condition},
    wait_for_settlement_or_error,
};
use jsonrpsee::{core::client::ClientT as _, rpc_params};
use pessimistic_proof_test_suite::forest::Forest;
use rand::random;
use rstest::rstest;

mod recovery;

const SEND_CERTIFICATE_ERROR: i32 = -10006;

/// Validate that a certificate that has been proven and sent to L1 can't be
/// replaced in the pending-pool.
#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn regression_pushing_certificate_while_settling(#[case] state: Forest) {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    // L1 is a RAII guard
    let (_shutdown_shutdown, _l1, client) = setup_network(&tmp_dir.path, None, None).await;

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);

    let first_certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate.clone()])
        .await
        .unwrap();

    // The real backstop is the outer #[timeout(180s)]; this inner budget only
    // exists to produce a descriptive failure message. It must stay well above
    // realistic settlement latency under the default config (confirmations=12,
    // retry_interval=10s) on 1s Anvil blocks, which can reach ~20-40s under the
    // resource-limited parallel load profile.
    wait_for_condition(
        "first certificate settlement while replacements are rejected",
        Duration::from_secs(150),
        || async {
            let replacement = certificate
                .clone()
                .with_new_local_exit_root(random::<[u8; 32]>().into());
            assert!(client
                .request::<CertificateId, _>("interop_sendCertificate", rpc_params![replacement],)
                .await
                .is_err());

            let header: CertificateHeader = client
                .request(
                    "interop_getCertificateHeader",
                    rpc_params![first_certificate_id],
                )
                .await
                .unwrap();

            matches!(header.status, CertificateStatus::Settled)
        },
    )
    .await;

    let result = wait_for_settlement_or_error!(client, first_certificate_id).await;

    assert!(matches!(result.status, CertificateStatus::Settled));

    scenario.teardown();
}

/// Validate that a certificate that has been proven and sent to L1 can't be
/// replaced in the certificate header store.
#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn regression_pushing_certificate_after_settling(#[case] state: Forest) {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    // L1 is a RAII guard
    let (_shutdown_shutdown, _l1, client) = setup_network(&tmp_dir.path, None, None).await;

    let withdrawals = vec![];

    let certificate = state.clone().apply_events(&[], &withdrawals);

    let first_certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate.clone()])
        .await
        .inspect_err(|err| eprintln!("Error sending first certificate: {err:?}"))
        .unwrap();

    // Send the first certificate. This should be settled.
    wait_for_settlement_or_error!(client, first_certificate_id).await;

    // Verify status is Settled.
    let first_header: CertificateHeader = client
        .request(
            "interop_getCertificateHeader",
            rpc_params![first_certificate_id],
        )
        .await
        .inspect_err(|err| eprintln!("Error getting certificate header: {err:?}"))
        .unwrap();
    assert!(matches!(first_header.status, CertificateStatus::Settled));

    // Send the second certificate, identical to the first and check the error.
    let second_submission_err = client
        .request::<CertificateId, _>("interop_sendCertificate", rpc_params![certificate.clone()])
        .await
        .inspect(|result| eprintln!("Managed to settle same certificate twice! Result: {result:?}"))
        .unwrap_err();
    match second_submission_err {
        jsonrpsee::core::ClientError::Call(error) => {
            assert_eq!(error.code(), SEND_CERTIFICATE_ERROR);
        }
        error => panic!("Unexpected error: {error:?}"),
    }

    // Verify status is Settled. The two submissions have the same ID.
    // This checks the first submission still succeeds.
    let first_header_again: CertificateHeader = client
        .request(
            "interop_getCertificateHeader",
            rpc_params![certificate.hash()],
        )
        .await
        .unwrap();
    assert!(matches!(
        first_header_again.status,
        CertificateStatus::Settled
    ));

    scenario.teardown();
}
