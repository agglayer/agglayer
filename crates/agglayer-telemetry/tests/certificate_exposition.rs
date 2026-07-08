//! Verifies the Prometheus exposition of the certificate metrics: the exact
//! metric names, the `network_id`/`stage` labels, and the `_total` suffix on
//! the errors counter. Runs in its own test binary so it fully owns the global
//! OpenTelemetry meter provider.

use agglayer_telemetry::certificate::{
    record_certificate_error, record_certificate_settled_height,
    record_certificate_stage_completed, record_certificate_total_duration, stage,
};
use opentelemetry::global;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use prometheus::{Encoder, Registry, TextEncoder};

#[test]
fn certificate_metrics_are_exposed_with_expected_names_and_labels() {
    // Wire the OTEL global meter provider to a Prometheus registry, exactly as
    // `ServerBuilder::init_meter_provider` does.
    let registry = Registry::new();
    let exporter = opentelemetry_prometheus::exporter()
        .with_registry(registry.clone())
        .build()
        .unwrap();
    let provider = SdkMeterProvider::builder().with_reader(exporter).build();
    global::set_meter_provider(provider);

    // Record one sample of each metric, across two stages / networks.
    record_certificate_stage_completed(1, stage::PENDING, 1.5);
    record_certificate_stage_completed(1, stage::PROVEN, 0.2);
    record_certificate_stage_completed(1, stage::CANDIDATE, 30.0);
    record_certificate_total_duration(1, 31.7);
    record_certificate_settled_height(1, 42);
    record_certificate_error(2, stage::CANDIDATE);

    let mut buf = Vec::new();
    TextEncoder::new()
        .encode(&registry.gather(), &mut buf)
        .unwrap();
    let text = String::from_utf8(buf).unwrap();
    println!("---- /metrics ----\n{text}\n------------------");

    // Metric names (histograms expose _bucket/_sum/_count; the counter gets a
    // `_total` suffix from the exporter).
    assert!(text.contains("agglayer_certificate_duration_seconds_bucket"));
    assert!(text.contains("agglayer_certificate_stage_duration_seconds_bucket"));
    assert!(text.contains("agglayer_certificate_settled_height"));
    assert!(text.contains("agglayer_certificate_errors_total"));

    // Labels.
    assert!(text.contains("network_id=\"1\""));
    assert!(text.contains("stage=\"pending\""));
    assert!(text.contains("stage=\"proven\""));
    assert!(text.contains("stage=\"candidate\""));

    // The settled-height gauge carries the recorded height.
    assert!(text.contains("agglayer_certificate_settled_height{"));
}
