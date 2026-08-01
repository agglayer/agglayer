use std::sync::{Mutex, OnceLock};

use agglayer_config::log::LogFormat;
use tracing::{Level, Metadata};
use tracing_subscriber::{
    filter::{filter_fn, FilterExt},
    fmt::writer::BoxMakeWriter,
    prelude::*,
    util::SubscriberInitExt,
    EnvFilter,
};

static SUBSCRIBER_INIT: Mutex<()> = Mutex::new(());
static SUBSCRIBER_INSTALLED: OnceLock<()> = OnceLock::new();

// Reject pinned records that expose a full endpoint, HTTP authority, WebSocket
// handshake request, or TLS server name before any user-configurable formatting
// layer. The dependencies do not give every sensitive and benign record
// distinct metadata, so same-target records at the filtered levels are also
// suppressed.
fn is_dependency_record_allowed(metadata: &Metadata<'_>) -> bool {
    let alloy_url_span = metadata.is_span()
        && metadata.target() == "alloy_transport_http::reqwest_transport"
        && metadata.name() == "ReqwestTransport"
        && *metadata.level() == Level::DEBUG;
    let alloy_hyper_url_span = metadata.is_span()
        && metadata.target() == "alloy_transport_http::hyper_transport"
        && metadata.name() == "HyperTransport"
        && *metadata.level() == Level::DEBUG;
    let reqwest_connect = metadata.is_event()
        && metadata.target() == "reqwest::connect"
        && matches!(*metadata.level(), Level::DEBUG | Level::TRACE);
    let hyper_connector = metadata.is_event()
        && metadata.target() == "hyper_util::client::legacy::connect::http"
        && matches!(*metadata.level(), Level::DEBUG | Level::TRACE);
    let hyper_pool = metadata.is_event()
        && metadata.target() == "hyper_util::client::legacy::pool"
        && matches!(*metadata.level(), Level::DEBUG | Level::TRACE);
    let hyper_client = metadata.is_event()
        && metadata.target() == "hyper_util::client::legacy::client"
        && matches!(*metadata.level(), Level::DEBUG | Level::WARN);
    let tungstenite_client = metadata.is_event()
        && metadata.target() == "tungstenite::client"
        && *metadata.level() == Level::DEBUG;
    let tungstenite_client_trace = metadata.is_event()
        && metadata.target() == "tungstenite::handshake::client"
        && *metadata.level() == Level::TRACE;
    let jsonrpsee_client = metadata.is_event()
        && metadata.target() == "jsonrpsee-client"
        && *metadata.level() == Level::DEBUG;
    let rustls_client_handshake = metadata.is_event()
        && metadata.target() == "rustls::client::hs"
        && matches!(*metadata.level(), Level::DEBUG | Level::TRACE);
    let rustls_client_tls12 = metadata.is_event()
        && metadata.target() == "rustls::client::tls12"
        && matches!(*metadata.level(), Level::DEBUG | Level::TRACE);

    !alloy_url_span
        && !alloy_hyper_url_span
        && !reqwest_connect
        && !hyper_connector
        && !hyper_pool
        && !hyper_client
        && !tungstenite_client
        && !tungstenite_client_trace
        && !jsonrpsee_client
        && !rustls_client_handshake
        && !rustls_client_tls12
}

