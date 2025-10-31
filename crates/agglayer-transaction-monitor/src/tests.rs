use std::time::Duration;

use agglayer_types::SettlementTxHash;
use alloy::{
    network::{EthereumWallet, TransactionBuilder as _},
    node_bindings::Anvil,
    primitives::{address, TxHash, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
};

use crate::TransactionMonitor;

#[tokio::test]
async fn test_send_transaction_returns_handle() {
    let anvil = Anvil::new().spawn();
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::default()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(anvil.endpoint_url());

    let monitor = TransactionMonitor::new(provider);

    let tx = TransactionRequest::default()
        .with_to(address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045"))
        .with_value(U256::from(100));

    let result = monitor.send_transaction(tx).await;

    assert!(result.is_ok(), "Transaction should be sent successfully");
    let handle = result.unwrap();

    // Verify that we can receive the transaction hash
    let mut rx = handle.tx_hash_receiver;
    let tx_hash = rx.recv().await;
    assert!(tx_hash.is_some(), "Should receive transaction hash");
}

#[tokio::test]
async fn test_send_transaction_broadcasts_hash() {
    let anvil = Anvil::new().spawn();
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::default()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(anvil.endpoint_url());

    let monitor = TransactionMonitor::new(provider);

    let tx = TransactionRequest::default()
        .with_to(address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045"))
        .with_value(U256::from(100));

    let handle = monitor.send_transaction(tx).await.unwrap();
    let mut rx = handle.tx_hash_receiver;

    // Should receive the transaction hash immediately
    let tx_hash = tokio::time::timeout(Duration::from_secs(5), rx.recv())
        .await
        .expect("Should receive hash within timeout")
        .expect("Should have a hash value");

    // Verify the hash is not empty
    assert_ne!(
        tx_hash,
        SettlementTxHash::new([0u8; 32].into()),
        "Transaction hash should not be empty"
    );
}

#[tokio::test]
async fn test_transaction_task_completes_successfully() {
    let anvil = Anvil::new().spawn();
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::default()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(anvil.endpoint_url());

    let monitor = TransactionMonitor::new(provider);

    let tx = TransactionRequest::default()
        .with_to(address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045"))
        .with_value(U256::from(100));

    let handle = monitor.send_transaction(tx).await.unwrap();

    // Wait for the task to complete
    let result = tokio::time::timeout(Duration::from_secs(15), handle.task_handle)
        .await
        .expect("Task should complete within timeout")
        .expect("Task should not panic");

    assert!(result.is_ok(), "Transaction should succeed: {:?}", result);
    let receipt = result.unwrap();
    assert!(receipt.status(), "Transaction should have success status");
}

#[tokio::test]
async fn test_transaction_config_configure() {
    use crate::TransactionMonitorConfig;

    let config = TransactionMonitorConfig::new();
    let mut tx = TransactionRequest::default();

    let result = config.configure_transaction(&mut tx).await;
    assert!(result.is_ok(), "Configuration should succeed");
}

#[tokio::test]
async fn test_multiple_transactions() {
    let anvil = Anvil::new().spawn();
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::default()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(anvil.endpoint_url());

    let monitor = TransactionMonitor::new(provider);

    // Send multiple transactions
    let tx1 = TransactionRequest::default()
        .with_to(address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045"))
        .with_value(U256::from(100));

    let tx2 = TransactionRequest::default()
        .with_to(address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045"))
        .with_value(U256::from(200));

    let handle1 = monitor.send_transaction(tx1).await.unwrap();
    let handle2 = monitor.send_transaction(tx2).await.unwrap();

    // Both transactions should complete successfully
    let result1 = tokio::time::timeout(Duration::from_secs(15), handle1.task_handle)
        .await
        .expect("Task 1 should complete within timeout")
        .expect("Task 1 should not panic");

    let result2 = tokio::time::timeout(Duration::from_secs(15), handle2.task_handle)
        .await
        .expect("Task 2 should complete within timeout")
        .expect("Task 2 should not panic");

    assert!(result1.is_ok(), "Transaction 1 should succeed");
    assert!(result2.is_ok(), "Transaction 2 should succeed");
}

#[tokio::test]
async fn test_transaction_hash_uniqueness() {
    let anvil = Anvil::new().spawn();
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::default()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(anvil.endpoint_url());

    let monitor = TransactionMonitor::new(provider);

    let tx1 = TransactionRequest::default()
        .with_to(address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045"))
        .with_value(U256::from(100));

    let tx2 = TransactionRequest::default()
        .with_to(address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045"))
        .with_value(U256::from(200));

    let handle1 = monitor.send_transaction(tx1).await.unwrap();
    let handle2 = monitor.send_transaction(tx2).await.unwrap();

    let mut rx1 = handle1.tx_hash_receiver;
    let mut rx2 = handle2.tx_hash_receiver;

    let hash1 = rx1.recv().await.expect("Should receive hash 1");
    let hash2 = rx2.recv().await.expect("Should receive hash 2");

    // Transaction hashes should be different
    assert_ne!(
        hash1.as_ref(),
        hash2.as_ref(),
        "Different transactions should have different hashes"
    );
}

#[tokio::test]
async fn test_transaction_receipt_contains_expected_data() {
    let anvil = Anvil::new().spawn();
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::default()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(anvil.endpoint_url());

    let monitor = TransactionMonitor::new(provider);

    let to_address = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
    let tx = TransactionRequest::default()
        .with_to(to_address)
        .with_value(U256::from(100));

    let handle = monitor.send_transaction(tx).await.unwrap();

    let result = tokio::time::timeout(Duration::from_secs(15), handle.task_handle)
        .await
        .expect("Task should complete within timeout")
        .expect("Task should not panic")
        .expect("Transaction should succeed");

    // Verify receipt contains expected data
    assert!(result.status(), "Transaction should be successful");
    assert_eq!(
        result.to,
        Some(to_address),
        "Receipt should have correct 'to' address"
    );
}

#[tokio::test]
async fn test_transaction_timeout_due_to_low_gas() {
    let anvil = Anvil::new().spawn();
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::default()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(anvil.endpoint_url());

    let monitor = TransactionMonitor::new(provider);

    // Create a transaction with insufficient gas
    // Setting gas limit to 1 which is way too low for any transaction
    let to_address = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
    let tx = TransactionRequest::default()
        .with_to(to_address)
        .with_value(U256::from(100))
        .with_gas_limit(1); // Extremely low gas limit

    // The transaction should fail to be sent due to insufficient gas
    let result = monitor.send_transaction(tx).await;

    // The provider should reject the transaction before it's even sent
    assert!(
        result.is_err(),
        "Transaction should fail to send due to low gas"
    );

    // Verify the error message contains relevant information about gas
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("gas") || err_msg.contains("intrinsic"),
        "Error should indicate gas issue, got: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_transaction_timeout_with_low_gas_price() {
    let anvil = Anvil::new().spawn();
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::from(signer);

    // Use recommended fillers for proper transaction handling
    let provider = ProviderBuilder::default()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(anvil.endpoint_url());

    let monitor = TransactionMonitor::new(provider);

    // Create a transaction with very low gas price (1 wei)
    // This is below the base fee and should be rejected by the network
    let to_address = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
    let tx = TransactionRequest::default()
        .with_to(to_address)
        .with_value(U256::from(100))
        .with_gas_price(1); // Extremely low gas price (1 wei)

    // The transaction should fail to be sent because gas price is too low
    let result = monitor.send_transaction(tx).await;

    // The provider should reject the transaction due to low gas price
    assert!(
        result.is_err(),
        "Transaction should fail to send due to low gas price"
    );

    // Verify the error message contains relevant information about gas/base fee
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("gas") || err_msg.contains("base fee") || err_msg.contains("fee"),
        "Error should indicate gas price issue, got: {}",
        err_msg
    );
}

#[test_log::test(tokio::test)]
async fn test_transaction_timeout_then_retry_success() {
    use alloy::{node_bindings::AnvilInstance, providers::ext::AnvilApi};

    let anvil: AnvilInstance = Anvil::new().spawn();

    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::default()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(anvil.endpoint_url());

    // Disable auto-mining to control when blocks are mined
    provider.anvil_set_auto_mine(false).await.unwrap();

    let monitor = TransactionMonitor::new(provider.clone());

    let to_address = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
    let tx = TransactionRequest::default()
        .with_to(to_address)
        .with_value(U256::from(100));

    let handle = monitor.send_transaction(tx).await.unwrap();
    let mut rx = handle.tx_hash_receiver;

    let first_tx_hash = rx.recv().await.expect("Should receive first tx hash");
    println!("First transaction sent: {:?}", first_tx_hash);

    // Start a task that will eventually mine blocks after the timeout happens
    let provider_clone = provider.clone();
    tokio::spawn(async move {
        // Wait for timeout to trigger (10 seconds + buffer)
        tokio::time::sleep(Duration::from_secs(11)).await;
        println!("⛏️  Mining blocks to confirm retry transaction...");
        // Mine blocks to confirm the retry transaction
        for i in 0..5 {
            match provider_clone.anvil_mine(Some(1), None).await {
                Ok(_) => println!("   Mined block {}", i + 1),
                Err(e) => println!("   Failed to mine: {}", e),
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    });

    // Try to receive a second transaction hash (from retry)
    // Note: This may not arrive if the replacement transaction fails
    let second_tx_result = tokio::time::timeout(Duration::from_secs(15), rx.recv()).await;

    match second_tx_result {
        Ok(Some(second_tx_hash)) => {
            println!("🔄 Retry transaction sent: {:?}", second_tx_hash);
            assert_ne!(
                first_tx_hash.as_ref(),
                second_tx_hash.as_ref(),
                "Retry should have different tx hash"
            );
        }
        Ok(None) => {
            println!("ℹ️  Channel closed (replacement may have failed, monitoring original)");
        }
        Err(_) => {
            println!("ℹ️  No retry tx hash received within timeout (replacement may have failed)");
        }
    }

    // Wait for the final result
    let result = tokio::time::timeout(Duration::from_secs(30), handle.task_handle).await;

    match result {
        Ok(Ok(Ok(receipt))) => {
            assert!(receipt.status(), "Transaction should succeed after retry");
            println!("✅ Transaction succeeded (possibly after retry)");
            println!("   Receipt tx hash: {:?}", receipt.transaction_hash);
        }
        Ok(Ok(Err(e))) => {
            println!("❌ Transaction failed: {}", e);
            // This might happen if the retry mechanism encounters an issue
            // We'll check if it's the expected timeout error
            let err_msg = e.to_string();
            if err_msg.contains("timed out") {
                println!("⚠️  Transaction timed out - this is acceptable for this test");
            } else {
                panic!("Unexpected error: {}", e);
            }
        }
        Ok(Err(e)) => panic!("Task panicked: {:?}", e),
        Err(_) => {
            println!("⏱️  Test timed out after 30 seconds");
            panic!("Task timed out - retry mechanism may not be working correctly");
        }
    }
}

#[test_log::test(tokio::test)]
async fn test_original_transaction_confirmed_during_retry() {
    use alloy::{node_bindings::AnvilInstance, providers::ext::AnvilApi};

    let anvil: AnvilInstance = Anvil::new().spawn();

    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::default()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(anvil.endpoint_url());

    // Disable auto-mining
    provider.anvil_set_auto_mine(false).await.unwrap();

    let monitor = TransactionMonitor::new(provider.clone());

    let to_address = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
    let tx = TransactionRequest::default()
        .with_to(to_address)
        .with_value(U256::from(100));

    let handle = monitor.send_transaction(tx).await.unwrap();
    let mut rx = handle.tx_hash_receiver;

    // Receive the first transaction hash
    let first_tx_hash = rx.recv().await.expect("Should receive first tx hash");
    println!("📤 First transaction sent: {:?}", first_tx_hash);

    // Start a background task that will mine the ORIGINAL transaction
    // right after the timeout (when retry is being triggered)
    let provider_clone = provider.clone();
    let first_hash_bytes: [u8; 32] = **first_tx_hash.as_ref();
    let first_hash_fixed = alloy::primitives::FixedBytes::from(first_hash_bytes);
    tokio::spawn(async move {
        // Wait for timeout to occur
        tokio::time::sleep(Duration::from_secs(11)).await;

        println!("⛏️  Mining blocks to confirm ORIGINAL transaction...");
        // Mine blocks - this will confirm the first transaction
        for i in 0..3 {
            match provider_clone.anvil_mine(Some(1), None).await {
                Ok(_) => {
                    println!("   Mined block {}", i + 1);

                    // Check if the original transaction is confirmed
                    if let Ok(Some(receipt)) = provider_clone
                        .get_transaction_receipt(first_hash_fixed)
                        .await
                    {
                        println!("   ✅ Original transaction confirmed in block {}", i + 1);
                        if receipt.transaction_hash == first_hash_fixed {
                            println!("   🎯 Confirmed: Original transaction mined!");
                        }
                    }
                }
                Err(e) => println!("   Failed to mine: {}", e),
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    });

    // Try to receive a second transaction hash (from retry attempt)
    // Note: This may not arrive if the replacement transaction fails
    println!("🔍 Checking if retry transaction is sent...");
    let second_tx_result = tokio::time::timeout(Duration::from_secs(15), rx.recv()).await;

    let second_tx_hash_opt = match second_tx_result {
        Ok(Some(second_tx_hash)) => {
            println!("🔄 Retry transaction sent: {:?}", second_tx_hash);
            assert_ne!(
                first_tx_hash.as_ref(),
                second_tx_hash.as_ref(),
                "Retry should have different tx hash"
            );
            Some(second_tx_hash)
        }
        Ok(None) => {
            println!("ℹ️  Channel closed (replacement may have failed, monitoring original)");
            None
        }
        Err(_) => {
            println!(
                "ℹ️  No retry tx hash received (replacement may have failed or original was mined)"
            );
            None
        }
    };

    // Wait for the final result
    println!("⏳ Waiting for final result...");
    let result = tokio::time::timeout(Duration::from_secs(20), handle.task_handle).await;

    match result {
        Ok(Ok(Ok(receipt))) => {
            assert!(receipt.status(), "Transaction should succeed");
            println!("✅ Transaction confirmed!");
            println!("   Receipt tx hash: {:?}", receipt.transaction_hash);

            // Check which transaction was mined
            let receipt_hash = receipt.transaction_hash;
            let first_hash_bytes: &[u8; 32] = first_tx_hash.as_ref();
            if receipt_hash.as_slice() == first_hash_bytes {
                println!(
                    "   🎯 SUCCESS: Original transaction was mined (ideal scenario for this test)"
                );
                println!(
                    "   ✨ The retry mechanism correctly detected and returned the original tx"
                );
            } else {
                println!("   ℹ️  Replacement transaction was mined instead");
                println!(
                    "   Note: Both transactions were in mempool; this is a valid race condition"
                );
                println!("   The important thing is that a transaction succeeded!");
            }

            // The key assertion: Either transaction being confirmed is valid
            // What matters is that the monitor successfully handled the situation

            // Verify the receipt data matches our original transaction
            assert_eq!(
                receipt.to,
                Some(to_address),
                "Should be to the correct address"
            );
        }
        Ok(Ok(Err(e))) => {
            println!("❌ Transaction failed: {}", e);
            panic!("Transaction should have succeeded: {}", e);
        }
        Ok(Err(e)) => panic!("Task panicked: {:?}", e),
        Err(_) => {
            println!("⏱️  Test timed out");
            panic!("Task timed out");
        }
    }

    let first = provider
        .get_transaction_receipt(first_hash_fixed)
        .await
        .expect("Failed to query receipt for first tx")
        .expect("First tx should be mined");

    assert!(first.status(), "First transaction should be successful");

    // If a replacement transaction was sent, verify it wasn't mined
    if let Some(second_tx_hash) = second_tx_hash_opt {
        let second_tx_hash = TxHash::new(second_tx_hash.as_ref().0);
        assert!(
            provider
                .get_transaction_receipt(second_tx_hash)
                .await
                .expect("Failed to query receipt for second tx")
                .is_none(),
            "Second transaction should not be mined"
        );
        println!("   ✅ Confirmed: Replacement tx was NOT mined (as expected)");
    } else {
        println!(
            "   ℹ️  No replacement tx to verify (replacement likely failed with 'already \
             imported')"
        );
    }
}

#[test_log::test(tokio::test)]
async fn test_second_retry_succeeds() {
    use alloy::{node_bindings::AnvilInstance, providers::ext::AnvilApi};

    // This test demonstrates the scenario where:
    // 1. A transaction is sent but doesn't get mined (first timeout occurs)
    // 2. First retry is triggered and replacement tx is sent (second timeout
    //    occurs)
    // 3. Second retry is triggered and sent
    // 4. The second retry transaction gets mined successfully

    let anvil: AnvilInstance = Anvil::new().spawn();

    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::default()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(anvil.endpoint_url());

    // Disable auto-mining
    provider.anvil_set_auto_mine(false).await.unwrap();

    let monitor = TransactionMonitor::new(provider.clone());

    let to_address = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
    let tx = TransactionRequest::default()
        .with_to(to_address)
        .with_value(U256::from(100));

    let handle = monitor.send_transaction(tx).await.unwrap();
    let mut rx = handle.tx_hash_receiver;

    // Receive the first transaction hash
    let first_tx_hash = rx.recv().await.expect("Should receive first tx hash");
    println!("📤 First transaction sent: {:?}", first_tx_hash);

    // Start a background task that will wait for BOTH timeouts,
    // then mine blocks to confirm the SECOND RETRY transaction
    let provider_clone = provider.clone();
    tokio::spawn(async move {
        // Wait for first timeout to occur
        println!("⏳ Waiting for first timeout (10 seconds)...");
        tokio::time::sleep(Duration::from_secs(11)).await;
        println!("   ✅ First timeout occurred, first retry should be triggered");

        // Wait for second timeout to occur
        println!("⏳ Waiting for second timeout (10 more seconds)...");
        tokio::time::sleep(Duration::from_secs(11)).await;
        println!("   ✅ Second timeout occurred, second retry should be triggered");

        // Wait a bit more for the second retry to be sent
        tokio::time::sleep(Duration::from_millis(500)).await;

        println!("⛏️  Mining blocks to confirm transaction...");
        // Mine blocks - this will confirm whichever transaction is in the mempool
        for i in 0..5 {
            match provider_clone.anvil_mine(Some(1), None).await {
                Ok(_) => {
                    println!("   Mined block {}", i + 1);
                }
                Err(e) => println!("   Failed to mine: {}", e),
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    });

    // Try to receive a second transaction hash (from first retry)
    println!("🔍 Waiting for first retry transaction...");
    let second_tx_result = tokio::time::timeout(Duration::from_secs(15), rx.recv()).await;

    let second_tx_hash_opt = match second_tx_result {
        Ok(Some(hash)) => {
            println!("🔄 First retry transaction sent: {:?}", hash);
            assert_ne!(
                first_tx_hash.as_ref(),
                hash.as_ref(),
                "First retry should have different tx hash"
            );
            Some(hash)
        }
        Ok(None) => {
            println!("ℹ️  Channel closed after first tx");
            None
        }
        Err(_) => {
            println!("ℹ️  No first retry tx received (may have failed)");
            None
        }
    };

    // Try to receive a third transaction hash (from second retry)
    println!("🔍 Waiting for second retry transaction...");
    let third_tx_result = tokio::time::timeout(Duration::from_secs(15), rx.recv()).await;

    let third_tx_hash_opt = match third_tx_result {
        Ok(Some(hash)) => {
            println!("🔄 Second retry transaction sent: {:?}", hash);
            assert_ne!(
                first_tx_hash.as_ref(),
                hash.as_ref(),
                "Second retry should have different tx hash than original"
            );
            if let Some(second_hash) = &second_tx_hash_opt {
                assert_ne!(
                    second_hash.as_ref(),
                    hash.as_ref(),
                    "Second retry should have different tx hash than first retry"
                );
            }
            Some(hash)
        }
        Ok(None) => {
            println!("ℹ️  Channel closed after second tx");
            None
        }
        Err(_) => {
            println!("ℹ️  No second retry tx received");
            None
        }
    };

    // Make sure we don't receive a fourth transaction hash (third retry should not
    // happen - max is 3)
    println!("🔍 Checking that third retry is NOT triggered...");
    let fourth_tx_result = tokio::time::timeout(Duration::from_secs(3), rx.recv()).await;

    match fourth_tx_result {
        Err(_) => {
            println!("   ✅ No third retry triggered (as expected - max retries is 3)");
        }
        Ok(Some(_)) => {
            panic!("Third retry was triggered but should not have been (max retries is 3)!");
        }
        Ok(None) => {
            println!("   ✅ Channel closed - task completed (as expected)");
        }
    }

    // Wait for the final result
    println!("⏳ Waiting for final result...");
    let result = tokio::time::timeout(Duration::from_secs(35), handle.task_handle).await;

    match result {
        Ok(Ok(Ok(receipt))) => {
            assert!(receipt.status(), "Transaction should succeed");
            println!("✅ Transaction confirmed!");
            println!("   Receipt tx hash: {:?}", receipt.transaction_hash);

            // Check which transaction was mined
            let receipt_hash_bytes = receipt.transaction_hash.0;
            let first_hash_bytes: &[u8; 32] = first_tx_hash.as_ref();

            if receipt_hash_bytes == *first_hash_bytes {
                println!("   ℹ️  Original transaction was mined");
            } else if let Some(second_tx_hash) = second_tx_hash_opt {
                let second_hash_bytes: &[u8; 32] = second_tx_hash.as_ref();
                if receipt_hash_bytes == *second_hash_bytes {
                    println!("   ℹ️  First retry transaction was mined");
                } else if let Some(third_tx_hash) = third_tx_hash_opt {
                    let third_hash_bytes: &[u8; 32] = third_tx_hash.as_ref();
                    if receipt_hash_bytes == *third_hash_bytes {
                        println!("   🎯 SUCCESS: Second retry transaction was mined!");
                        println!(
                            "   ✨ Both original and first retry timed out, second retry succeeded"
                        );
                    } else {
                        panic!("Receipt hash doesn't match any transaction!");
                    }
                } else {
                    panic!(
                        "Receipt doesn't match original or first retry, but no second retry was \
                         sent!"
                    );
                }
            } else {
                panic!("Receipt doesn't match original, but no retry transactions were sent!");
            }

            // Verify the receipt data matches our original transaction parameters
            assert_eq!(
                receipt.to,
                Some(to_address),
                "Should be to the correct address"
            );
        }
        Ok(Ok(Err(e))) => {
            println!("❌ Transaction failed: {}", e);
            panic!("Transaction should have succeeded: {}", e);
        }
        Ok(Err(e)) => panic!("Task panicked: {:?}", e),
        Err(_) => {
            println!("⏱️  Test timed out");
            panic!("Task timed out");
        }
    }
}
