use std::sync::Arc;

use agglayer_config::Config;
use agglayer_telemetry::KeyValue;
use ethers::{providers::Middleware, types::H256};
use futures::TryFutureExt;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
    server::{middleware::http::ProxyGetRequestLayer, PingConfig, ServerBuilder, ServerHandle},
    types::{
        error::{
            CALL_EXECUTION_FAILED_CODE, INTERNAL_ERROR_CODE, INTERNAL_ERROR_MSG,
            INVALID_PARAMS_CODE, INVALID_PARAMS_MSG,
        },
        ErrorObject, ErrorObjectOwned,
    },
};
use pessimistic_proof::certificate::Certificate;
use tokio::{sync::mpsc, try_join};
use tower_http::cors::CorsLayer;
use tracing::{debug, error, info, instrument};

use crate::{
    kernel::{Kernel, ZkevmNodeVerificationError},
    signed_tx::SignedTx,
};

#[cfg(test)]
mod tests;

#[rpc(server, namespace = "interop")]
trait Agglayer {
    #[method(name = "sendTx")]
    async fn send_tx(&self, tx: SignedTx) -> RpcResult<H256>;

    #[method(name = "getTxStatus")]
    async fn get_tx_status(&self, hash: H256) -> RpcResult<TxStatus>;

    #[method(name = "sendCertificate")]
    async fn send_certificate(&self, certificate: Certificate) -> RpcResult<()>;
}

/// The RPC agglayer service implementation.
pub(crate) struct AgglayerImpl<Rpc> {
    kernel: Kernel<Rpc>,
    certificate_sender: mpsc::Sender<Certificate>,
}

impl<Rpc> AgglayerImpl<Rpc> {
    /// Create an instance of the RPC agglayer service.
    pub(crate) fn new(kernel: Kernel<Rpc>, certificate_sender: mpsc::Sender<Certificate>) -> Self {
        Self {
            kernel,
            certificate_sender,
        }
    }
}
impl<Rpc> AgglayerImpl<Rpc>
where
    Rpc: Middleware + 'static,
{
    pub(crate) async fn start(self, config: Arc<Config>) -> anyhow::Result<ServerHandle> {
        // Create the RPC service
        let mut service = self.into_rpc();

        // Register the system_health method to serve health checks.
        service.register_method(
            "system_health",
            |_, _, _| serde_json::json!({ "health": true }),
        )?;

        // Create the RPC server.
        let mut server_builder = ServerBuilder::new()
            // Set the maximum request body size. The default is 10MB.
            .max_request_body_size(config.rpc.max_request_body_size)
            // Set the maximum response body size. The default is 10MB.
            .max_response_body_size(config.rpc.max_response_body_size)
            // Set the maximum number of connections. The default is 100.
            .max_connections(config.rpc.max_connections)
            // Set the batch request limit. The default is unlimited.
            .set_batch_request_config(match config.rpc.batch_request_limit {
                None => jsonrpsee::server::BatchRequestConfig::Unlimited,
                Some(0) => jsonrpsee::server::BatchRequestConfig::Disabled,
                Some(n) => jsonrpsee::server::BatchRequestConfig::Limit(n),
            });

        // Enable WebSocket ping/pong with the configured interval.
        // By default, pings are disabled.
        if let Some(duration) = config.rpc.ping_interval {
            server_builder =
                server_builder.enable_ws_ping(PingConfig::default().ping_interval(duration));
        }

        // Create a CORS middleware to allow cross-origin requests.
        let cors = CorsLayer::new()
            .allow_methods([
                hyper::Method::POST,
                hyper::Method::GET,
                hyper::Method::OPTIONS,
            ])
            .allow_origin(tower_http::cors::Any)
            .allow_headers([hyper::header::CONTENT_TYPE]);

        // Create a middleware stack with the CORS middleware and a proxy layer for
        // health checks.
        let middleware = tower::ServiceBuilder::new()
            .layer(ProxyGetRequestLayer::new("/health", "system_health")?)
            .layer(cors);

        let addr = config.rpc_addr();

        let server = server_builder
            .set_http_middleware(middleware)
            .build(addr)
            .await?;

        info!("Listening on {addr}");

        Ok(server.start(service))
    }
}

/// Helper function to create an invalid params error with a custom message.
fn invalid_params_error(msg: impl Into<String>) -> ErrorObjectOwned {
    ErrorObject::owned(INVALID_PARAMS_CODE, INVALID_PARAMS_MSG, Some(msg.into()))
}

/// Helper function to create an internal error with a custom message.
fn internal_error(msg: impl Into<String>) -> ErrorObjectOwned {
    ErrorObject::owned(INTERNAL_ERROR_CODE, INTERNAL_ERROR_MSG, Some(msg.into()))
}

