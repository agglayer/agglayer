use std::sync::Arc;
use std::time::Duration;

use agglayer_config::Config;
use ethers::providers::{self, Http, Middleware, Provider, ProviderExt as _};
use ethers::types::TransactionRequest;
use ethers::utils::Anvil;
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::{core::client::ClientT, rpc_params};

use crate::rpc::TxStatus;
use crate::{kernel::Kernel, rpc::AgglayerImpl};

#[tokio::test]
async fn healthcheck_method_can_be_called() {
    use hyper::{Body, Client, Request};

    let _ = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    let config = Arc::new(Config::default());
    let (provider, _mock) = providers::Provider::mocked();

    let kernel = Kernel::new(provider, config.clone());

    let _server_handle = AgglayerImpl::new(kernel)
        .start(config.clone())
        .await
        .unwrap();

    let http_client = Client::new();
    let uri = format!("http://{}/health", config.rpc_addr());

    let req = Request::builder()
        .method("GET")
        .uri(&uri)
        .body(Body::empty())
        .expect("request builder");
    let res = http_client.request(req).await.unwrap();

    assert!(res.status().is_success());

    let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let out = String::from_utf8(bytes.to_vec()).unwrap();
    assert_eq!(out.as_str(), "{\"health\":true}");
}

#[tokio::test]
async fn check_tx_status() {
    let _ = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    let anvil = Anvil::new().block_time(1u64).spawn();
    let client = Provider::<Http>::connect(&anvil.endpoint()).await;
    let accounts = client.get_accounts().await.unwrap();
    let from = accounts[0];
    let to = accounts[1];

    let tx = TransactionRequest::new().to(to).value(1000).from(from);

    let hash = client
        .send_transaction(tx, None)
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap()
        .transaction_hash;

    let mut config = Config::default();
    let addr = next_available_addr();
    if let std::net::IpAddr::V4(ip) = addr.ip() {
        config.rpc.host = ip;
    }
    config.rpc.port = addr.port();

    let config = Arc::new(config);

    let kernel = Kernel::new(client, config.clone());

    let _server_handle = AgglayerImpl::new(kernel)
        .start(config.clone())
        .await
        .unwrap();

    let url = format!("http://{}/", config.rpc_addr());
    let client = HttpClientBuilder::default().build(url).unwrap();

    let res: TxStatus = client
        .request("interop_getTxStatus", rpc_params![hash])
        .await
        .unwrap();

    // The transaction is not yet mined, so we should get a pending status
    assert_eq!(res.status, "pending");

    tokio::time::sleep(Duration::from_secs(1)).await;

    let res: TxStatus = client
        .request("interop_getTxStatus", rpc_params![hash])
        .await
        .unwrap();

    assert_eq!(res.status, "done");
}

fn next_available_addr() -> std::net::SocketAddr {
    use std::net::{TcpListener, TcpStream};

    let host = "127.0.0.1";
    // Request a random available port from the OS
    let listener = TcpListener::bind((host, 0)).expect("Can't bind to an available port");
    let addr = listener.local_addr().expect("Can't find an available port");

    // Create and accept a connection (which we'll promptly drop) in order to force
    // the port into the TIME_WAIT state, ensuring that the port will be
    // reserved from some limited amount of time (roughly 60s on some Linux
    // systems)
    let _sender = TcpStream::connect(addr).expect("Can't connect to an available port");
    let _incoming = listener.accept().expect("Can't accept an available port");

    addr
}
