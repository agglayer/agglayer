//! RPC middleware for recording cancelled requests.

use std::{borrow::Cow, future::Future};

use jsonrpsee::{
    server::middleware::rpc::RpcServiceT,
    types::{Id, Request},
    MethodResponse,
};

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
            method: request.method.clone(),
            request_id: request.id.clone(),
            inner: self.0.call(request),
        }
    }
}

#[pin_project::pin_project(PinnedDrop)]
pub struct CancelLoggerFuture<'a, F> {
    // Future state.
    completed: bool,

    // Method information.
    method: Cow<'a, str>,
    request_id: Id<'a>,

    // The future to execute under a timeout.
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
            let method = &*self.method;
            let id = &self.request_id;
            tracing::warn!("Request ID `{id}` to `{method}` was cancelled");
        }
    }
}
