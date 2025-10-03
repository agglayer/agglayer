use std::sync::atomic::Ordering;

use axum::extract::State;
use hyper::StatusCode;

use crate::node::ReadinessState;

pub(crate) fn health_router() -> axum::Router {
    axum::Router::new().route("/health", axum::routing::get(health))
}

pub(crate) async fn health() -> impl axum::response::IntoResponse {
    (
        StatusCode::OK,
        serde_json::json!({ "health": true }).to_string(),
    )
}

pub(crate) fn readiness_router(state: ReadinessState) -> axum::Router {
    axum::Router::new()
        .route("/ready", axum::routing::get(readiness))
        .with_state(state)
}

pub(crate) async fn readiness(state: State<ReadinessState>) -> impl axum::response::IntoResponse {
    (
        StatusCode::OK,
        serde_json::json!({
            "rpc": state.rpc.load(Ordering::Relaxed),
        })
        .to_string(),
    )
}
