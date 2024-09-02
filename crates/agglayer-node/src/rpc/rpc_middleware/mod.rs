//! RPC middleware layers.

use jsonrpsee::server::middleware::rpc::RpcServiceBuilder;
use tower::layer::util::{Identity, Stack};

mod cancel_logger;
mod logging_timeout;

pub use cancel_logger::CancelLoggerLayer;
pub use logging_timeout::LoggingTimeoutLayer;

/// The stack of RPC middleware layers.
pub type RpcStack = Stack<CancelLoggerLayer, Stack<LoggingTimeoutLayer, Identity>>;

/// Build the RPC middleware stack from config.
pub fn build(config: &agglayer_config::Config) -> RpcServiceBuilder<RpcStack> {
    jsonrpsee::server::middleware::rpc::RpcServiceBuilder::new()
        .layer(LoggingTimeoutLayer::new(config.rpc.request_timeout))
        .layer(CancelLoggerLayer::new())
}
