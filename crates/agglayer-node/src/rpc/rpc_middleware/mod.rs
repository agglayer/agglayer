//! RPC middleware layers.

use jsonrpsee::{server::middleware::rpc::RpcServiceBuilder, types::Id};
use tower::layer::util::{Identity, Stack};

mod cancel_logger;
mod logging_timeout;

pub use cancel_logger::CancelLoggerLayer;
pub use logging_timeout::LoggingTimeoutLayer;

/// Information about the method being executed.
struct RequestInfo<'a> {
    method: std::borrow::Cow<'a, str>,
    request_id: Id<'a>,
}

impl<'a> RequestInfo<'a> {
    fn from_request(request: &jsonrpsee::types::Request<'a>) -> Self {
        Self {
            method: request.method.clone(),
            request_id: request.id.clone(),
        }
    }
}

/// The stack of RPC middleware layers.
pub type RpcStack = Stack<CancelLoggerLayer, Stack<LoggingTimeoutLayer, Identity>>;

/// Build the RPC middleware stack from config.
pub fn build(config: &agglayer_config::Config) -> RpcServiceBuilder<RpcStack> {
    jsonrpsee::server::middleware::rpc::RpcServiceBuilder::new()
        .layer(LoggingTimeoutLayer::new(config.rpc.request_timeout))
        .layer(CancelLoggerLayer::new())
}
