use std::time::Duration;

use agglayer_types::{Digest, SettlementTxHash};
use alloy::{
    consensus::Transaction as _,
    network::Ethereum,
    providers::{PendingTransactionBuilder, PendingTransactionError, Provider, WatchTxError},
    rpc::types::{TransactionReceipt, TransactionRequest},
};
use eyre::{bail, Result};
use tokio::sync::mpsc;
use tracing::{info, instrument, warn};

#[derive(Clone)]
pub struct TransactionMonitorConfig {
    max_retries: usize,
}

impl TransactionMonitorConfig {
    fn new() -> Self {
        Self { max_retries: 3 }
    }

    async fn configure_transaction(&self, _transaction: &mut TransactionRequest) -> Result<()> {
        // Here you can set gas price, nonce, or other parameters as needed
        // For example:
        // transaction.gas_price = Some(self.get_gas_price().await?);
        // transaction.nonce = Some(self.get_nonce().await?);
        Ok(())
    }
}
pub struct TransactionMonitor<RpcProvider> {
    provider: RpcProvider,
    config: TransactionMonitorConfig,
}

impl<RpcProvider> TransactionMonitor<RpcProvider>
where
    RpcProvider: Provider + Clone + 'static,
{
    #[instrument(skip(self, transaction))]
    pub async fn send_transaction(
        &self,
        mut transaction: TransactionRequest,
    ) -> Result<TransactionMonitorTaskHandle> {
        self.config.configure_transaction(&mut transaction).await?;
        info!("Transaction configured");

        // Clone the transaction request before sending for potential retries
        let original_tx_request = transaction.clone();

        let pending_tx = self.provider.send_transaction(transaction).await?;
        let tx_hash = Digest::from(*pending_tx.tx_hash());
        info!(%tx_hash, "Transaction sent");

        let (tx_hash_notifier, tx_hash_receiver) = mpsc::channel(10);

        if tx_hash_notifier
            .send(SettlementTxHash::from(tx_hash))
            .await
            .is_err()
        {
            bail!("Failed to send tx hash to notifier channel");
        }

        let task = TransactionMonitorTask {
            transaction: pending_tx,
            provider: self.provider.clone(),
            tx_hash_notifier: Some(tx_hash_notifier),
            remaining_retries: self.config.max_retries,
            config: self.config.clone(),
            original_tx_request,
        };

        let task_handle = tokio::spawn(task.run());
        Ok(TransactionMonitorTaskHandle {
            tx_hash_receiver,
            task_handle,
        })
    }

    pub fn new(provider: RpcProvider) -> Self {
        Self {
            provider,
            config: TransactionMonitorConfig::new(),
        }
    }
}

pub struct TransactionMonitorTask<RpcProvider> {
    transaction: PendingTransactionBuilder<Ethereum>,
    provider: RpcProvider,
    #[allow(unused)]
    config: TransactionMonitorConfig,
    tx_hash_notifier: Option<mpsc::Sender<SettlementTxHash>>,
    remaining_retries: usize,
    original_tx_request: TransactionRequest,
}

