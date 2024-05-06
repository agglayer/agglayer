use std::sync::Arc;

use agglayer_config::Config;
use ethers::providers;

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
