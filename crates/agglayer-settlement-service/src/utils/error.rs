use std::time::Duration;

use alloy::primitives::TxHash;

#[derive(thiserror::Error, Debug)]
pub enum ClientRpcError {
    #[error("Unable to fetch transaction receipt for {tx_hash}")]
    TransactionReceiptError {
        tx_hash: TxHash,
        #[source]
        source: eyre::Error,
    },
    #[error("Timeout waiting for transaction receipt for {tx_hash} after {timeout:?}")]
    TransactionReceiptTimeout { tx_hash: TxHash, timeout: Duration },
    #[error("Client provider error: {source}")]
    ProviderError {
        #[source]
        source: eyre::Error,
    },
    #[error("Receipt without block number: {tx_hash}")]
    ReceiptWithoutBlockNumberError { tx_hash: TxHash },
}