impl<RpcProvider> TransactionMonitorTask<RpcProvider>
where
    RpcProvider: Provider + Clone,
{
    pub(crate) async fn run(mut self) -> Result<TransactionReceipt> {
        info!("Starting transaction monitoring");
        loop {
            info!(
                "Waiting for transaction receipt ({} retries left)",
                self.remaining_retries
            );
            match PendingTransactionBuilder::new(
                self.provider.root().clone(),
                *self.transaction.tx_hash(),
            )
            .with_timeout(Some(Duration::from_secs(10)))
            .with_required_confirmations(1)
            .get_receipt()
            .await
            {
                Ok(receipt) if receipt.status() => {
                    info!("Transaction confirmed successfully");
                    return Ok(receipt);
                }
                Ok(_) => {
                    warn!("Transaction reverted");
                    // Handle transaction reverted
                    bail!("Transaction reverted");
                }
                Err(PendingTransactionError::TxWatcher(WatchTxError::Timeout))
                    if self.remaining_retries > 0 =>
                {
                    info!("Transaction timed out, attempting retry...");
                    self.remaining_retries -= 1;

                    // Check if the transaction was actually mined
                    let tx_hash = *self.transaction.tx_hash();
                    let receipt = self.provider.get_transaction_receipt(tx_hash).await?;

                    if let Some(receipt) = receipt {
                        // Transaction was mined, just return the receipt
                        info!("Transaction was already mined during timeout");
                        if receipt.status() {
                            return Ok(receipt);
                        } else {
                            bail!("Transaction reverted");
                        }
                    }

                    // Transaction not mined, send replacement with higher gas
                    info!("Transaction not mined, sending replacement with higher gas");
                    match self.send_replacement_tx().await {
                        Ok(new_pending_tx) => {
                            // Update the transaction we're monitoring
                            self.transaction = new_pending_tx;

                            // Notify about the new transaction hash
                            if let Some(ref notifier) = self.tx_hash_notifier {
                                let new_tx_hash = Digest::from(*self.transaction.tx_hash());
                                let _ = notifier.send(SettlementTxHash::from(new_tx_hash)).await;
                            }

                            // Continue monitoring the new transaction
                            continue;
                        }
                        Err(e) => {
                            // If replacement fails (e.g., "transaction already imported"),
                            // continue monitoring the original transaction
                            warn!(
                                "Replacement transaction failed, continuing to monitor original: \
                                 {}",
                                e
                            );

                            // Continue monitoring the same transaction
                            continue;
                        }
                    }
                }
                Err(PendingTransactionError::TxWatcher(WatchTxError::Timeout)) => {
                    // No retries left
                    warn!("Transaction timed out and no retries left");
                    bail!("Transaction timed out and no retries left");
                }
                Err(_) => {
                    // Handle other errors
                    warn!("Transaction failed due to an error");
                    bail!("Transaction failed");
                }
            }
        }
    }

    async fn send_replacement_tx(&mut self) -> Result<PendingTransactionBuilder<Ethereum>> {
        let original_tx_hash = *self.transaction.tx_hash();
        let original_tx = self
            .provider
            .get_transaction_by_hash(original_tx_hash)
            .await?
            .ok_or_else(|| eyre::eyre!("Original transaction not found"))?;

        let mut replacement_tx = self.original_tx_request.clone();

        // CRITICAL: Reuse the same nonce to replace the original transaction
        let original_nonce = original_tx.inner.nonce();

        info!(
            "Creating replacement transaction with nonce {} (same as original)",
            original_nonce
        );

        replacement_tx.nonce = Some(original_nonce);

        if let Some(gas_price) = replacement_tx.gas_price {
            let new_gas_price = gas_price * 150 / 100;
            replacement_tx.gas_price = Some(new_gas_price);
            info!(
                "Increased gas_price from {} to {} (+50%)",
                gas_price, new_gas_price
            );
        }

        if let Some(max_fee) = replacement_tx.max_fee_per_gas {
            let new_max_fee = max_fee * 150 / 100;
            replacement_tx.max_fee_per_gas = Some(new_max_fee);
            info!(
                "Increased max_fee_per_gas from {} to {} (+50%)",
                max_fee, new_max_fee
            );
        }
        if let Some(max_priority_fee) = replacement_tx.max_priority_fee_per_gas {
            let new_priority_fee = max_priority_fee * 150 / 100;
            replacement_tx.max_priority_fee_per_gas = Some(new_priority_fee);
            info!(
                "Increased max_priority_fee_per_gas from {} to {} (+50%)",
                max_priority_fee, new_priority_fee
            );
        }

        info!("Sending replacement transaction with same nonce and increased gas");
        match self.provider.send_transaction(replacement_tx).await {
            Ok(pending_tx) => {
                info!("Replacement transaction sent successfully");
                Ok(pending_tx)
            }
            Err(e) => {
                warn!("Failed to send replacement transaction: {}", e);
                Err(e.into())
            }
        }
    }
}

#[derive(Debug)]
pub struct TransactionMonitorTaskHandle {
    pub tx_hash_receiver: mpsc::Receiver<SettlementTxHash>,
    pub task_handle: tokio::task::JoinHandle<eyre::Result<TransactionReceipt>>,
}

#[cfg(test)]
mod tests;
