use std::time::Duration;

use agglayer_config::tracing::{TracingFormat, TracingOutput};
use opentelemetry::{global, trace::TracerProvider, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
    trace::{BatchConfigBuilder, BatchSpanProcessor, Sampler, SpanLimits},
    Resource,
};
use tracing_subscriber::{prelude::*, util::SubscriberInitExt, EnvFilter};

pub fn setup_tracing(config: &agglayer_config::Tracing, version: &str) -> anyhow::Result<()> {
    let writer = config.outputs.first().cloned().unwrap_or_default();

    let mut layers = Vec::new();

    // Setup instrumentation if both otlp agent url and
    // otlp service name are provided as arguments
    if config
        .outputs
        .iter()
        .any(|output| *output == TracingOutput::Otlp)
    {
        if let (Some(otlp_agent), Some(otlp_service_name)) =
            (&config.otlp_agent, &config.otlp_service_name)
        {
            let resources = build_resources(otlp_service_name, version);
            let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(otlp_agent)
                .build()?;

            let batch_processor_config = BatchConfigBuilder::default()
                .with_scheduled_delay(match std::env::var("OTLP_BATCH_SCHEDULED_DELAY") {
                    Ok(v) => Duration::from_millis(v.parse::<u64>().unwrap_or(5_000)),
                    _ => Duration::from_millis(5_000),
                })
                .with_max_queue_size(match std::env::var("OTLP_BATCH_MAX_QUEUE_SIZE") {
                    Ok(v) => v.parse::<usize>().unwrap_or(2048),
                    _ => 2048,
                })
                .with_max_export_batch_size(
                    match std::env::var("OTLP_BATCH_MAX_EXPORTER_BATCH_SIZE") {
                        Ok(v) => v.parse::<usize>().unwrap_or(512),
                        _ => 512,
                    },
                );

            let span_limits_default = SpanLimits::default();

            let trace_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
                .with_sampler(Sampler::AlwaysOn)
                .with_max_events_per_span(match std::env::var("OTLP_MAX_EVENTS_PER_SPAN") {
                    Ok(v) => v
                        .parse::<u32>()
                        .unwrap_or(span_limits_default.max_events_per_span),
                    _ => span_limits_default.max_events_per_span,
                })
                .with_max_attributes_per_span(match std::env::var("OTLP_MAX_ATTRIBUTES_PER_SPAN") {
                    Ok(v) => v
                        .parse::<u32>()
                        .unwrap_or(span_limits_default.max_attributes_per_span),
                    _ => span_limits_default.max_attributes_per_span,
                })
                .with_max_links_per_span(match std::env::var("OTLP_MAX_LINK_PER_SPAN") {
                    Ok(v) => v
                        .parse::<u32>()
                        .unwrap_or(span_limits_default.max_links_per_span),
                    _ => span_limits_default.max_links_per_span,
                })
                .with_max_attributes_per_event(
                    match std::env::var("OTLP_MAX_ATTRIBUTES_PER_EVENT") {
                        Ok(v) => v
                            .parse::<u32>()
                            .unwrap_or(span_limits_default.max_attributes_per_event),
                        _ => span_limits_default.max_attributes_per_event,
                    },
                )
                .with_max_attributes_per_link(match std::env::var("OTLP_MAX_ATTRIBUTES_PER_LINK") {
                    Ok(v) => v
                        .parse::<u32>()
                        .unwrap_or(span_limits_default.max_attributes_per_link),
                    _ => span_limits_default.max_attributes_per_link,
                })
                .with_resource(Resource::builder().with_attributes(resources).build())
                .with_span_processor(
                    BatchSpanProcessor::builder(otlp_exporter)
                        .with_batch_config(batch_processor_config.build())
                        .build(),
                )
                .build();

            let tracer = trace_provider
                .tracer("agglayer-otlp");

            let _ = global::set_tracer_provider(trace_provider);

            layers.push(
                tracing_opentelemetry::layer()
                    .with_tracer(tracer)
                    .with_filter(
                        EnvFilter::try_from_default_env().unwrap_or_else(|_| config.level.into()),
                    )
                    .boxed(),
            );

            global::set_text_map_propagator(TraceContextPropagator::new());
        } else {
            anyhow::bail!("Otlp tracing requires both otlp agent url and otlp service provided");
        }
    }

    layers.push(match config.format {
        TracingFormat::Pretty => tracing_subscriber::fmt::layer()
            .pretty()
            .with_writer(writer.as_make_writer())
            .with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| config.level.into()))
            .boxed(),

        TracingFormat::Json => tracing_subscriber::fmt::layer()
            .json()
            .with_writer(writer.as_make_writer())
            .with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| config.level.into()))
            .boxed(),
    });

    // We are using try_init because integration test may try
    // to initialize this multiple times.
    _ = tracing_subscriber::Registry::default()
        .with(layers)
        .try_init();

    tracing::info!("Tracing initialized with config: {config:?}");

    Ok(())
}

fn build_resources(otlp_service_name: &str, version: &str) -> Vec<KeyValue> {
    let mut resources = Vec::new();

    resources.push(KeyValue::new("service.name", otlp_service_name.to_string()));
    resources.push(KeyValue::new("service.version", version.to_string()));

    let custom_resources: Vec<_> = std::env::var("AGGLAYER_OTLP_TAGS")
        .unwrap_or_default()
        .split(',')
        // NOTE: limit to 10 tags to avoid exploit
        .take(10)
        .filter_map(|tag_raw| {
            let mut v = tag_raw.splitn(2, '=');
            match (v.next(), v.next()) {
                (Some(key), Some(value)) if !key.trim().is_empty() && !value.trim().is_empty() => {
                    Some(KeyValue::new(
                        key.trim().to_string(),
                        value.trim().to_string(),
                    ))
                }
                _ => None,
            }
        })
        .collect();

    resources.extend(custom_resources);

    resources
}