#[async_trait]
impl<Rpc> AgglayerServer for AgglayerImpl<Rpc>
where
    Rpc: Middleware + 'static,
{
    #[instrument(skip(self, tx), fields(hash = tx.hash().to_string(), rollup_id = tx.tx.rollup_id), level = "debug")]
    async fn send_tx(&self, tx: SignedTx) -> RpcResult<H256> {
        let tx_hash = tx.hash().to_string();
        debug!(
            "Received transaction {tx_hash} for rollup {}",
            tx.tx.rollup_id
        );
        let rollup_id_str = tx.tx.rollup_id.to_string();
        let metrics_attrs = &[KeyValue::new("rollup_id", rollup_id_str)];

        agglayer_telemetry::SEND_TX.add(1, metrics_attrs);

        if !self.kernel.check_rollup_registered(tx.tx.rollup_id) {
            // Return an invalid params error if the rollup is not registered.
            return Err(invalid_params_error(
                ZkevmNodeVerificationError::InvalidRollupId(tx.tx.rollup_id).to_string(),
            ));
        }

        agglayer_telemetry::CHECK_TX.add(1, metrics_attrs);

        // Run all the verification checks in parallel.
        try_join!(
            self.kernel
                .verify_signature(&tx)
                .map_err(|e| {
                    error!(
                        tx_hash,
                        "Failed to verify the signature of transaction {tx_hash}: {e}"
                    );
                    invalid_params_error(e.to_string())
                })
                .map_ok(|_| {
                    agglayer_telemetry::VERIFY_SIGNATURE.add(1, metrics_attrs);
                }),
            self.kernel
                .verify_proof_eth_call(&tx)
                .map_err(|e| {
                    error!(
                        tx_hash,
                        "Failed to dry-run the verify_batches_trusted_aggregator for transaction \
                         {tx_hash}: {e}"
                    );
                    invalid_params_error(e.to_string())
                })
                .map_ok(|_| {
                    agglayer_telemetry::EXECUTE.add(1, metrics_attrs);
                }),
            self.kernel
                .verify_proof_zkevm_node(&tx)
                .map_err(|e| {
                    error!(
                        tx_hash,
                        "Failed to verify the batch local_exit_root and state_root of transaction \
                         {tx_hash}: {e}"
                    );
                    invalid_params_error(e.to_string())
                })
                .map_ok(|_| {
                    agglayer_telemetry::VERIFY_ZKP.add(1, metrics_attrs);
                })
        )?;

        // Settle the proof on-chain and return the transaction hash.
        let receipt = self.kernel.settle(&tx).await.map_err(|e| {
            error!(tx_hash, "Failed to settle transaction {tx_hash} on L1: {e}");
            internal_error(e.to_string())
        })?;

        agglayer_telemetry::SETTLE.add(1, metrics_attrs);

        info!("Successfully settled transaction {tx_hash} => receipt {receipt:?}");

        Ok(receipt.transaction_hash)
    }

    #[instrument(skip(self), fields(hash = hash.to_string()), level = "debug")]
    async fn get_tx_status(&self, hash: H256) -> RpcResult<TxStatus> {
        debug!("Received request to get transaction status for hash {hash}");
        let recipt = self.kernel.check_tx_status(hash).await.map_err(|e| {
            error!("Failed to get transaction status for hash {hash}: {e}");

            ErrorObject::owned(
                CALL_EXECUTION_FAILED_CODE,
                INTERNAL_ERROR_MSG,
                Some(format!("failed to get tx, error: {}", e)),
            )
        })?;

        let current_block = self.kernel.current_l1_block_height().await.map_err(|e| {
            error!("Failed to get current L1 block: {e}");

            ErrorObject::owned(
                CALL_EXECUTION_FAILED_CODE,
                INTERNAL_ERROR_MSG,
                Some(format!("failed to get current L1 block, error: {}", e)),
            )
        })?;

        recipt
            .map(|recipt| match recipt.block_number {
                Some(block_number) if block_number < current_block => "done".to_string(),
                Some(_) => "pending".to_string(),
                None => "not found".to_string(),
            })
            .ok_or_else(|| {
                ErrorObject::owned(
                    CALL_EXECUTION_FAILED_CODE,
                    INTERNAL_ERROR_MSG,
                    Some(format!("tx not found for hash: {}", hash)),
                )
            })
    }

    async fn send_certificate(&self, certificate: Certificate) -> RpcResult<()> {
        if let Err(error) = self.certificate_sender.send(certificate).await {
            error!("Failed to send certificate: {error}");

            return Err(internal_error("Unable to send certificate to collector"));
        }

        Ok(())
    }
}

type TxStatus = String;
