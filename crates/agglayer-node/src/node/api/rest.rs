use hyper::StatusCode;

pub(crate) fn health_router() -> axum::Router {
    axum::Router::new().route("/health", axum::routing::get(health))
}

pub(crate) async fn health() -> impl axum::response::IntoResponse {
    (
        StatusCode::OK,
        serde_json::json!({ "health": true }).to_string(),
    )
}
