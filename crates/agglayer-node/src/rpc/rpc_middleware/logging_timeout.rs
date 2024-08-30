//! A logging timeout layer.

use std::{borrow::Cow, future::Future, time::Duration};

use futures::TryFutureExt;
use jsonrpsee::{
    server::middleware::rpc::RpcServiceT,
    types::{ErrorObject, Id, Request},
    MethodResponse,
};
use tracing::warn;

/// Error code to return when the response time is too long.
pub const TIMEOUT_ERROR_CODE: i32 = -32001;

/// A layer that applies a timeout on a request and issues a log entry if the
/// timeout expires before the request is completed.
#[derive(Clone, Debug)]
pub struct LoggingTimeoutLayer {
    timeout: Duration,
}

impl LoggingTimeoutLayer {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl<S> tower::Layer<S> for LoggingTimeoutLayer {
    type Service = LoggingTimeoutService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggingTimeoutService::new(self.timeout, inner)
    }
}

pub struct LoggingTimeoutService<S> {
    inner: S,
    timeout: Duration,
}

impl<S> LoggingTimeoutService<S> {
    pub fn new(timeout: Duration, inner: S) -> Self {
        Self { inner, timeout }
    }
}

impl<'a, S: RpcServiceT<'a>> RpcServiceT<'a> for LoggingTimeoutService<S> {
    type Future = LoggingTimeoutFuture<'a, S::Future>;

    fn call(&self, request: Request<'a>) -> Self::Future {
        LoggingTimeoutFuture {
            timeout: self.timeout,
            method: request.method.clone(),
            request_id: request.id.clone(),
            inner: self.inner.call(request),
        }
    }
}

#[pin_project::pin_project]
pub struct LoggingTimeoutFuture<'a, F> {
    // Timeout duration.
    timeout: Duration,

    // Method information.
    method: Cow<'a, str>,
    request_id: Id<'a>,

    // The future to execute under a timeout.
    #[pin]
    inner: F,
}

impl<F: Future<Output = MethodResponse>> Future for LoggingTimeoutFuture<'_, F> {
    type Output = MethodResponse;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        let timeout = *this.timeout;

        let fut = tokio::time::timeout(timeout, this.inner).unwrap_or_else(move |e| {
            let method = &**this.method;
            let id = &*this.request_id;
            warn!("Request ID `{id}` to `{method}` timed out: {e}");

            let info = serde_json::json!({ "timeout": timeout.as_secs() });
            let err = ErrorObject::owned(TIMEOUT_ERROR_CODE, "request timed out", Some(info));
            MethodResponse::error(id.to_owned(), err)
        });

        std::pin::pin!(fut).as_mut().poll(cx)
    }
}
