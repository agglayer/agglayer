use std::sync::Arc;

use agglayer_config::Config;
use agglayer_telemetry::KeyValue;
use ethers::{
    contract::{ContractError, ContractRevert},
    providers::Middleware,
    types::H256,
};
use futures::TryFutureExt;
use jsonrpsee::{
    core::async_trait,
    proc_macros::rpc,
    server::{middleware::http::ProxyGetRequestLayer, PingConfig, ServerBuilder, ServerHandle},
};
use pessimistic_proof::local_exit_tree::hasher::Keccak256Hasher;
use pessimistic_proof::multi_batch_header::MultiBatchHeader;
use tokio::{sync::mpsc, try_join};
use tower_http::cors::CorsLayer;
use tracing::{debug, error, info, instrument};

use crate::{
    kernel::Kernel,
    rpc::error::{Error, RpcResult, StatusError},
    signed_tx::SignedTx,
};

mod error;

#[cfg(test)]
mod tests;

#[rpc(server, namespace = "interop")]
trait Agglayer {
    #[method(name = "sendTx")]
    async fn send_tx(&self, tx: SignedTx) -> RpcResult<H256>;

    #[method(name = "getTxStatus")]
    async fn get_tx_status(&self, hash: H256) -> RpcResult<TxStatus>;

    #[method(name = "sendCertificate")]
    async fn send_certificate(
        &self,
        certificate: MultiBatchHeader<Keccak256Hasher>,
    ) -> RpcResult<()>;
}

/// The RPC agglayer service implementation.
pub(crate) struct AgglayerImpl<Rpc> {
    kernel: Kernel<Rpc>,
    certificate_sender: mpsc::Sender<MultiBatchHeader<Keccak256Hasher>>,
}

impl<Rpc> AgglayerImpl<Rpc> {
    /// Create an instance of the RPC agglayer service.
    pub(crate) fn new(
        kernel: Kernel<Rpc>,
        certificate_sender: mpsc::Sender<MultiBatchHeader<Keccak256Hasher>>,
    ) -> Self {
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

#[async_trait]
impl<Rpc> AgglayerServer for AgglayerImpl<Rpc>
where
    Rpc: Middleware + 'static,
{
    #[instrument(skip(self, tx), fields(hash, rollup_id = tx.tx.rollup_id), level = "info")]
    async fn send_tx(&self, tx: SignedTx) -> RpcResult<H256> {
        let hash = format!("{:?}", tx.hash());
        tracing::Span::current().record("hash", &hash);

        info!(
            hash,
            "Received transaction {hash} for rollup {}", tx.tx.rollup_id
        );
        let rollup_id_str = tx.tx.rollup_id.to_string();
        let metrics_attrs = &[KeyValue::new("rollup_id", rollup_id_str)];

        agglayer_telemetry::SEND_TX.add(1, metrics_attrs);

        if !self.kernel.check_rollup_registered(tx.tx.rollup_id) {
            // Return an invalid params error if the rollup is not registered.
            return Err(Error::rollup_not_registered(tx.tx.rollup_id));
        }

        agglayer_telemetry::CHECK_TX.add(1, metrics_attrs);

        // Run all the verification checks in parallel.
        try_join!(
            self.kernel
                .verify_signature(&tx)
                .map_err(|e| {
                    error!(error = %e, hash, "Failed to verify the signature of transaction {hash}: {e}");
                    Error::signature_mismatch(e)
                })
                .map_ok(|_| {
                    agglayer_telemetry::VERIFY_SIGNATURE.add(1, metrics_attrs);
                }),
            self.kernel
                .verify_proof_eth_call(&tx)
                .map_err(|e| {
                    let zkevm_error = decode_contract_error::<
                        _,
                        crate::contracts::polygon_zk_evm::PolygonZkEvmErrors,
                    >(&e);

                    let rollup_error = decode_contract_error::<
                        _,
                        crate::contracts::polygon_rollup_manager::PolygonRollupManagerErrors,
                    >(&e);

                    let error = match (zkevm_error, rollup_error) {
                        (Some(zkevm_error), _) => zkevm_error,
                        (_, Some(rollup_error)) => rollup_error,
                        (_, _) => e.to_string(),
                    };

                    error!(
                        error_code = %e,
                        error,
                        hash,
                        "Failed to dry-run the verify_batches_trusted_aggregator for transaction \
                         {hash}: {error}"
                    );
                    Error::dry_run(error)
                })
                .map_ok(|_| {
                    agglayer_telemetry::EXECUTE.add(1, metrics_attrs);
                }),
            self.kernel
                .verify_proof_zkevm_node(&tx)
                .map_err(|e| {
                    error!(
                        error = %e,
                        hash,
                        "Failed to verify the batch local_exit_root and state_root of transaction \
                         {hash}: {e}"
                    );
                    Error::root_verification(e)
                })
                .map_ok(|_| {
                    agglayer_telemetry::VERIFY_ZKP.add(1, metrics_attrs);
                })
        )?;

        // Settle the proof on-chain and return the transaction hash.
        let receipt = self.kernel.settle(&tx).await.map_err(|e| {
            error!(
                error = %e,
                hash,
                "Failed to settle transaction {hash} on L1: {e}"
            );
            Error::settlement(e)
        })?;

        agglayer_telemetry::SETTLE.add(1, metrics_attrs);

        info!(hash, "Successfully settled transaction {hash}");

        Ok(receipt.transaction_hash)
    }

    #[instrument(skip(self), fields(hash = hash.to_string()), level = "info")]
    async fn get_tx_status(&self, hash: H256) -> RpcResult<TxStatus> {
        debug!("Received request to get transaction status for hash {hash}");
        let receipt = self.kernel.check_tx_status(hash).await.map_err(|e| {
            error!("Failed to get transaction status for hash {hash}: {e}");
            StatusError::tx_status(e)
        })?;

        let current_block = self.kernel.current_l1_block_height().await.map_err(|e| {
            error!("Failed to get current L1 block: {e}");
            StatusError::l1_block(e)
        })?;

        let receipt = receipt.ok_or_else(|| StatusError::tx_not_found(hash))?;

        let status = match receipt.block_number {
            Some(block_number) if block_number < current_block => "done",
            Some(_) => "pending",
            None => "not found",
        };
        Ok(status.to_string())
    }

    async fn send_certificate(
        &self,
        certificate: MultiBatchHeader<Keccak256Hasher>,
    ) -> RpcResult<()> {
        if let Err(error) = self.certificate_sender.send(certificate).await {
            error!("Failed to send certificate: {error}");

            return Err(Error::send_certificate(error));
        }

        Ok(())
    }
}

fn decode_contract_error<M: Middleware, E: ContractRevert + std::fmt::Debug>(
    e: &ContractError<M>,
) -> Option<String> {
    e.decode_contract_revert::<E>()
        .map(|err| format!("{:?}", err))
}

type TxStatus = String;
