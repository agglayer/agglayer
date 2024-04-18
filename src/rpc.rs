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

use crate::{kernel::Kernel, signed_tx::SignedTx};

#[rpc(server, namespace = "interop")]
trait Agglayer {
    #[method(name = "sendTx")]
    async fn send_tx(&self, tx: SignedTx) -> RpcResult<H256>;
}

/// The gRPC agglayer service implementation.
pub(crate) struct AgglayerImpl<Rpc> {
    kernel: Kernel<Rpc>,
}

impl<Rpc> AgglayerImpl<Rpc> {
    /// Create an instance of the gRPC agglayer service.
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
        // Run all the verification checks in parallel.
        try_join!(
            self.kernel.verify_signature(&tx).map_err(|e| {
                error!("failed to verify signature: {e}");
                invalid_params_error(e.to_string())
            }),
            self.kernel.verify_proof_eth_call(&tx).map_err(|e| {
                error!("failed to verify proof eth_call: {e}");
                invalid_params_error(e.to_string())
            }),
            self.kernel.verify_proof_zkevm_node(&tx).map_err(|e| {
                error!("failed to verify proof zkevm_node: {e}");
                invalid_params_error(e.to_string())
            }),
        )?;

        // Settle the proof on-chain and return the transaction hash.
        let receipt = self.kernel.settle(&tx).await.map_err(|e| {
            error!("failed to settle transaction: {e}");
            internal_error(e.to_string())
        })?;

        info!("transaction settled: {receipt:?}");

        Ok(receipt.transaction_hash)
    }
}
