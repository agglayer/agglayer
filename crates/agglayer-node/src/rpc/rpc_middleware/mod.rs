//! RPC middleware layers.

use jsonrpsee::server::middleware::rpc::RpcServiceBuilder;
use tower::layer::util::{Identity, Stack};

mod logging_timeout;

pub use logging_timeout::LoggingTimeoutLayer;

/// The stack of RPC middleware layers.
pub type RpcStack = Stack<LoggingTimeoutLayer, Identity>;

/// Build the RPC middleware stack from config.
pub fn build(config: &agglayer_config::Config) -> RpcServiceBuilder<RpcStack> {
    jsonrpsee::server::middleware::rpc::RpcServiceBuilder::new()
        .layer(LoggingTimeoutLayer::new(config.rpc.request_timeout))
}
