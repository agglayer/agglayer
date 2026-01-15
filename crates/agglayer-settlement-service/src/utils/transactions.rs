use std::{sync::Arc, time::Duration};

use agglayer_types::primitives::alloy_primitives::TxHash;
use alloy::{providers::Provider, rpc::types::TransactionReceipt};
use tracing::{debug, error, info, warn};

use crate::utils::error::TransactionReceiptError;

const RETRY_INTERVAL: Duration = Duration::from_secs(10);

#[tracing::instrument(level = "debug", skip(rpc_provider))]
async fn fetch_transaction_receipt<P>(
    // The L1 Middleware provider.
    rpc_provider: Arc<P>,
    // Chain transaction hash.
    tx_hash: TxHash,
) -> Result<Option<TransactionReceipt>, TransactionReceiptError>
where
    P: Provider,
{
    rpc_provider
        .get_transaction_receipt(tx_hash)
        .await
        .map_err(|error| {
            error!(
                ?error,
                "Failed to fetch transaction receipt for tx {tx_hash}"
            );
            TransactionReceiptError::RpcTransportError {
                tx_hash,
                source: error.into(),
            }
        })
}

/// Wait for the transaction to be included in the blockchain.
#[tracing::instrument(level = "debug", skip(rpc_provider))]
pub async fn wait_for_transaction_receipt<P>(
    // The L1 Middleware provider.
    rpc_provider: Arc<P>,
    // Chain transaction hash.
    tx_hash: TxHash,
    // Timeout for the transaction receipt.
    timeout: Duration,
) -> Result<TransactionReceipt, TransactionReceiptError>
where
    P: Provider + 'static,
{
    let retry_interval = RETRY_INTERVAL;

    debug!(?timeout, "Waiting for tx {tx_hash} to be mined");

    let wait_task = async {
        let mut attempts: usize = 0;
        loop {
            attempts += 1;

            match fetch_transaction_receipt(rpc_provider.clone(), tx_hash).await {
                Ok(Some(receipt)) => {
                    // If receipt is without block number, maybe node returned
                    // receipt for the pending tx. Try again.
                    if receipt.block_number.is_none() {
                        warn!("Tx {tx_hash} receipt has no block number. Trying again...");
                        tokio::time::sleep(retry_interval).await;
                        continue;
                    };
                    info!(attempts, "Successfully fetched receipt for tx {tx_hash}");
                    return Ok(receipt);
                }
                Ok(None) => {
                    debug!(
                        attempts,
                        ?retry_interval,
                        "L1 receipt for tx {tx_hash} not found yet, retrying",
                    );
                    tokio::time::sleep(retry_interval).await;
                }
                Err(error) => {
                    // Other error (e.g., network issue, RPC error)
                    error!(?error, "Error while waiting for the tx {tx_hash} receipt");
                    return Err(error);
                }
            }
        }
    };

    tokio::time::timeout(timeout, wait_task)
        .await
        .map_err(|_| {
            warn!(?timeout, "Timeout waiting for tx {tx_hash} receipt");
            TransactionReceiptError::TransactionReceiptTimeout { tx_hash, timeout }
        })?
}

// Check if there is a different receipt for the same transaction or if it is
// missing. Returns Ok(None) if no changes, Ok(new transaction receipt) if there
// is a new receipt, error if there is no receipt detected for the transaction.
async fn check_for_reorg<P>(
    rpc_provider: Arc<P>,
    tx_receipt: &TransactionReceipt,
) -> Result<Option<TransactionReceipt>, TransactionReceiptError>
where
    P: Provider + 'static,
{
    let tx_hash = tx_receipt.transaction_hash;
    // Get the receipt again, check for reorg
    match fetch_transaction_receipt(rpc_provider.clone(), tx_hash).await? {
        Some(new_receipt) => {
            if *tx_receipt != new_receipt {
                // Small reorg detected. We have a new receipt for the transaction.
                warn!(
                    "Reorg detected, receipts are different. Old block {:?} new block {:?}",
                    tx_receipt.block_number, new_receipt.block_number
                );
                Ok(Some(new_receipt))
            } else {
                // No reorg, receipts are the same
                Ok(None)
            }
        }
        None => {
            warn!(
                "Reorg detected, receipts are different. Old block {:?}, no new receipt available",
                tx_receipt.block_number
            );
            // No receipt for the transaction available anymore. Return error.
            Err(TransactionReceiptError::ReorgDetected {
                tx_hash,
                old_receipt: Box::new(Some(tx_receipt.clone())),
                new_receipt: Box::new(None),
            })
        }
    }
}

