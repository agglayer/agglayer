use hyper::StatusCode;

pub(crate) fn health_router(version: String) -> axum::Router {
    axum::Router::new().route(
        "/health",
        axum::routing::get(move || health(version.clone())),
    )
}

pub(crate) async fn health(version: String) -> impl axum::response::IntoResponse {
    (
        StatusCode::OK,
        serde_json::json!({ "health": true, "version": version }).to_string(),
    )
}
