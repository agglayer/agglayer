use std::time::Duration;

use agglayer_config::tracing::{TracingFormat, TracingOutput};
use anyhow::anyhow;
use opentelemetry::{global, trace::TracerProvider, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
    trace::{BatchConfigBuilder, BatchSpanProcessor, Sampler, SpanLimits},
    Resource,
};
use tracing_subscriber::{prelude::*, util::SubscriberInitExt, EnvFilter};

pub const OTLP_BATCH_SCHEDULED_DELAY: Duration = Duration::from_millis(5_000);
pub const OTLP_BATCH_MAX_QUEUE_SIZE: usize = 2048;
pub const OTLP_BATCH_MAX_EXPORTER_BATCH_SIZE: usize = 512;

pub fn setup_tracing(config: &agglayer_config::TracingConfig, version: &str) -> anyhow::Result<()> {
    let mut layers = Vec::new();

    for writer in &config.outputs {
        // Setup instrumentation if both otlp agent url and
        // otlp service name are provided as arguments
        if writer == &TracingOutput::Otlp {
            let (Some(otlp_agent), Some(otlp_service_name)) =
                (&config.otlp_agent, &config.otlp_service_name)
            else {
                anyhow::bail!(
                    "Otlp tracing requires both otlp agent url and otlp service provided"
                );
            };

            let resources = build_resources(otlp_service_name, version);
            let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(otlp_agent)
                .build()?;

            let batch_processor_config = BatchConfigBuilder::default()
                .with_scheduled_delay(match std::env::var("OTLP_BATCH_SCHEDULED_DELAY") {
                    Ok(v) => {
                        if let Ok(millis) = v.parse::<u64>() {
                            Duration::from_millis(millis)
                        } else {
                            OTLP_BATCH_SCHEDULED_DELAY
                        }
                    }
                    _ => OTLP_BATCH_SCHEDULED_DELAY,
                })
                .with_max_queue_size(match std::env::var("OTLP_BATCH_MAX_QUEUE_SIZE") {
                    Ok(v) => v.parse::<usize>().unwrap_or(OTLP_BATCH_MAX_QUEUE_SIZE),
                    _ => OTLP_BATCH_MAX_QUEUE_SIZE,
                })
                .with_max_export_batch_size(
                    match std::env::var("OTLP_BATCH_MAX_EXPORTER_BATCH_SIZE") {
                        Ok(v) => v
                            .parse::<usize>()
                            .unwrap_or(OTLP_BATCH_MAX_EXPORTER_BATCH_SIZE),
                        _ => OTLP_BATCH_MAX_EXPORTER_BATCH_SIZE,
                    },
                );

            let span_limits = {
                let mut span_limits = SpanLimits::default();
                if let Ok(max_events) = std::env::var("OTLP_MAX_EVENTS_PER_SPAN") {
                    if let Ok(value) = max_events.parse::<u32>() {
                        span_limits.max_events_per_span = value;
                    }
                }

                if let Ok(max_attributes) = std::env::var("OTLP_MAX_ATTRIBUTES_PER_SPAN") {
                    if let Ok(value) = max_attributes.parse::<u32>() {
                        span_limits.max_attributes_per_span = value;
                    }
                }

                if let Ok(max_links_per_span) = std::env::var("OTLP_MAX_LINKS_PER_SPAN") {
                    if let Ok(value) = max_links_per_span.parse::<u32>() {
                        span_limits.max_links_per_span = value;
                    }
                }

                if let Ok(max_attributes_per_event) = std::env::var("OTLP_MAX_ATTRIBUTES_PER_EVENT")
                {
                    if let Ok(value) = max_attributes_per_event.parse::<u32>() {
                        span_limits.max_attributes_per_event = value;
                    }
                }

                if let Ok(max_attributes_per_link) = std::env::var("OTLP_MAX_ATTRIBUTES_PER_LINK") {
                    if let Ok(value) = max_attributes_per_link.parse::<u32>() {
                        span_limits.max_attributes_per_link = value;
                    }
                }
                span_limits
            };

            // Ensure that the span limits are not too low
            let trace_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
                .with_sampler(Sampler::AlwaysOn)
                .with_span_limits(span_limits)
                .with_resource(Resource::builder().with_attributes(resources).build())
                .with_span_processor(
                    BatchSpanProcessor::builder(otlp_exporter)
                        .with_batch_config(batch_processor_config.build())
                        .build(),
                )
                .build();

            let tracer = trace_provider.tracer("agglayer-otlp");

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
            layers.push(match config.format {
                TracingFormat::Pretty => tracing_subscriber::fmt::layer()
                    .pretty()
                    .with_writer(writer.as_make_writer())
                    .with_filter(
                        EnvFilter::try_from_default_env().unwrap_or_else(|_| config.level.into()),
                    )
                    .boxed(),

                TracingFormat::Json => tracing_subscriber::fmt::layer()
                    .json()
                    .with_writer(writer.as_make_writer())
                    .with_filter(
                        EnvFilter::try_from_default_env().unwrap_or_else(|_| config.level.into()),
                    )
                    .boxed(),
            });
        }
    }

    // We are using try_init because integration test may try
    // to initialize this multiple times.
    tracing_subscriber::Registry::default()
        .with(layers)
        .try_init()
        .map_err(|e| anyhow!("Unable to initialize tracing subscriber: {e:?}"))?;

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
        .filter_map(|tag_raw| {
            let mut v = tag_raw.splitn(2, '=');
            match (v.next(), v.next()) {
                (Some(key), Some(value)) if !key.trim().is_empty() && !value.trim().is_empty() => {
                    Some(KeyValue::new(
                        key.trim().to_string(),
                        value.trim().to_string(),
                    ))
                }
                _ => {
                    eprint!(
                        "Invalid AGGLAYER_OTLP_TAGS entry: {tag_raw}. Expected format: key=value"
                    );
                    None
                }
            }
        })
        .collect();

    resources.extend(custom_resources);

    resources
}