/// Wait for the transaction to be included in the blockchain and
/// to `confirmation` number of blocks to pass since that inclusion.
#[tracing::instrument(level = "debug", skip(rpc_provider))]
pub async fn wait_for_transaction_receipt_with_confirmations<P>(
    // The L1 Middleware provider.
    rpc_provider: Arc<P>,
    // Chain transaction hash.
    tx_hash: TxHash,
    // Timeout for the transaction receipt.
    timeout: Duration,
    // Required number of blocks for confirmation
    confirmations: usize,
) -> Result<TransactionReceipt, TransactionReceiptError>
where
    P: Provider + 'static,
{
    let retry_interval = RETRY_INTERVAL;

    let mut tx_receipt =
        wait_for_transaction_receipt(rpc_provider.clone(), tx_hash, timeout).await?;

    // Wait for the required number of confirmations
    if confirmations > 0 {
        let mut receipt_block = tx_receipt.block_number.ok_or_else(|| {
            // `wait_for_transaction_receipt` should return receipt with existing block
            // number.
            error!("Tx {tx_hash} receipt has no block number");
            TransactionReceiptError::InvalidReceipt { tx_hash }
        })?;

        debug!(
            receipt_block,
            confirmations,
            ?timeout,
            "Waiting for tx {tx_hash} chain block confirmations"
        );

        // Wait until we have the required number of confirmations.
        // Block where the transaction is included is the first confirmation.
        let confirmation_task = async {
            let mut attempts: usize = 0;
            loop {
                attempts += 1;
                match rpc_provider.get_block_number().await {
                    Ok(current_block) => {
                        let current_confirmations = current_block
                            .saturating_sub(receipt_block)
                            .saturating_add(1);
                        if current_confirmations >= confirmations as u64 {
                            // Read the receipt for the tx again, check for possible reorg while
                            // waiting for confirmations.
                            if let Some(new_tx_receipt) =
                                check_for_reorg(rpc_provider.clone(), &tx_receipt).await?
                            {
                                // Reorg detected. New receipt available for this tx.
                                // Update receipt and confirmations block number start.
                                receipt_block = match new_tx_receipt.block_number {
                                    Some(new_block_number) => new_block_number,
                                    None => {
                                        error!(
                                            "Tx {tx_hash} receipt has no block number after reorg"
                                        );
                                        return Err(TransactionReceiptError::ReorgDetected {
                                            tx_hash,
                                            old_receipt: Box::new(Some(tx_receipt)),
                                            new_receipt: Box::new(Some(new_tx_receipt)),
                                        });
                                    }
                                };
                                tx_receipt = new_tx_receipt;
                            }

                            info!(
                                current_confirmations,
                                confirmations,
                                current_block,
                                attempts,
                                "Tx {tx_hash} confirmed with {current_confirmations} confirmations"
                            );
                            return Ok(tx_receipt.clone());
                        } else {
                            debug!(
                                current_confirmations,
                                confirmations,
                                "Waiting for tx {tx_hash} more confirmations, sleeping"
                            );
                            tokio::time::sleep(retry_interval).await;
                        }
                    }
                    Err(error) => {
                        warn!(
                            ?error,
                            "Failed to get current block number while waiting for tx {tx_hash}, \
                             retrying"
                        );
                        tokio::time::sleep(retry_interval).await;
                    }
                }
            }
        };

        tokio::time::timeout(timeout, confirmation_task)
            .await
            .map_err(|_| {
                error!(?timeout, "Timeout waiting for tx {tx_hash} confirmations");
                TransactionReceiptError::TransactionReceiptTimeout { tx_hash, timeout }
            })?
    } else {
        Ok(tx_receipt)
    }
}
