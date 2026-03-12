use alloy::transports::{RpcError, TransportError, TransportErrorKind};

type Request = alloy::rpc::json_rpc::RequestPacket;
type Response = alloy::rpc::json_rpc::ResponsePacket;

/// Tower layer that strips the URL from [`reqwest::Error`]s embedded inside
/// [`TransportError`], preventing RPC endpoint paths (which may contain API
/// tokens) from leaking into error messages or logs.
#[derive(Clone, Copy)]
pub struct UrlRedactLayer;

impl<S> tower::Layer<S> for UrlRedactLayer {
    type Service = UrlRedactService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        UrlRedactService { inner }
    }
}

#[derive(Clone)]
pub struct UrlRedactService<S> {
    inner: S,
}

impl<S> tower::Service<Request> for UrlRedactService<S>
where
    S: tower::Service<Request, Response = Response, Error = TransportError>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = TransportError;
    type Future = futures::future::BoxFuture<'static, Result<Response, TransportError>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let fut = self.inner.call(request);
        Box::pin(async move { fut.await.map_err(strip_reqwest_url) })
    }
}

/// If the [`TransportError`] wraps a [`reqwest::Error`], call
/// [`reqwest::Error::without_url`] so that the RPC endpoint URL (which may
/// contain secret API tokens in the path) is never included in `Display` or
/// `Debug` output.
fn strip_reqwest_url(err: TransportError) -> TransportError {
    let RpcError::Transport(TransportErrorKind::Custom(boxed)) = err else {
        return err;
    };
    match boxed.downcast::<reqwest::Error>() {
        Ok(re) => TransportErrorKind::custom(re.without_url()),
        Err(other) => RpcError::Transport(TransportErrorKind::Custom(other)),
    }
}

#[cfg(test)]
mod tests {
    use alloy::providers::{Provider, ProviderBuilder};

    use super::*;

    /// A URL whose path component contains a fake secret token.
    const SECRET_URL: &str = "http://127.0.0.1:1/v1/my-secret-token";
    const SECRET_PATH: &str = "my-secret-token";

    /// Fire an `eth_blockNumber` request that will immediately fail (nothing
    /// listens on 127.0.0.1:1) and return the error as a string.
    async fn get_error(with_redact_layer: bool) -> String {
        let url: reqwest::Url = SECRET_URL.parse().unwrap();

        let client = if with_redact_layer {
            alloy::rpc::client::RpcClient::builder()
                .layer(UrlRedactLayer)
                .http(url)
        } else {
            alloy::rpc::client::RpcClient::builder().http(url)
        };

        let provider = ProviderBuilder::new().connect_client(client);
        let err = provider.get_block_number().await.unwrap_err();
        format!("{err}")
    }

    #[tokio::test]
    async fn url_present_without_layer() {
        let msg = get_error(false).await;
        assert!(
            msg.contains(SECRET_PATH),
            "expected the secret token in the error without the redact layer, got: {msg}"
        );
    }

    #[tokio::test]
    async fn url_absent_with_layer() {
        let msg = get_error(true).await;
        assert!(
            !msg.contains(SECRET_PATH),
            "secret token must not appear in the error with the redact layer, got: {msg}"
        );
    }
}
