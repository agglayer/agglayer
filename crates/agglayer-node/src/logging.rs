use agglayer_config::log::LogFormat;
use tracing::{Level, Metadata};
use tracing_subscriber::{
    filter::filter_fn, fmt::writer::BoxMakeWriter, prelude::*, util::SubscriberInitExt, EnvFilter,
};

// Reject pinned records that expose a full endpoint, HTTP authority, or
// WebSocket handshake request before any user-configurable formatting layer.
// The dependencies do not give every sensitive and benign record distinct
// metadata, so same-target records at the filtered levels are also suppressed.
fn is_dependency_record_allowed(metadata: &Metadata<'_>) -> bool {
    let alloy_url_span = metadata.is_span()
        && metadata.target() == "alloy_transport_http::reqwest_transport"
        && metadata.name() == "ReqwestTransport";
    let reqwest_connect = metadata.is_event()
        && metadata.target() == "reqwest::connect"
        && *metadata.level() == Level::DEBUG;
    let hyper_connector = metadata.is_event()
        && metadata.target() == "hyper_util::client::legacy::connect::http"
        && *metadata.level() == Level::TRACE;
    let hyper_pool = metadata.is_event()
        && metadata.target() == "hyper_util::client::legacy::pool"
        && matches!(*metadata.level(), Level::DEBUG | Level::TRACE);
    let tungstenite_client_trace = metadata.is_event()
        && metadata.target() == "tungstenite::handshake::client"
        && *metadata.level() == Level::TRACE;

    !alloy_url_span
        && !reqwest_connect
        && !hyper_connector
        && !hyper_pool
        && !tungstenite_client_trace
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
        time::Duration,
    };

    use alloy::{
        pubsub::PubSubConnect,
        rpc::json_rpc::{Id, Request, RequestPacket},
        transports::{http::Http, ws::WsConnect},
    };
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
        time::timeout,
    };
    use tower::Service;
    use tracing_subscriber::fmt::MakeWriter;

    use super::*;

    const ALLOY_SECRET: &str = "alloy-url-secret-803";
    const HTTP_HOST_SECRET: &str = "http-host-secret-803.invalid";
    const TUNGSTENITE_SECRET: &str = "tungstenite-request-secret-803";
    const HTTP_PATH_SECRET: &str = "http-path-secret-803";
    const HTTP_QUERY_SECRET: &str = "http-query-secret-803";
    const WS_PATH_SECRET: &str = "ws-path-secret-803";
    const WS_QUERY_SECRET: &str = "ws-query-secret-803";
    const WS_BASIC_AUTH: &str = "Basic d3MtdXNlci04MDM6d3MtcGFzcy04MDM=";

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
            target: "reqwest::connect",
            host = HTTP_HOST_SECRET,
            "reqwest-host-secret"
        );
        tracing::trace!(
            target: "hyper_util::client::legacy::connect::http",
            host = HTTP_HOST_SECRET,
            "hyper-connector-host-secret"
        );
        tracing::debug!(
            target: "hyper_util::client::legacy::pool",
            authority = HTTP_HOST_SECRET,
            "hyper-pool-debug-secret"
        );
        tracing::trace!(
            target: "hyper_util::client::legacy::pool",
            authority = HTTP_HOST_SECRET,
            "hyper-pool-trace-secret"
        );
        tracing::debug!(
            target: "tungstenite::handshake::client",
            "same-target-debug-control"
        );
        tracing::trace!(target: "reqwest::connect", "reqwest-trace-control");
        tracing::debug!(
            target: "hyper_util::client::legacy::connect::http",
            "hyper-connector-debug-control"
        );
        tracing::info!(
            target: "hyper_util::client::legacy::pool",
            "hyper-pool-info-control"
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
                 tungstenite::handshake::client=trace,reqwest::connect=trace,\
                 hyper_util::client::legacy::connect::http=trace,\
                 hyper_util::client::legacy::pool=trace,safe_control=trace",
            );
            let subscriber = subscriber(format, BoxMakeWriter::new(capture.clone()), filter);

            tracing::subscriber::with_default(subscriber, emit_dependency_records);
            let output = capture.output();

            assert!(!output.contains(ALLOY_SECRET));
            assert!(!output.contains(HTTP_HOST_SECRET));
            assert!(!output.contains(TUNGSTENITE_SECRET));
            for control in [
                "alloy-request-control",
                "same-alloy-name-event-control",
                "same-target-debug-control",
                "reqwest-trace-control",
                "hyper-connector-debug-control",
                "hyper-pool-info-control",
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

    fn subscriber_without_endpoint_policy(
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

        tracing_subscriber::Registry::default().with(layer)
    }

    async fn emit_actual_dependency_records() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0_u8; 2048];
            let _ = stream.read(&mut request).await.unwrap();
            let body = r#"{"jsonrpc":"2.0","id":1,"result":"0x1"}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: \
                 {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });

        let client = reqwest13::Client::builder()
            .no_proxy()
            .resolve(HTTP_HOST_SECRET, address)
            .build()
            .unwrap();
        let url = format!(
            "http://{HTTP_HOST_SECRET}:{}/{HTTP_PATH_SECRET}?key={HTTP_QUERY_SECRET}",
            address.port()
        )
        .parse()
        .unwrap();
        let request = Request::new("eth_blockNumber", Id::Number(1), ())
            .serialize()
            .unwrap();
        let mut transport = Http::with_client(client, url);
        let _ = transport.call(RequestPacket::Single(request)).await;
        server.await.unwrap();

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0_u8; 2048];
            let _ = stream.read(&mut request).await.unwrap();
            stream
                .write_all(
                    b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .await
                .unwrap();
        });

        let ws = WsConnect::new(format!(
            "ws://ws-user-803:ws-pass-803@127.0.0.1:{port}/{WS_PATH_SECRET}?key={WS_QUERY_SECRET}"
        ));
        let _ = ws.connect().await;
        server.await.unwrap();
    }

    async fn capture_actual_dependency_records<S>(subscriber: S, capture: Capture) -> String
    where
        S: tracing::Subscriber + Send + Sync + 'static,
    {
        let dispatch = tracing::Dispatch::new(subscriber);
        let guard = tracing::dispatcher::set_default(&dispatch);
        timeout(Duration::from_secs(5), emit_actual_dependency_records())
            .await
            .expect("dependency transports did not finish");
        drop(guard);
        capture.output()
    }

    #[tokio::test(flavor = "current_thread")]
    async fn actual_dependency_transports_do_not_log_endpoint_credentials() {
        tracing_log::LogTracer::init().expect("test process must own the log bridge");

        let filter = || {
            EnvFilter::new(
                "off,alloy_transport_http::reqwest_transport=debug,\
                 tungstenite::handshake::client=trace,reqwest::connect=debug,\
                 hyper_util::client::legacy::connect::http=trace,\
                 hyper_util::client::legacy::pool=trace",
            )
        };
        let secrets = [
            HTTP_HOST_SECRET,
            HTTP_PATH_SECRET,
            HTTP_QUERY_SECRET,
            WS_PATH_SECRET,
            WS_QUERY_SECRET,
            WS_BASIC_AUTH,
        ];
        let sensitive_targets = [
            "alloy_transport_http::reqwest_transport",
            "reqwest::connect",
            "hyper_util::client::legacy::connect::http",
            "hyper_util::client::legacy::pool",
            "tungstenite::handshake::client",
        ];

        for format in [LogFormat::Pretty, LogFormat::Json] {
            let capture = Capture::default();
            let unfiltered = subscriber_without_endpoint_policy(
                format,
                BoxMakeWriter::new(capture.clone()),
                filter(),
            );
            let output = capture_actual_dependency_records(unfiltered, capture).await;
            for secret in secrets {
                assert!(output.contains(secret), "missing {secret:?} in {output}");
            }
            for target in sensitive_targets {
                assert!(output.contains(target), "missing {target:?} in {output}");
            }

            let capture = Capture::default();
            let filtered = subscriber(format, BoxMakeWriter::new(capture.clone()), filter());
            let output = capture_actual_dependency_records(filtered, capture).await;
            for secret in secrets {
                assert!(!output.contains(secret), "found {secret:?} in {output}");
            }
        }
    }
}
