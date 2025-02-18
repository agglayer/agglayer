use hyper::StatusCode;

pub(crate) async fn health() -> impl axum::response::IntoResponse {
    (
        StatusCode::OK,
        serde_json::json!({ "health": true }).to_string(),
    )
}
