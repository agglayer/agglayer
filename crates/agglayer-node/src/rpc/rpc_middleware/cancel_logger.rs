//! RPC middleware for recording cancelled requests.

use std::future::Future;

use jsonrpsee::{server::middleware::rpc::RpcServiceT, types::Request, MethodResponse};

use super::RequestInfo;

/// An RPC layer that logs request cancellations.
#[derive(Clone, Debug)]
pub struct CancelLoggerLayer {}

impl CancelLoggerLayer {
    pub fn new() -> Self {
        CancelLoggerLayer {}
    }
}

impl<S> tower::Layer<S> for CancelLoggerLayer {
    type Service = CancelLoggerService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CancelLoggerService(inner)
    }
}

pub struct CancelLoggerService<S>(S);

impl<'a, S: RpcServiceT<'a>> RpcServiceT<'a> for CancelLoggerService<S> {
    type Future = CancelLoggerFuture<'a, S::Future>;

    fn call(&self, request: Request<'a>) -> Self::Future {
        CancelLoggerFuture {
            completed: false,
            request_info: RequestInfo::from_request(&request),
            inner: self.0.call(request),
        }
    }
}

#[pin_project::pin_project(PinnedDrop)]
pub struct CancelLoggerFuture<'a, F> {
    /// The future completion state.
    completed: bool,

    /// Request and method information.
    request_info: RequestInfo<'a>,

    /// The future to log cancellation for.
    #[pin]
    inner: F,
}

impl<F: Future<Output = MethodResponse>> Future for CancelLoggerFuture<'_, F> {
    type Output = MethodResponse;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        let poll_result = this.inner.poll(cx);
        if poll_result.is_ready() {
            *this.completed = true;
        }
        poll_result
    }
}

#[pin_project::pinned_drop]
impl<F> PinnedDrop for CancelLoggerFuture<'_, F> {
    fn drop(self: std::pin::Pin<&mut Self>) {
        if !self.completed {
            let method = &*self.request_info.method;
            let id = &self.request_info.request_id;
            tracing::info!("Request ID `{id}` to `{method}` was cancelled");
        }
    }
}
