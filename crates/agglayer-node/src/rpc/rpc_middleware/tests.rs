use std::{future::Future, time::Duration};

use jsonrpsee::{
    core::{async_trait, client::ClientT, ClientError},
    http_client::HttpClient,
    proc_macros::rpc,
    server::{
        middleware::rpc::{RpcService, RpcServiceT},
        RpcServiceBuilder,
    },
};
use tracing_subscriber::layer::SubscriberExt;

use super::LoggingTimeoutLayer;

#[rpc(server)]
trait Test {
    #[method(name = "do_stuff")]
    async fn do_stuff(&self) -> &'static str;
}

/// Test RPC server. Requests take given duration.
struct TestRpc {
    stuff_duration: Duration,
}

#[async_trait]
impl TestServer for TestRpc {
    async fn do_stuff(&self) -> &'static str {
        tokio::time::sleep(self.stuff_duration).await;
        "stuff done"
    }
}

impl TestRpc {
    async fn start<T>(
        stuff_duration: Duration,
        middleware: RpcServiceBuilder<T>,
    ) -> (ServerGuard, ClientHandle)
    where
        T: tower::Layer<RpcService> + Clone + Send + 'static,
        T::Service: for<'a> RpcServiceT<'a> + Send + Sync + 'static,
    {
        let server = jsonrpsee::server::Server::builder()
            .set_rpc_middleware(middleware)
            .build("127.0.0.1:0")
            .await
            .unwrap();
        let url = format!("http://{}", server.local_addr().unwrap());
        let server = ServerGuard(server.start((TestRpc { stuff_duration }).into_rpc()).into());
        let client = ClientHandle(HttpClient::builder().build(&url).unwrap());

        (server, client)
    }
}

/// A slightly more convenient way to `do_stuff`.
struct ClientHandle(HttpClient);

impl ClientHandle {
    async fn do_stuff(&self) -> Result<String, ClientError> {
        self.0.request("do_stuff", [(); 0]).await
    }
}

/// Just a guard that stops the server on drop.
struct ServerGuard(Option<jsonrpsee::server::ServerHandle>);

impl Drop for ServerGuard {
    fn drop(&mut self) {
        self.0.take().map(|s| s.stop().unwrap());
    }
}

async fn capture_log<R>(proc: impl Future<Output = R>) -> (tracing_capture::SharedStorage, R) {
    let storage = tracing_capture::SharedStorage::default();
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_ansi(false)
        .compact()
        .finish()
        .with(tracing_capture::CaptureLayer::new(&storage));
    let _guard = tracing::subscriber::set_default(subscriber);
    let ret = proc.await;
    (storage, ret)
}

fn log_contains(log: &tracing_capture::SharedStorage, needle: &str) -> bool {
    log.lock()
        .all_events()
        .any(|e| e.message().is_some_and(|m| m.contains(needle)))
}

const TIMED_OUT_STR: &'static str = "`do_stuff` timed out";
const CANCELLED_STR: &'static str = "`do_stuff` was cancelled";

#[tokio::test]
async fn completed_before_deadline() {
    tokio::time::pause();

    let (log, res) = capture_log(async {
        let middleware = super::build(Duration::from_secs(45));
        let (_server, client) = TestRpc::start(Duration::from_secs(30), middleware).await;

        let test_task = tokio::spawn(async move { client.do_stuff().await });
        tokio::time::advance(Duration::from_secs(32)).await;
        test_task.await.unwrap()
    })
    .await;

    assert_eq!(res.unwrap(), "stuff done");
    assert!(!log_contains(&log, TIMED_OUT_STR));
    assert!(!log_contains(&log, CANCELLED_STR));
}

#[tokio::test]
async fn timed_out() {
    tokio::time::pause();

    let (log, res) = capture_log(async {
        let middleware = super::build(Duration::from_secs(20));
        let (_server, client) = TestRpc::start(Duration::from_secs(30), middleware).await;

        let test_task = tokio::spawn(async move { client.do_stuff().await });
        tokio::time::advance(Duration::from_secs(22)).await;

        test_task.await.unwrap()
    })
    .await;

    match res.unwrap_err() {
        ClientError::Call(err) => {
            assert_eq!(err.code(), LoggingTimeoutLayer::ERROR_CODE);
            assert_eq!(err.message(), "request timed out");
            assert_eq!(
                serde_json::to_value(err.data()).unwrap(),
                serde_json::json!({ "timeout": 20 }),
            );
        }
        _ => panic!("Unexpected error kind"),
    }

    assert!(log_contains(&log, TIMED_OUT_STR));
    assert!(!log_contains(&log, CANCELLED_STR));
}

#[tokio::test]
async fn request_dropped() {
    tokio::time::pause();

    let (log, res) = capture_log(async {
        let middleware = super::build(Duration::from_secs(45));
        let (_server, client) = TestRpc::start(Duration::from_secs(30), middleware).await;

        let test_task = tokio::spawn(async move {
            tokio::time::timeout(Duration::from_secs(10), client.do_stuff()).await
        });

        tokio::time::advance(Duration::from_secs(11)).await;
        let res = test_task.await.unwrap();
        tokio::time::advance(Duration::from_secs(1)).await;

        res
    })
    .await;

    // On the client side, the result should be a timeout
    assert!(res.is_err());

    // On the server side, the request cancellation should have been logged
    assert!(!log_contains(&log, TIMED_OUT_STR));
    assert!(log_contains(&log, CANCELLED_STR));
}
