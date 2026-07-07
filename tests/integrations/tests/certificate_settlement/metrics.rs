use std::time::Duration;

use agglayer_storage::tests::TempDBDir;
use agglayer_types::{CertificateId, CertificateStatus};
use fail::FailScenario;
use integrations::{agglayer_setup::setup_network_with_config, wait_for_settlement_or_error};
use jsonrpsee::{core::client::ClientT as _, rpc_params};
use pessimistic_proof_test_suite::forest::Forest;
use rstest::rstest;

const METRICS_POLL_TIMEOUT: Duration = Duration::from_secs(30);
const METRICS_POLL_INTERVAL: Duration = Duration::from_millis(500);

/// Scrape the metrics page and extract the value of `metric` for
/// `network_id`, if the series is currently exported.
async fn sample_value(metrics_url: &str, metric: &str, network_id: u32) -> Option<u64> {
    let body = reqwest::get(metrics_url).await.ok()?.text().await.ok()?;
    let label = format!("network_id=\"{network_id}\"");
    body.lines()
        .find(|line| line.starts_with(&format!("{metric}{{")) && line.contains(&label))
        .and_then(|line| line.rsplit(' ').next()?.parse().ok())
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn per_network_height_metrics_are_exposed(#[case] state: Forest) {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    // L1 is a RAII guard
    let (_handle, _l1, client, config) = setup_network_with_config(&tmp_dir.path, None, None).await;
    let metrics_url = format!("http://{}/metrics", config.telemetry.addr);

    let withdrawals = vec![];
    let certificate = state.clone().apply_events(&[], &withdrawals);
    let network_id = certificate.network_id.to_u32();

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    let result = wait_for_settlement_or_error!(client, certificate_id).await;
    assert_eq!(result.status, CertificateStatus::Settled);

    // The latest-settled pointer is written shortly after the header flips
    // to Settled; poll until the gauge appears.
    let start = tokio::time::Instant::now();
    let settled_height = loop {
        if let Some(value) =
            sample_value(&metrics_url, "agglayer_network_settled_height", network_id).await
        {
            break value;
        }
        assert!(
            start.elapsed() < METRICS_POLL_TIMEOUT,
            "settled height metric did not appear within {METRICS_POLL_TIMEOUT:?}"
        );
        tokio::time::sleep(METRICS_POLL_INTERVAL).await;
    };
    assert_eq!(settled_height, 0);

    assert_eq!(
        sample_value(&metrics_url, "agglayer_network_pending_height", network_id).await,
        Some(0),
    );
    assert_eq!(
        sample_value(&metrics_url, "agglayer_network_proven_height", network_id).await,
        Some(0),
    );
    assert_eq!(
        sample_value(
            &metrics_url,
            "agglayer_network_latest_certificate_in_error",
            network_id
        )
        .await,
        Some(0),
    );

    scenario.teardown();
}
