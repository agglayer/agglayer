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

/// Scrape the metrics page, returning the raw prometheus text body if the
/// endpoint responded.
async fn fetch_metrics(metrics_url: &str) -> Option<String> {
    reqwest::get(metrics_url).await.ok()?.text().await.ok()
}

/// Extract the value of `metric` for `network_id` (and the `stage` label,
/// when given) from a scraped metrics body, if the series is exported in
/// that snapshot.
fn sample_value(body: &str, metric: &str, network_id: u32, stage: Option<&str>) -> Option<u64> {
    let prefix = format!("{metric}{{");
    let network_label = format!("network_id=\"{network_id}\"");
    let stage_label = stage.map(|stage| format!("stage=\"{stage}\""));
    body.lines()
        .find(|line| {
            line.starts_with(&prefix)
                && line.contains(&network_label)
                && stage_label
                    .as_ref()
                    .is_none_or(|label| line.contains(label))
        })
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
    // to Settled; poll until the gauge appears, then evaluate every gauge
    // from that scrape so all values come from one consistent snapshot.
    let start = tokio::time::Instant::now();
    let body = loop {
        let last_observation = match fetch_metrics(&metrics_url).await {
            Some(body) => {
                if sample_value(
                    &body,
                    "agglayer_node_network_height",
                    network_id,
                    Some("settled"),
                )
                .is_some()
                {
                    break body;
                }

                format!(
                    "settled height series for network {network_id} absent from metrics body ({} \
                     bytes)",
                    body.len()
                )
            }
            None => "failed to fetch the metrics page".to_string(),
        };

        assert!(
            start.elapsed() < METRICS_POLL_TIMEOUT,
            "settled height metric did not appear within {METRICS_POLL_TIMEOUT:?}. Last \
             observation: {last_observation}"
        );
        tokio::time::sleep(METRICS_POLL_INTERVAL).await;
    };

    for stage in ["settled", "pending", "proven"] {
        assert_eq!(
            sample_value(
                &body,
                "agglayer_node_network_height",
                network_id,
                Some(stage)
            ),
            Some(0),
            "stage {stage}, metrics body:\n{body}"
        );
    }
    assert_eq!(
        sample_value(
            &body,
            "agglayer_node_network_latest_certificate_in_error",
            network_id,
            None
        ),
        Some(0),
        "metrics body:\n{body}"
    );

    scenario.teardown();
}
