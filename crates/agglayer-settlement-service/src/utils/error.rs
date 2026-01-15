use std::time::Duration;

use alloy::primitives::TxHash;

#[derive(thiserror::Error, Debug)]
pub enum TransactionReceiptError {
    #[error("Rpc transport error while fetch receipt for tx {tx_hash}")]
    RpcTransportError {
        tx_hash: TxHash,
        #[source]
        source: eyre::Error,
    },

    #[error("Timeout waiting for transaction receipt for {tx_hash} after {timeout:?}")]
    TransactionReceiptTimeout { tx_hash: TxHash, timeout: Duration },

    #[error("Invalid receipt for {tx_hash}")]
    InvalidReceipt { tx_hash: TxHash },
}