fn subscriber(
    format: LogFormat,
    writer: BoxMakeWriter,
    filter: EnvFilter,
) -> impl tracing::Subscriber + Send + Sync {
    // Keep both filters on the formatting layer so disabled callsites retain
    // `Interest::never`.
    let filter = filter.and(filter_fn(is_dependency_record_allowed));
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

pub(crate) fn tracing(config: &agglayer_config::Log) -> eyre::Result<()> {
    if SUBSCRIBER_INSTALLED.get().is_some() {
        return Ok(());
    }

    let _init_guard = SUBSCRIBER_INIT
        .lock()
        .expect("Agglayer logging subscriber initialization lock was poisoned");
    if SUBSCRIBER_INSTALLED.get().is_some() {
        return Ok(());
    }

    // TODO: Support multiple outputs.
    let writer = config.outputs.first().cloned().unwrap_or_default();
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| config.level.into());

    // Repeated node starts may reuse the subscriber installed by Agglayer. A
    // subscriber installed by another component is rejected: continuing would
    // leave dependency endpoint records outside this redaction policy.
    subscriber(config.format, writer.as_make_writer(), filter)
        .try_init()
        .map_err(|error| {
            eyre::eyre!(
                "Agglayer logging requires its endpoint-redaction policy, but the global tracing \
                 subscriber is already initialized: {error}"
            )
        })?;
    let _ = SUBSCRIBER_INSTALLED.set(());

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        env, io,
        num::NonZeroU64,
        process::Command,
        sync::{Arc, Mutex},
        time::Duration,
    };

    use agglayer_clock::BlockClock;
    use alloy::{
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
    const TLS_HOST_SECRET: &str = "tls-host-secret-803.localhost";
    const JSONRPSEE_SECRET: &str = "jsonrpsee-url-secret-803";
    const TUNGSTENITE_SECRET: &str = "tungstenite-request-secret-803";
    const HTTP_PATH_SECRET: &str = "http-path-secret-803";
    const HTTP_QUERY_SECRET: &str = "http-query-secret-803";
    const WS_USERNAME: &str = "ws-user-803";
    const WS_PASSWORD: &str = "ws-pass-803";
    const WS_PATH_SECRET: &str = "ws-path-secret-803";
    const WS_QUERY_SECRET: &str = "ws-query-secret-803";
    const WS_BASIC_AUTH: &str = "Basic d3MtdXNlci04MDM6d3MtcGFzcy04MDM=";
    const BLOCK_CLOCK_FAILURE: &str = "Failed to start BlockClock";

    static UNRELATED_DEBUG_CALLSITE: tracing::callsite::DefaultCallsite =
        tracing::callsite::DefaultCallsite::new(&UNRELATED_DEBUG_METADATA);
    static UNRELATED_DEBUG_METADATA: tracing::Metadata<'static> = tracing::metadata! {
        name: "unrelated_debug_callsite",
        target: "unrelated_target",
        level: tracing::Level::DEBUG,
        fields: tracing::fieldset!(),
        callsite: &UNRELATED_DEBUG_CALLSITE,
        kind: tracing::metadata::Kind::EVENT,
    };

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

        let span = tracing::info_span!(
            target: "alloy_transport_http::reqwest_transport",
            "ReqwestTransport",
            marker = "alloy-info-span-control"
        );
        let _entered = span.enter();
        tracing::debug!(target: "safe_control", "alloy-info-span-event-control");
        drop(_entered);

        tracing::debug!(
            name: "ReqwestTransport",
            target: "alloy_transport_http::reqwest_transport",
            marker = "same-alloy-name-event-control"
        );

        let span = tracing::debug_span!(
            target: "alloy_transport_http::hyper_transport",
            "HyperTransport",
            url = ALLOY_SECRET
        );
        let _entered = span.enter();
        tracing::debug!(target: "safe_control", "hyper-alloy-event-control");
        drop(_entered);
        let span = tracing::info_span!(
            target: "alloy_transport_http::hyper_transport",
            "HyperTransport",
            marker = "hyper-alloy-info-span-control"
        );
        let _entered = span.enter();
        tracing::debug!(target: "safe_control", "hyper-alloy-info-span-event-control");
        drop(_entered);

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
            target: "reqwest::connect",
            request = HTTP_QUERY_SECRET,
            "reqwest-trace-secret"
        );
        tracing::info!(target: "reqwest::connect", "reqwest-info-control");
        tracing::trace!(
            target: "hyper_util::client::legacy::connect::http",
            host = HTTP_HOST_SECRET,
            "hyper-connector-host-secret"
        );
        tracing::debug!(
            target: "hyper_util::client::legacy::connect::http",
            host = HTTP_HOST_SECRET,
            "hyper-connector-debug-secret"
        );
        tracing::info!(
            target: "hyper_util::client::legacy::connect::http",
            "hyper-connector-info-control"
        );
        tracing::warn!(
            target: "hyper_util::client::legacy::client",
            path = HTTP_PATH_SECRET,
            "hyper-client-connect-path"
        );
        tracing::debug!(
            target: "hyper_util::client::legacy::client",
            uri = HTTP_QUERY_SECRET,
            "hyper-client-uri"
        );
        tracing::info!(
            target: "hyper_util::client::legacy::client",
            "hyper-client-info-control"
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
        tracing::debug!(
            target: "tungstenite::client",
            uri = JSONRPSEE_SECRET,
            "tungstenite-client-secret"
        );
        tracing::info!(
            target: "tungstenite::client",
            "tungstenite-client-info-control"
        );
        tracing::debug!(
            target: "jsonrpsee-client",
            target_url = JSONRPSEE_SECRET,
            "jsonrpsee-target-secret"
        );
        tracing::info!(
            target: "jsonrpsee-client",
            "jsonrpsee-info-control"
        );
        tracing::info!(
            target: "hyper_util::client::legacy::pool",
            "hyper-pool-info-control"
        );
        tracing::debug!(
            target: "rustls::client::hs",
            server_name = TLS_HOST_SECRET,
            "rustls-server-name"
        );
        tracing::trace!(
            target: "rustls::client::hs",
            client_hello = TLS_HOST_SECRET,
            "rustls-client-hello"
        );
        tracing::info!(target: "rustls::client::hs", "rustls-info-control");
        tracing::debug!(
            target: "rustls::client::tls12",
            server_name = TLS_HOST_SECRET,
            "rustls-tls12-server-name"
        );
        tracing::trace!(
            target: "rustls::client::tls12",
            server_certificate = TLS_HOST_SECRET,
            "rustls-tls12-certificate"
        );
        tracing::info!(
            target: "rustls::client::tls12",
            "rustls-tls12-info-control"
        );
        tracing::debug!(
            target: "rustls::client::common",
            "rustls-nearby-target-control"
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
            target: "rustls::client::hs",
            "RustlsHandshakeSpan",
            marker = "rustls-debug-span-control"
        );
        let _entered = span.enter();
        tracing::debug!(target: "safe_control", "rustls-debug-span-event-control");
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
                 alloy_transport_http::hyper_transport=debug,tungstenite::handshake::client=trace,\
                 reqwest::connect=trace,hyper_util::client::legacy::connect::http=trace,\
                 hyper_util::client::legacy::pool=trace,hyper_util::client::legacy::client=debug,\
                 tungstenite::client=debug,jsonrpsee-client=debug,rustls::client::hs=trace,\
                 rustls::client::tls12=trace,rustls::client::common=debug,safe_control=trace",
            );
            let subscriber = subscriber(format, BoxMakeWriter::new(capture.clone()), filter);

            tracing::subscriber::with_default(subscriber, emit_dependency_records);
            let output = capture.output();

            assert!(!output.contains(ALLOY_SECRET));
            assert!(!output.contains(HTTP_HOST_SECRET));
            assert!(!output.contains(HTTP_PATH_SECRET));
            assert!(!output.contains(HTTP_QUERY_SECRET));
            assert!(!output.contains(TUNGSTENITE_SECRET));
            assert!(!output.contains(TLS_HOST_SECRET));
            assert!(!output.contains(JSONRPSEE_SECRET));
            for control in [
                "alloy-request-control",
                "alloy-info-span-control",
                "alloy-info-span-event-control",
                "same-alloy-name-event-control",
                "hyper-alloy-event-control",
                "hyper-alloy-info-span-control",
                "hyper-alloy-info-span-event-control",
                "same-target-debug-control",
                "reqwest-info-control",
                "hyper-connector-info-control",
                "hyper-client-info-control",
                "hyper-pool-info-control",
                "tungstenite-client-info-control",
                "jsonrpsee-info-control",
                "rustls-info-control",
                "rustls-tls12-info-control",
                "rustls-nearby-target-control",
                "unrelated-trace-control",
                "tungstenite-trace-span-control",
                "tungstenite-trace-span-event-control",
                "rustls-debug-span-control",
                "rustls-debug-span-event-control",
                "other-alloy-span-field-control",
                "other-alloy-event-control",
            ] {
                assert!(output.contains(control), "missing {control:?} in {output}");
            }
        }
    }

    #[test]
    fn disabled_callsites_keep_interest_never() {
        let subscriber = subscriber(
            LogFormat::Pretty,
            BoxMakeWriter::new(Capture::default()),
            EnvFilter::new("info"),
        );

        assert!(
            tracing::Subscriber::register_callsite(&subscriber, &UNRELATED_DEBUG_METADATA)
                .is_never()
        );
    }

    #[test]
    fn global_subscriber_contract_is_enforced() {
        const CHILD_MODE: &str = "AGGLAYER_LOGGING_TEST_CHILD";

        match env::var(CHILD_MODE).as_deref() {
            Ok("preinstalled") => {
                tracing::subscriber::set_global_default(tracing_subscriber::registry()).unwrap();
                let error = tracing(&agglayer_config::Log::default()).unwrap_err();
                assert!(error
                    .to_string()
                    .contains("requires its endpoint-redaction policy"));
            }
            Ok("repeated") => {
                let config = agglayer_config::Log::default();
                tracing(&config).unwrap();
                tracing(&config).unwrap();
            }
            _ => {
                for mode in ["preinstalled", "repeated"] {
                    let status = Command::new(env::current_exe().unwrap())
                        .arg("--exact")
                        .arg("logging::tests::global_subscriber_contract_is_enforced")
                        .env(CHILD_MODE, mode)
                        .status()
                        .unwrap();
                    assert!(status.success(), "isolated {mode} logging test failed");
                }
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

        let Err(error) = BlockClock::new_with_ws(
            WsConnect::new(format!(
                "ws://{WS_USERNAME}:{WS_PASSWORD}@127.0.0.1:{port}/{WS_PATH_SECRET}?\
                 key={WS_QUERY_SECRET}"
            )),
            0,
            NonZeroU64::new(1).unwrap(),
            Duration::from_secs(1),
        )
        .await
        else {
            panic!("test server accepted WebSocket upgrade");
        };
        tracing::error!(
            target: "agglayer_node::node",
            "Failed to start BlockClock: {:?}",
            error
        );
        server.await.unwrap();

        // Rustls logs the SNI while producing ClientHello, before any peer response, so
        // a loopback TCP peer exercises the real TLS logging path without a
        // test certificate.
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut client_hello = [0_u8; 2048];
            let _ = stream.read(&mut client_hello).await.unwrap();
        });

        let client = reqwest13::Client::builder()
            .no_proxy()
            .resolve(TLS_HOST_SECRET, address)
            .build()
            .unwrap();
        let url = format!(
            "https://{TLS_HOST_SECRET}:{}/{HTTP_PATH_SECRET}?key={HTTP_QUERY_SECRET}",
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

    #[test_log::test(tokio::test(flavor = "current_thread"))]
    async fn actual_dependency_transports_do_not_log_endpoint_credentials() {
        // test-log may have already installed the process-wide bridge with a lower max
        // level.
        let _ = tracing_log::LogTracer::init();
        tracing_log::log::set_max_level(tracing_log::log::LevelFilter::Trace);

        let filter = || EnvFilter::new("trace");
        let baseline_observed_leaks = [
            HTTP_HOST_SECRET,
            TLS_HOST_SECRET,
            HTTP_PATH_SECRET,
            HTTP_QUERY_SECRET,
            WS_PATH_SECRET,
            WS_QUERY_SECRET,
            WS_BASIC_AUTH,
        ];
        // Tungstenite encodes userinfo in `WS_BASIC_AUTH`; the raw values remain
        // absence-only checks for a future dependency regression.
        let all_sensitive_endpoint_components = [
            HTTP_HOST_SECRET,
            TLS_HOST_SECRET,
            HTTP_PATH_SECRET,
            HTTP_QUERY_SECRET,
            WS_USERNAME,
            WS_PASSWORD,
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
            "rustls::client::hs",
        ];

        for format in [LogFormat::Pretty, LogFormat::Json] {
            let capture = Capture::default();
            let unfiltered = subscriber_without_endpoint_policy(
                format,
                BoxMakeWriter::new(capture.clone()),
                filter(),
            );
            let output = capture_actual_dependency_records(unfiltered, capture).await;
            for secret in baseline_observed_leaks {
                assert!(output.contains(secret), "missing {secret:?} in {output}");
            }
            for target in sensitive_targets {
                assert!(output.contains(target), "missing {target:?} in {output}");
            }
            assert!(
                output.contains(BLOCK_CLOCK_FAILURE),
                "missing {BLOCK_CLOCK_FAILURE:?} in {output}"
            );

            let capture = Capture::default();
            let filtered = subscriber(format, BoxMakeWriter::new(capture.clone()), filter());
            let output = capture_actual_dependency_records(filtered, capture).await;
            for secret in all_sensitive_endpoint_components {
                assert!(!output.contains(secret), "found {secret:?} in {output}");
            }
            assert!(
                output.contains(BLOCK_CLOCK_FAILURE),
                "missing {BLOCK_CLOCK_FAILURE:?} in {output}"
            );
        }
    }
}
