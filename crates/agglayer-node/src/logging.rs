use agglayer_config::log::LogFormat;
use tracing::{Level, Metadata};
use tracing_subscriber::{
    filter::filter_fn, fmt::writer::BoxMakeWriter, prelude::*, util::SubscriberInitExt, EnvFilter,
};

// Reject the pinned dependency records that serialize a full endpoint or
// WebSocket handshake request before any user-configurable formatting layer.
fn is_dependency_record_allowed(metadata: &Metadata<'_>) -> bool {
    let alloy_url_span = metadata.is_span()
        && metadata.target() == "alloy_transport_http::reqwest_transport"
        && metadata.name() == "ReqwestTransport";
    let tungstenite_client_trace = metadata.is_event()
        && metadata.target() == "tungstenite::handshake::client"
        && *metadata.level() == Level::TRACE;

    !alloy_url_span && !tungstenite_client_trace
}

fn subscriber(
    format: LogFormat,
    writer: BoxMakeWriter,
    filter: EnvFilter,
) -> impl tracing::Subscriber + Send + Sync {
    let layer = match format {
        LogFormat::Pretty => tracing_subscriber::fmt::layer()
            .pretty()
            .with_writer(writer)
            .with_filter(filter)
            .boxed(),

        LogFormat::Json => tracing_subscriber::fmt::layer()
            .json()
            .with_writer(writer)
            .with_filter(filter)
            .boxed(),
    };

    tracing_subscriber::Registry::default()
        .with(filter_fn(is_dependency_record_allowed))
        .with(layer)
}

pub(crate) fn tracing(config: &agglayer_config::Log) -> eyre::Result<()> {
    // TODO: Support multiple outputs.
    let writer = config.outputs.first().cloned().unwrap_or_default();
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| config.level.into());

    // We are using try_init because integration test may try to initialize this
    // multiple times.
    subscriber(config.format, writer.as_make_writer(), filter).try_init()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        io,
        sync::{Arc, Mutex},
    };

    use tracing_subscriber::fmt::MakeWriter;

    use super::*;

    const ALLOY_SECRET: &str = "alloy-url-secret-803";
    const TUNGSTENITE_SECRET: &str = "tungstenite-request-secret-803";

    #[derive(Clone, Default)]
    struct Capture(Arc<Mutex<Vec<u8>>>);

    struct CaptureWriter(Arc<Mutex<Vec<u8>>>);

    impl io::Write for CaptureWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for Capture {
        type Writer = CaptureWriter;

        fn make_writer(&'a self) -> Self::Writer {
            CaptureWriter(self.0.clone())
        }
    }

    impl Capture {
        fn output(&self) -> String {
            String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
        }
    }

    // Mirrors pinned dependency callsites; re-audit these identities on upgrade.
    fn emit_dependency_records() {
        let span = tracing::debug_span!(
            target: "alloy_transport_http::reqwest_transport",
            "ReqwestTransport",
            url = ALLOY_SECRET
        );
        let _entered = span.enter();
        tracing::debug!(
            target: "alloy_transport_http::reqwest_transport",
            "alloy-request-control"
        );
        drop(_entered);
        tracing::debug!(
            name: "ReqwestTransport",
            target: "alloy_transport_http::reqwest_transport",
            marker = "same-alloy-name-event-control"
        );

        tracing::trace!(
            target: "tungstenite::handshake::client",
            request = TUNGSTENITE_SECRET,
            "Request"
        );
        tracing::debug!(
            target: "tungstenite::handshake::client",
            "same-target-debug-control"
        );
        tracing::trace!(target: "safe_control", "unrelated-trace-control");
        let span = tracing::trace_span!(
            target: "tungstenite::handshake::client",
            "TungsteniteTraceSpan",
            marker = "tungstenite-trace-span-control"
        );
        let _entered = span.enter();
        tracing::debug!(target: "safe_control", "tungstenite-trace-span-event-control");
        drop(_entered);

        let span = tracing::debug_span!(
            target: "alloy_transport_http::reqwest_transport",
            "OtherTransportSpan",
            marker = "other-alloy-span-field-control"
        );
        let _entered = span.enter();
        tracing::debug!(target: "safe_control", "other-alloy-event-control");
    }

    #[test]
    fn pinned_endpoint_metadata_is_filtered() {
        for format in [LogFormat::Pretty, LogFormat::Json] {
            let capture = Capture::default();
            let filter = EnvFilter::new(
                "off,alloy_transport_http::reqwest_transport=debug,\
                 tungstenite::handshake::client=trace,safe_control=trace",
            );
            let subscriber = subscriber(format, BoxMakeWriter::new(capture.clone()), filter);

            tracing::subscriber::with_default(subscriber, emit_dependency_records);
            let output = capture.output();

            assert!(!output.contains(ALLOY_SECRET));
            assert!(!output.contains(TUNGSTENITE_SECRET));
            for control in [
                "alloy-request-control",
                "same-alloy-name-event-control",
                "same-target-debug-control",
                "unrelated-trace-control",
                "tungstenite-trace-span-control",
                "tungstenite-trace-span-event-control",
                "other-alloy-span-field-control",
                "other-alloy-event-control",
            ] {
                assert!(output.contains(control), "missing {control:?} in {output}");
            }
        }
    }
}
