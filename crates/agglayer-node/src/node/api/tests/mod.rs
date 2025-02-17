use std::future::IntoFuture;

use agglayer_jsonrpc_api::tests::TestContext;
use http::Request;
use http_body_util::Empty;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

use crate::node::api;

#[test_log::test(tokio::test)]
async fn healthcheck_method_can_be_called() {
    let router = api::rest::health_router();
    let addr = TestContext::next_available_address();
    let listener = TcpListener::bind(addr).await.unwrap();

    let token = CancellationToken::new();
    tokio::spawn(
        axum::serve(listener, router)
            .with_graceful_shutdown(token.clone().cancelled_owned())
            .into_future(),
    );

    let http_client = Client::builder(TokioExecutor::new()).build_http();
    let uri = format!("http://{}/health", addr);

    let req = Request::builder()
        .method("GET")
        .uri(&uri)
        .body(Empty::<hyper::body::Bytes>::new())
        .expect("request builder");
    let res = http_client.request(req).await.unwrap();

    assert!(res.status().is_success());

    let bytes = http_body_util::BodyExt::collect(res.into_body())
        .await
        .unwrap();
    let out = String::from_utf8(bytes.to_bytes().to_vec()).unwrap();
    assert_eq!(out.as_str(), "{\"health\":true}");
    token.cancel();
}
