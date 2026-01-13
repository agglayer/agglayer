use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use agglayer_types::primitives::alloy_primitives::TxHash;
use alloy::{providers::Provider, rpc::types::TransactionReceipt};
use tracing::{debug, error, info, warn};

use crate::utils::error::ClientRpcError;

const RECEIPT_RETRY_INTERVAL: Duration = Duration::from_secs(10);

async fn fetch_transaction_receipt<P>(
    // The L1 Middleware provider.
    rpc_provider: Arc<P>,
    // Chain transaction hash.
    tx_hash: TxHash,
) -> Result<Option<TransactionReceipt>, ClientRpcError>
where
    P: Provider,
{
    rpc_provider
        .get_transaction_receipt(tx_hash)
        .await
        .map_err(|error| {
            error!(%tx_hash, ?error, "Failed to fetch transaction receipt for tx {tx_hash}");
            ClientRpcError::TransactionReceiptError {
                tx_hash,
                source: error.into(),
            }
        })
}

/// Wait for the transaction to be included in the blockchain.
async fn wait_for_transaction_receipt<P>(
    // The L1 Middleware provider.
    rpc_provider: Arc<P>,
    // Chain transaction hash.
    tx_hash: TxHash,
    // Timeout for the transaction receipt.
    timeout: Duration,
) -> Result<TransactionReceipt, ClientRpcError>
where
    P: Provider + 'static,
{
    let mut attempts: usize = 0;
    let start_time = Instant::now();
    let retry_interval = RECEIPT_RETRY_INTERVAL;

    debug!(
        ?timeout,
        %tx_hash,
        "Waiting for transaction {tx_hash} to be mined",
    );

    loop {
        attempts += 1;

        match fetch_transaction_receipt(rpc_provider.clone(), tx_hash).await {
            Ok(Some(receipt)) => {
                info!(attempts, "Successfully fetched transaction receipt");
                return Ok(receipt);
            }
            Ok(None) => {
                // Transaction not yet included in a block, check if timeout has passed.
                let elapsed = start_time.elapsed();
                if elapsed >= timeout {
                    warn!(%tx_hash, ?elapsed, ?timeout, attempts, "Timeout waiting for transaction receipt");
                    return Err(ClientRpcError::TransactionReceiptTimeout { tx_hash, timeout });
                }

                debug!(%tx_hash, attempts, ?elapsed, ?timeout, ?retry_interval,
                    "L1 transaction receipt not found yet, retrying",
                );
                tokio::time::sleep(retry_interval).await;
            }
            Err(error) => {
                // Other error (e.g., network issue, RPC error)
                error!(
                    ?error,
                    "Error watching the pending signed transaction settlement"
                );
                return Err(error);
            }
        }
    }
}

/// Wait for the transaction to be included in the blockchain and
/// to `confirmation` number of blocks to pass since that inclusion.
async fn wait_for_transaction_receipt_with_confirmations<P>(
    // The L1 Middleware provider.
    rpc_provider: Arc<P>,
    // Chain transaction hash.
    tx_hash: TxHash,
    // Timeout for the transaction receipt.
    timeout: Duration,
    // Required number of blocks for confirmation
    confirmations: usize,
) -> Result<TransactionReceipt, ClientRpcError>
where
    P: Provider + 'static,
{
    let mut attempts: usize = 0;
    let start_time = Instant::now();
    let retry_interval = RECEIPT_RETRY_INTERVAL;

    let tx_receipt = wait_for_transaction_receipt(rpc_provider.clone(), tx_hash, timeout).await?;

    // Wait for the required number of confirmations
    if confirmations > 0 {
        let receipt_block = tx_receipt.block_number.ok_or_else(|| {
            error!(%tx_hash, "Transaction receipt has no block number");
            ClientRpcError::ReceiptWithoutBlockNumberError(tx_receipt.transaction_hash)
        })?;

        debug!(
            receipt_block,
            confirmations, "Waiting for chain block confirmations"
        );

        // Wait until we have the required number of confirmations.
        // Block where the transaction is included is the first confirmation.
        loop {
            attempts += 1;
            match rpc_provider.get_block_number().await {
                Ok(current_block) => {
                    let current_confirmations = current_block
                        .saturating_sub(receipt_block)
                        .saturating_add(1);
                    if current_confirmations >= confirmations as u64 {
                        info!(
                            current_confirmations,
                            confirmations,
                            current_block,
                            attempts,
                            "L1 transaction confirmed with required confirmations"
                        );
                        return Ok(tx_receipt.clone());
                    } else {
                        debug!(
                            current_confirmations,
                            confirmations, "Waiting for more confirmations, sleeping"
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

                    // Error happened while trying to get block number, check if timeout has passed.
                    let elapsed = start_time.elapsed();
                    if elapsed >= timeout {
                        warn!(%tx_hash, attempts, ?elapsed, ?timeout, attempts, "Timeout waiting for transaction receipt");
                        return Err(ClientRpcError::TransactionReceiptTimeout { tx_hash, timeout });
                    }

                    tokio::time::sleep(retry_interval).await;
                    continue;
                }
            }
        }
    } else {
        Ok(tx_receipt)
    }
}
