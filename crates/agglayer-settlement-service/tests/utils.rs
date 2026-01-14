use std::{sync::Arc, time::Duration};

use agglayer_settlement_service::utils::transactions::{
    wait_for_transaction_receipt, wait_for_transaction_receipt_with_confirmations,
};
use alloy::{
    node_bindings::Anvil,
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
};

#[test_log::test(tokio::test)]
async fn test_wait_for_transaction_receipt() {
    // Start Anvil node with 1 second block time
    let anvil = Anvil::new().block_time(1u64).spawn();
    let provider = ProviderBuilder::new().connect_http(anvil.endpoint_url());

    // Get accounts and send a transaction
    let accounts = provider.get_accounts().await.unwrap();
    let from = accounts[0];
    let to = accounts[1];

    let tx = TransactionRequest::default()
        .to(to)
        .value(alloy::primitives::U256::from(1000))
        .from(from);

    let pending_tx = provider.send_transaction(tx).await.unwrap();
    let tx_hash = *pending_tx.tx_hash();

    // Wrap the provider in Arc for the function call
    let provider_arc = Arc::new(provider);

    // Wait for the transaction receipt with a reasonable timeout
    let timeout = Duration::from_secs(30);
    let receipt = wait_for_transaction_receipt(provider_arc.clone(), tx_hash, timeout)
        .await
        .unwrap();

    // Verify the receipt contains expected data
    assert_eq!(receipt.transaction_hash, tx_hash);
    assert!(receipt.block_number.is_some());
    assert!(receipt.status());
}

#[test_log::test(tokio::test)]
async fn test_wait_for_transaction_receipt_with_confirmations() {
    // Start Anvil node with 1 second block time
    let anvil = Anvil::new().block_time(1u64).spawn();
    let provider = ProviderBuilder::new().connect_http(anvil.endpoint_url());

    // Get accounts and send a transaction
    let accounts = provider.get_accounts().await.unwrap();
    let from = accounts[0];
    let to = accounts[1];

    let tx = TransactionRequest::default()
        .to(to)
        .value(alloy::primitives::U256::from(2000))
        .from(from);

    let pending_tx = provider.send_transaction(tx).await.unwrap();
    let tx_hash = *pending_tx.tx_hash();

    // Wrap the provider in Arc for the function call
    let provider_arc = Arc::new(provider);

    // Wait for the transaction receipt with 3 confirmations
    let timeout = Duration::from_secs(30);
    let confirmations = 3;
    let receipt = wait_for_transaction_receipt_with_confirmations(
        provider_arc.clone(),
        tx_hash,
        timeout,
        confirmations,
    )
    .await
    .unwrap();

    // Verify the receipt contains expected data
    assert_eq!(receipt.transaction_hash, tx_hash);
    assert!(receipt.block_number.is_some());
    assert!(receipt.status());

    // Verify that we have at least the required number of confirmations
    let receipt_block = receipt.block_number.unwrap();
    let current_block = provider_arc.get_block_number().await.unwrap();
    let actual_confirmations = current_block
        .saturating_sub(receipt_block)
        .saturating_add(1);
    assert!(
        actual_confirmations >= confirmations as u64,
        "Expected at least {} confirmations, got {}",
        confirmations,
        actual_confirmations
    );
}

#[test_log::test(tokio::test)]
async fn test_wait_for_transaction_receipt_timeout() {
    // Start Anvil node with a very long block time (won't produce blocks during
    // test)
    let anvil = Anvil::new().block_time(1000u64).spawn();
    let provider = ProviderBuilder::new().connect_http(anvil.endpoint_url());

    // Use a fake transaction hash that doesn't exist
    let fake_tx_hash = alloy::primitives::TxHash::from([0u8; 32]);

    // Wrap the provider in Arc for the function call
    let provider_arc = Arc::new(provider);

    // Wait for a transaction that doesn't exist with a short timeout
    let timeout = Duration::from_secs(2);
    let result = wait_for_transaction_receipt(provider_arc.clone(), fake_tx_hash, timeout).await;

    // Should timeout
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(
            err,
            agglayer_settlement_service::utils::error::ClientRpcError::TransactionReceiptTimeout { .. }
        ),
        "Expected TransactionReceiptTimeout error, got: {:?}",
        err
    );
}
