use tracing::debug;

type Request = alloy::rpc::json_rpc::RequestPacket;
type Response = alloy::rpc::json_rpc::ResponsePacket;

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

impl<S> tower::Service<Request> for L1TraceService<S>
where
    S: tower::Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: std::fmt::Debug,
{
    type Response = Response;
    type Error = S::Error;
    type Future = futures::future::BoxFuture<'static, Result<Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let span = tracing::debug_span!("L1 interaction");
        let _span_guard = span.enter();
        match &request {
            Request::Single(request) => debug!(?request, "L1 request"),
            Request::Batch(request) => debug!(?request, "L1 batch request"),
        }

        let inner_fut = self.inner.call(request);
        Box::pin(async move {
            let start = tokio::time::Instant::now();
            let res = inner_fut.await;
            let elapsed_ms = start.elapsed().as_millis();
            match &res {
                Ok(response) => match response {
                    Response::Single(response) => debug!(elapsed_ms, ?response, "L1 response"),
                    Response::Batch(response) => debug!(elapsed_ms, ?response, "L1 batch response"),
                },
                Err(error) => debug!(elapsed_ms, ?error, "L1 interaction error"),
            }
            res
        })
    }
}
