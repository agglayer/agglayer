use ethers::{providers::Middleware, types::H256};
use futures::TryFutureExt;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
    types::{
        error::{INTERNAL_ERROR_CODE, INTERNAL_ERROR_MSG, INVALID_PARAMS_CODE, INVALID_PARAMS_MSG},
        ErrorObject, ErrorObjectOwned,
    },
};
use tokio::try_join;
use tracing::{error, info};

use crate::{
    kernel::{Kernel, ZkevmNodeVerificationError},
    signed_tx::SignedTx,
};

#[rpc(server, namespace = "interop")]
trait Agglayer {
    #[method(name = "sendTx")]
    async fn send_tx(&self, tx: SignedTx) -> RpcResult<H256>;
}

/// The RPC agglayer service implementation.
pub(crate) struct AgglayerImpl<Rpc> {
    kernel: Kernel<Rpc>,
}

impl<Rpc> AgglayerImpl<Rpc> {
    /// Create an instance of the RPC agglayer service.
    pub(crate) fn new(kernel: Kernel<Rpc>) -> Self {
        Self { kernel }
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
    async fn send_tx(&self, tx: SignedTx) -> RpcResult<H256> {
        let rollup_id_str = tx.tx.rollup_id.to_string();
        crate::telemetry::SEND_TX
            .with_label_values(&[&rollup_id_str])
            .inc();

        if !self.kernel.check_rollup_registered(tx.tx.rollup_id) {
            // Return an invalid params error if the rollup is not registered.
            return Err(invalid_params_error(
                ZkevmNodeVerificationError::InvalidRollupId(tx.tx.rollup_id).to_string(),
            ));
        }

        crate::telemetry::CHECK_TX
            .with_label_values(&[&rollup_id_str])
            .inc();

        // Run all the verification checks in parallel.
        try_join!(
            self.kernel
                .verify_signature(&tx)
                .map_err(|e| {
                    error!("failed to verify signature: {e}");
                    invalid_params_error(e.to_string())
                })
                .map_ok(|_| {
                    crate::telemetry::VERIFY_SIGNATURE
                        .with_label_values(&[&rollup_id_str])
                        .inc();
                }),
            self.kernel
                .verify_proof_eth_call(&tx)
                .map_err(|e| {
                    error!("failed to verify proof eth_call: {e}");
                    invalid_params_error(e.to_string())
                })
                .map_ok(|_| {
                    crate::telemetry::VERIFY_ZKP
                        .with_label_values(&[&rollup_id_str])
                        .inc();
                }),
            self.kernel
                .verify_proof_zkevm_node(&tx)
                .map_err(|e| {
                    error!("failed to verify proof zkevm_node: {e}");
                    invalid_params_error(e.to_string())
                })
                .map_ok(|_| {
                    crate::telemetry::EXECUTE
                        .with_label_values(&[&rollup_id_str])
                        .inc();
                })
        )?;

        // Settle the proof on-chain and return the transaction hash.
        let receipt = self.kernel.settle(&tx).await.map_err(|e| {
            error!("failed to settle transaction: {e}");
            internal_error(e.to_string())
        })?;

        crate::telemetry::SETTLE
            .with_label_values(&[&rollup_id_str])
            .inc();

        info!("transaction settled: {receipt:?}");

        Ok(receipt.transaction_hash)
    }
}
