use alloy::primitives::TxHash;
use alloy::rpc::types::TransactionReceipt;

pub(crate) async fn check_for_receipts<P>(
    provider: P,
    tx_hashes: Vec<TxHash>,
) -> eyre::Result<Vec<Option<TransactionReceipt>>>
where
    P: alloy::providers::Provider + Clone + 'static,
{
    let mut tasks = Vec::new();
    for tx_hash in tx_hashes {
        let provider = provider.clone();
        tasks.push(tokio::task::spawn(async move {
            provider.get_transaction_receipt(tx_hash).await
        }));
    }

    let mut receipts = Vec::new();
    for task in tasks {
        receipts.push(task.await??);
    }

    Ok(receipts)
}
