//! A logging timeout layer.

use std::{future::Future, time::Duration};

use futures::TryFutureExt;
use jsonrpsee::{server::middleware::rpc::RpcServiceT, types::ErrorObject, MethodResponse};

use super::RequestInfo;

/// A layer that applies a timeout on a request and issues a log entry if the
/// timeout expires before the request is completed.
#[derive(Clone, Debug)]
pub struct LoggingTimeoutLayer {
    /// Maximum duration for the request to complete.
    timeout: Duration,
}

impl LoggingTimeoutLayer {
    /// Error code to return when the response time is too long.
    pub const ERROR_CODE: i32 = -32001;

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
    /// The underlying service.
    inner: S,

    /// Maximum duration for the request to complete.
    timeout: Duration,
}

impl<S> LoggingTimeoutService<S> {
    pub fn new(timeout: Duration, inner: S) -> Self {
        Self { inner, timeout }
    }
}

impl<'a, S: RpcServiceT<'a>> RpcServiceT<'a> for LoggingTimeoutService<S> {
    type Future = LoggingTimeoutFuture<'a, S::Future>;

    fn call(&self, request: jsonrpsee::types::Request<'a>) -> Self::Future {
        LoggingTimeoutFuture {
            timeout: self.timeout,
            request_info: RequestInfo::from_request(&request),
            inner: tokio::time::timeout(self.timeout, self.inner.call(request)),
        }
    }
}

#[pin_project::pin_project]
pub struct LoggingTimeoutFuture<'a, F> {
    /// The timeout duration.
    timeout: Duration,

    /// Information about the request and the method.
    request_info: RequestInfo<'a>,

    /// The future to be executed under the timeout.
    #[pin]
    inner: tokio::time::Timeout<F>,
}

impl<F: Future<Output = MethodResponse>> Future for LoggingTimeoutFuture<'_, F> {
    type Output = MethodResponse;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();

        let fut = this.inner.unwrap_or_else(move |e| {
            let method = &*this.request_info.method;
            let id = &this.request_info.request_id;
            tracing::warn!("Request ID `{id}` to `{method}` timed out: {e}");

            let info = serde_json::json!({ "timeout": this.timeout.as_secs() });
            let error_code = LoggingTimeoutLayer::ERROR_CODE;
            let err = ErrorObject::owned(error_code, "request timed out", Some(info));
            MethodResponse::error(id.to_owned(), err)
        });

        std::pin::pin!(fut).as_mut().poll(cx)
    }
}
