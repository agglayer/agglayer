use tracing::debug;

#[derive(Clone)]
pub struct L1TraceLayer;

impl<S> tower::Layer<S> for L1TraceLayer {
    type Service = L1TraceService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        L1TraceService { inner }
    }
}

#[derive(Clone)]
pub struct L1TraceService<S> {
    inner: S,
}

impl<S, Request> tower::Service<Request> for L1TraceService<S>
where
    S: tower::Service<Request> + Clone + Send + 'static,
    S::Future: Send + 'static,
    Request: std::fmt::Debug,
    S::Response: std::fmt::Debug,
    S::Error: std::fmt::Debug,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let span = tracing::debug_span!("L1 interaction");
        let _span_guard = span.enter();
        debug!(?req, "L1 request");

        let inner_fut = self.inner.call(req);
        Box::pin(async move {
            let start = std::time::Instant::now();
            let res = inner_fut.await;
            let elapsed_ms = start.elapsed().as_millis();
            match &res {
                Ok(response) => debug!(elapsed_ms, ?response, "L1 response"),
                Err(error) => debug!(elapsed_ms, ?error, "L1 error"),
            }
            res
        })
    }
}
