use agglayer_telemetry::KeyValue;
use alloy::{primitives::B256, providers::Provider};
use futures::{future::try_join, TryFutureExt};
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
    #[instrument(skip(self, tx), fields(signed_tx_hash, rollup_id = tx.tx.rollup_id), level = "info")]
    pub async fn send_tx(&self, tx: SignedTx) -> Result<B256, SendTxError> {
        let signed_tx_hash = format!("{:?}", tx.hash());
        tracing::Span::current().record("signed_tx_hash", &signed_tx_hash);

        info!(
            signed_tx = ?tx,
            rollup_id = %tx.tx.rollup_id,
            "Received signed transaction {signed_tx_hash} for rollup {}",
            tx.tx.rollup_id
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
            error!("Rollup {rollup_id} is not registered");
            // Return an invalid params error if the rollup is not registered.
            return Err(SendTxError::RollupNotRegistered { rollup_id });
        }

        self.kernel
            .verify_tx_signature(&tx)
            .await
            .inspect_err(|error| {
                error!(?error, "Failed to verify the signature of transaction");
            })?;

        agglayer_telemetry::VERIFY_SIGNATURE.add(1, metrics_attrs_tx);

        // Reserve a rate limiting slot.
        let guard = self
            .kernel
            .rate_limiter()
            .reserve_send_tx(tx.tx.rollup_id, tokio::time::Instant::now())?;

        agglayer_telemetry::CHECK_TX.add(1, metrics_attrs);

        // Run all the verification checks in parallel.
        let verification = try_join(
            async {
                self.kernel
                    .verify_batches_trusted_aggregator(&tx)
                    .and_then(|call| async move { call.call().await })
                    .await
                    .map_err(|error| {
                        error!(
                            ?error,
                            "Failed to dry-run the verify_batches_trusted_aggregator for \
                             transaction"
                        );
                        SendTxError::dry_run(error)
                    })
                    .inspect(|_| agglayer_telemetry::EXECUTE.add(1, metrics_attrs))
            },
            async {
                self.kernel
                    .verify_proof_zkevm_node(&tx)
                    .await
                    .map_err(|error| {
                        error!(
                            ?error,
                            "Failed to verify the batch local_exit_root and state_root of \
                             transaction"
                        );
                        SendTxError::RootVerification(error)
                    })
                    .inspect(|_| agglayer_telemetry::VERIFY_ZKP.add(1, metrics_attrs))
            },
        )
        .await;

        if let Err(e) = verification {
            guard.record(tokio::time::Instant::now());
            return Err(e);
        }

        // Settle the proof on-chain and return the transaction hash.
        let receipt = self
            .kernel
            .settle(&tx, guard)
            .await
            .inspect_err(|error| error!(?error, "Failed to settle transaction on L1"))?;

        agglayer_telemetry::SETTLE.add(1, metrics_attrs);

        info!(
            l1_tx_hash = %receipt.transaction_hash,
            block_number = ?receipt.block_number,
            gas_used = %receipt.gas_used,
            "Signed transaction completed",

        );

        Ok(receipt.transaction_hash)
    }

    #[instrument(skip(self), fields(hash = hash.to_string()), level = "info")]
    pub async fn get_tx_status(&self, hash: B256) -> Result<TxStatus, TxStatusError> {
        debug!("Received request to get transaction status for l1 tx hash {hash}");

        let receipt = self.kernel.check_tx_status(hash).await.map_err(|error| {
            error!(
                ?error,
                "Failed to get transaction status for l1 tx hash {hash}"
            );
            TxStatusError::StatusCheck(error)
        })?;

        let current_block = self
            .kernel
            .current_l1_block_height()
            .await
            .map_err(|error| {
                error!(?error, "Failed to get current L1 block");
                TxStatusError::L1BlockRetrieval(error)
            })?;

        let receipt = receipt.ok_or_else(|| TxStatusError::TxNotFound { hash })?;

        let status = match receipt.block_number {
            Some(block_number) if block_number < current_block => TxStatus::Done,
            Some(_) => TxStatus::Pending,
            None => TxStatus::Pending,
        };
        Ok(status)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TxStatus {
    Done,
    Pending,
}

impl TxStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TxStatus::Done => "done",
            TxStatus::Pending => "pending",
        }
    }
}

impl std::fmt::Display for TxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
