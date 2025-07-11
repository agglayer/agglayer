use agglayer_telemetry::KeyValue;
use alloy::{primitives::B256, providers::Provider};
use futures::future::try_join;
use tracing::{debug, error, info, instrument};

pub use self::error::{CertificateRetrievalError, SendTxError, TxStatusError};
use crate::{kernel::Kernel, signed_tx::SignedTx};

pub mod error;

/// The RPC agglayer service implementation.
pub struct AgglayerService<Rpc> {
    pub(crate) kernel: Kernel<Rpc>,
}

impl<Rpc> AgglayerService<Rpc> {
    /// Create an instance of the RPC agglayer service.
    pub fn new(kernel: Kernel<Rpc>) -> Self {
        Self { kernel }
    }
}

impl<Rpc> Drop for AgglayerService<Rpc> {
    fn drop(&mut self) {
        info!("Shutting down the agglayer JSON-RPC service");
    }
}

impl<Rpc> AgglayerService<Rpc>
where
    Rpc: Provider + Clone + 'static,
{
    #[instrument(skip(self, tx), fields(hash, rollup_id = tx.tx.rollup_id), level = "info")]
    pub async fn send_tx(&self, tx: SignedTx) -> Result<B256, SendTxError> {
        let hash = format!("{:?}", tx.hash());
        tracing::Span::current().record("hash", &hash);

        info!(
            hash,
            "Received transaction {hash} for rollup {}", tx.tx.rollup_id
        );
        let rollup_id_str = tx.tx.rollup_id.to_string();
        let metrics_attrs_tx = &[
            KeyValue::new("rollup_id", rollup_id_str),
            KeyValue::new("type", "tx"),
        ];
        let metrics_attrs = &metrics_attrs_tx[..1];

        agglayer_telemetry::SEND_TX.add(1, metrics_attrs);

        let rollup_id = tx.tx.rollup_id;
        if !self.kernel.check_rollup_registered(rollup_id) {
            // Return an invalid params error if the rollup is not registered.
            return Err(SendTxError::RollupNotRegistered { rollup_id });
        }

        self.kernel.verify_tx_signature(&tx).await.inspect_err(|e| {
            error!(error = %e, hash, "Failed to verify the signature of transaction {hash}: {e}");
        })?;

        agglayer_telemetry::VERIFY_SIGNATURE.add(1, metrics_attrs_tx);

        // Reserve a rate limiting slot.
        let guard = self
            .kernel
            .rate_limiter()
            .reserve_send_tx(tx.tx.rollup_id, tokio::time::Instant::now())?;

        agglayer_telemetry::CHECK_TX.add(1, metrics_attrs);

        // Run all the verification checks in parallel.
        let _ = try_join(
            async {
                self.kernel
                    .verify_batches_trusted_aggregator(&tx)
                    .await
                    .map_err(|e| {
                        error!(
                            error_code = %e,
                            hash,
                            "Failed to dry-run the verify_batches_trusted_aggregator for \
                             transaction {hash}: {e}"
                        );
                        SendTxError::dry_run(e)
                    })
                    .inspect(|_| agglayer_telemetry::EXECUTE.add(1, metrics_attrs))
            },
            async {
                self.kernel
                    .verify_proof_zkevm_node(&tx)
                    .await
                    .map_err(|e| {
                        error!(
                            error = %e,
                            hash,
                            "Failed to verify the batch local_exit_root and state_root of \
                             transaction {hash}: {e}"
                        );
                        SendTxError::RootVerification(e)
                    })
                    .inspect(|_| agglayer_telemetry::VERIFY_ZKP.add(1, metrics_attrs))
            },
        )
        .await?;

        // Settle the proof on-chain and return the transaction hash.
        let receipt = self.kernel.settle(&tx, guard).await.inspect_err(|e| {
            error!(
                error = %e,
                hash,
                "Failed to settle transaction {hash} on L1: {e}"
            )
        })?;

        agglayer_telemetry::SETTLE.add(1, metrics_attrs);

        info!(hash, "Successfully settled transaction {hash}");

        Ok(receipt.transaction_hash)
    }

    #[instrument(skip(self), fields(hash = hash.to_string()), level = "info")]
    pub async fn get_tx_status(&self, hash: B256) -> Result<TxStatus, TxStatusError> {
        debug!("Received request to get transaction status for hash {hash}");

        let receipt = self.kernel.check_tx_status(hash).await.map_err(|e| {
            error!("Failed to get transaction status for hash {hash}: {e}");
            TxStatusError::StatusCheck(e)
        })?;

        let current_block = self.kernel.current_l1_block_height().await.map_err(|e| {
            error!("Failed to get current L1 block: {e}");
            TxStatusError::L1BlockRetrieval(e)
        })?;

        let receipt = receipt.ok_or_else(|| TxStatusError::TxNotFound { hash })?;

        let status = match receipt.block_number {
            Some(block_number) if block_number < current_block => TxStatus::Done,
            Some(_) => TxStatus::Pending,
            None => TxStatus::NotFound,
        };
        Ok(status)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TxStatus {
    Done,
    Pending,
    NotFound,
}

impl TxStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TxStatus::Done => "done",
            TxStatus::Pending => "pending",
            TxStatus::NotFound => "not found",
        }
    }
}

impl std::fmt::Display for TxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
