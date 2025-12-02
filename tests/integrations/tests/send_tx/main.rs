use std::sync::Arc;
use std::time::Duration;

use agglayer_config::rate_limiting::{RateLimitingConfig, TimeRateLimit};
use agglayer_storage::tests::TempDBDir;
use alloy::{
    primitives::{keccak256, B256, U64},
    signers::{local::LocalSigner, SignerSync},
};
use fail::FailScenario;
use integrations::agglayer_setup::{get_signer, setup_network};
use jsonrpsee::{
    core::client::ClientT as _,
    rpc_params,
    server::{Server, ServerHandle},
    types::{ErrorCode, ErrorObjectOwned},
    RpcModule,
};
use rstest::rstest;
use serde_json::Value as JsonValue;
use tokio_util::sync::CancellationToken;

// Deterministically derive roots for a given batch number
fn roots_for_batch(batch_no: u64) -> (B256, B256) {
    let s = format!("state-root-{batch_no}");
    let e = format!("exit-root-{batch_no}");
    (keccak256(s.as_bytes()), keccak256(e.as_bytes()))
}

// Start a lightweight mock ZkEVM HTTP JSON-RPC server
async fn start_mock_zkevm_server() -> (ServerHandle, String) {
    let server = Server::builder().build("127.0.0.1:0").await.unwrap();
    let addr = server.local_addr().unwrap();

    let mut module = RpcModule::new(());
    let _ = module
        .register_method(
            "zkevm_getBatchByNumber",
            |params, _, _| -> Result<JsonValue, ErrorObjectOwned> {
                let mut seq = params.sequence();
                let arg: String = seq.next().map_err(|_| ErrorCode::InvalidParams)?;
                // second param (bool) may be present but we don't need it
                let stripped = arg.strip_prefix("0x").ok_or(ErrorCode::InvalidParams)?;
                let batch_no =
                    u64::from_str_radix(stripped, 16).map_err(|_| ErrorCode::InvalidParams)?;
                let (state_root, exit_root) = roots_for_batch(batch_no);
                Ok(serde_json::json!({
                    "stateRoot": format!("0x{}", hex::encode(state_root.as_slice())),
                    "localExitRoot": format!("0x{}", hex::encode(exit_root.as_slice())),
                }))
            },
        )
        .unwrap();

    let handle = server.start(module);
    (handle, format!("http://{addr}"))
}

// Helper to create a test SignedTx with deterministic variation by index
fn create_signed_tx(
    rollup_id: u32,
    signer: &LocalSigner<alloy::signers::k256::ecdsa::SigningKey>,
    tx_index: u64,
) -> serde_json::Value {
    let last_verified_batch = U64::from(tx_index.saturating_sub(1));
    let new_verified_batch = U64::from(tx_index);
    let (new_state_root, new_local_exit_root) = roots_for_batch(tx_index);
    let proof = vec![tx_index as u8; 32 * 24];

    let last_verified_batch_hex = format!("0x{:x}", last_verified_batch.as_limbs()[0]);
    let new_verified_batch_hex = format!("0x{:x}", new_verified_batch.as_limbs()[0]);
    let proof_hex = format!("0x{}", hex::encode(&proof));

    let data = [
        last_verified_batch_hex.as_bytes(),
        new_verified_batch_hex.as_bytes(),
        new_state_root.as_slice(),
        new_local_exit_root.as_slice(),
        proof_hex.as_bytes(),
    ]
    .concat();

    let hash = keccak256(data);
    let signature = signer.sign_hash_sync(&hash).unwrap();

    serde_json::json!({
        "tx": {
            "RollupID": rollup_id,
            "lastVerifiedBatch": format!("0x{:x}", last_verified_batch.as_limbs()[0]),
            "newVerifiedBatch": format!("0x{:x}", new_verified_batch.as_limbs()[0]),
            "ZKP": {
                "newStateRoot": new_state_root,
                "newLocalExitRoot": new_local_exit_root,
                "proof": format!("0x{}", hex::encode(&proof))
            }
        },
        "signature": signature.to_string()
    })
}

/// Test that send_tx works without rate limiting (unlimited)
#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(60))]
async fn send_tx_unlimited_rate_limit() {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    let mut config = agglayer_config::Config::new(&tmp_dir.path);

    // Configure rate limiting as unlimited
    config.rate_limiting = RateLimitingConfig::new(TimeRateLimit::Unlimited);

    // Set up a proof signer for rollup_id 1
    let signer = get_signer(0);
    config.proof_signers.insert(1, signer.address().into());

    // Configure full node RPC (required for send_tx verification) -> mock ZkEVM
    let (_server, zkevm_url) = start_mock_zkevm_server().await;
    config.full_node_rpcs.insert(1, zkevm_url.parse().unwrap());

    let handle = CancellationToken::new();
    let (agglayer_shutdowned, _l1, client) =
        setup_network(&tmp_dir.path, Some(config), Some(handle.clone())).await;

    // Create and send multiple signed transactions in a loop
    // With unlimited rate limiting, all requests should succeed (or fail for reasons
    // other than rate limiting)
    for i in 1..=3u64 {
        let tx = create_signed_tx(1, &signer, i);
        let result: Result<B256, _> = client.request("interop_sendTx", rpc_params![tx]).await;
        
        // Check that the error (if any) is NOT a rate limit error
        if let Err(e) = result {
            assert!(
                !e.to_string().contains("rate limit"),
                "Request {} should not be rate limited with unlimited config: {}",
                i, e
            );
        }
    }

    handle.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}

/// Test that send_tx respects rate limiting
#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(60))]
async fn send_tx_with_rate_limit() {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    let mut config = agglayer_config::Config::new(&tmp_dir.path);

    // Configure rate limiting: 2 requests per 10 seconds
    config.rate_limiting =
        RateLimitingConfig::new(TimeRateLimit::limited(2, Duration::from_secs(10)));

    // Set up a proof signer for rollup_id 1
    let signer = get_signer(0);
    config.proof_signers.insert(1, signer.address().into());

    // Configure full node RPC (required for send_tx verification) -> mock ZkEVM
    let (_server, zkevm_url) = start_mock_zkevm_server().await;
    config.full_node_rpcs.insert(1, zkevm_url.parse().unwrap());

    let handle = CancellationToken::new();
    let (agglayer_shutdowned, _l1, client) =
        setup_network(&tmp_dir.path, Some(config), Some(handle.clone())).await;

    // Send transactions sequentially to test rate limiting
    // First 2 should succeed (or fail for non-rate-limit reasons)
    // Third should be rate limited
    let mut results = Vec::new();
    for i in 1..=3u64 {
        let tx = create_signed_tx(1, &signer, i);
        let result: Result<B256, _> = client.request("interop_sendTx", rpc_params![tx]).await;
        results.push(result);
    }

    // First two requests should not hit rate limit (max 2 per 10 seconds)
    for (i, result) in results[..2].iter().enumerate() {
        if let Err(e) = result {
            assert!(
                !e.to_string().contains("rate limit") && !e.to_string().contains("limited"),
                "Request {} should not be rate limited: {}",
                i + 1, e
            );
        }
    }

    // Third request should be rate limited
    assert!(
        results[2].is_err(),
        "Third request should fail due to rate limiting"
    );
    let error = results[2].as_ref().unwrap_err();
    assert!(
        error.to_string().contains("rate limit") || error.to_string().contains("limited") || error.to_string().contains("disabled"),
        "Third request should be rate limited, but got: {}",
        error
    );

    handle.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}

/// Test that send_tx rate limiting works per network
#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(60))]
async fn send_tx_rate_limit_per_network() {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    let mut config = agglayer_config::Config::new(&tmp_dir.path);

    // Configure rate limiting: 1 request per 10 seconds globally
    config.rate_limiting =
        RateLimitingConfig::new(TimeRateLimit::limited(1, Duration::from_secs(10)));

    // Set up proof signers for multiple rollups
    let signer1 = get_signer(0);
    let signer2 = get_signer(1);
    config.proof_signers.insert(1, signer1.address().into());
    config.proof_signers.insert(2, signer2.address().into());

    // Configure full node RPCs -> same mock ZkEVM for both networks
    let (_server, zkevm_url) = start_mock_zkevm_server().await;
    config.full_node_rpcs.insert(1, zkevm_url.parse().unwrap());
    config.full_node_rpcs.insert(2, zkevm_url.parse().unwrap());

    let handle = CancellationToken::new();
    let (agglayer_shutdowned, _l1, client) =
        setup_network(&tmp_dir.path, Some(config), Some(handle.clone())).await;

    // Test that each network has independent rate limit counters
    // Network 1: first request should succeed, second should be rate limited
    // Network 2: should have its own counter, not affected by network 1
    
    // First request to network 1 (should not be rate limited)
    let tx1_network1 = create_signed_tx(1, &signer1, 1);
    let result1: Result<B256, _> = client
        .request("interop_sendTx", rpc_params![tx1_network1])
        .await;
    if let Err(e) = &result1 {
        assert!(
            !e.to_string().contains("rate limit") && !e.to_string().contains("limited"),
            "First request to network 1 should not be rate limited: {}",
            e
        );
    }

    // First request to network 2 (should not be rate limited, independent counter)
    let tx1_network2 = create_signed_tx(2, &signer2, 1);
    let result2: Result<B256, _> = client
        .request("interop_sendTx", rpc_params![tx1_network2])
        .await;
    if let Err(e) = &result2 {
        assert!(
            !e.to_string().contains("rate limit") && !e.to_string().contains("limited"),
            "First request to network 2 should not be rate limited: {}",
            e
        );
    }

    // Second request to network 1 (should be rate limited, max 1 per 10s)
    let tx2_network1 = create_signed_tx(1, &signer1, 2);
    let result3: Result<B256, _> = client
        .request("interop_sendTx", rpc_params![tx2_network1])
        .await;
    assert!(
        result3.is_err(),
        "Second request to network 1 should fail due to rate limiting"
    );
    let error = result3.unwrap_err();
    assert!(
        error.to_string().contains("rate limit") || error.to_string().contains("limited") || error.to_string().contains("disabled"),
        "Second request to network 1 should be rate limited, but got: {}",
        error
    );

    handle.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}

/// Test that send_tx rate limiting can be overridden per network
#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(60))]
async fn send_tx_rate_limit_network_override() {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    let mut config = agglayer_config::Config::new(&tmp_dir.path);

    // Configure rate limiting: 1 request per 10 seconds globally,
    // but unlimited for network 2
    config.rate_limiting =
        RateLimitingConfig::new(TimeRateLimit::limited(1, Duration::from_secs(10)))
            .with_send_tx_override(2, TimeRateLimit::Unlimited);

    // Set up proof signers for multiple rollups
    let signer1 = get_signer(0);
    let signer2 = get_signer(1);
    config.proof_signers.insert(1, signer1.address().into());
    config.proof_signers.insert(2, signer2.address().into());

    let (_server, zkevm_url) = start_mock_zkevm_server().await;
    config.full_node_rpcs.insert(1, zkevm_url.parse().unwrap());
    config.full_node_rpcs.insert(2, zkevm_url.parse().unwrap());

    let handle = CancellationToken::new();
    let (agglayer_shutdowned, _l1, client) =
        setup_network(&tmp_dir.path, Some(config), Some(handle.clone())).await;

    // Test network-specific overrides
    // Network 1: global limit (1 per 10s)
    // Network 2: unlimited override
    
    // Network 1: First request should succeed
    let tx1_network1 = create_signed_tx(1, &signer1, 1);
    let result1: Result<B256, _> = client
        .request("interop_sendTx", rpc_params![tx1_network1])
        .await;
    if let Err(e) = &result1 {
        assert!(
            !e.to_string().contains("rate limit") && !e.to_string().contains("limited"),
            "First request to network 1 should not be rate limited: {}",
            e
        );
    }

    // Network 1: Second request should be rate limited (global limit applies)
    let tx2_network1 = create_signed_tx(1, &signer1, 2);
    let result2: Result<B256, _> = client
        .request("interop_sendTx", rpc_params![tx2_network1])
        .await;
    assert!(
        result2.is_err(),
        "Second request to network 1 should fail due to rate limiting"
    );
    let error = result2.unwrap_err();
    assert!(
        error.to_string().contains("rate limit") || error.to_string().contains("limited") || error.to_string().contains("disabled"),
        "Second request to network 1 should be rate limited, but got: {}",
        error
    );

    // Network 2: Multiple requests should all succeed (unlimited override)
    for i in 1..=3u64 {
        let tx_network2 = create_signed_tx(2, &signer2, i);
        let result: Result<B256, _> = client
            .request("interop_sendTx", rpc_params![tx_network2])
            .await;
        if let Err(e) = result {
            assert!(
                !e.to_string().contains("rate limit") && !e.to_string().contains("limited"),
                "Network 2 request {} should not be rate limited (unlimited override): {}",
                i, e
            );
        }
    }

    handle.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}

/// Test parallel requests to verify slot reservation mechanism
/// This test verifies that the rate limiter reserves slots immediately,
/// preventing spam even before transactions are processed.
/// 
/// With rate limit of 2 per 10 seconds and 3 parallel requests:
/// - 2 requests should reserve slots and hang (processing)
/// - 1 request should be immediately rejected (no slots available)
#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(60))]
async fn send_tx_rate_limit_parallel_requests() {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    let mut config = agglayer_config::Config::new(&tmp_dir.path);

    // Configure rate limiting: 2 requests per 10 seconds
    config.rate_limiting =
        RateLimitingConfig::new(TimeRateLimit::limited(2, Duration::from_secs(10)));

    // Set up a proof signer for rollup_id 1
    let signer = get_signer(0);
    config.proof_signers.insert(1, signer.address().into());

    // Configure full node RPC (required for send_tx verification) -> mock ZkEVM
    let (_server, zkevm_url) = start_mock_zkevm_server().await;
    config.full_node_rpcs.insert(1, zkevm_url.parse().unwrap());

    let handle = CancellationToken::new();
    let (agglayer_shutdowned, _l1, client) =
        setup_network(&tmp_dir.path, Some(config), Some(handle.clone())).await;

    // Configure fail point to inject delay after slot reservation
    // This ensures requests hang after reserving slots, allowing us to test
    // that the third request is rejected immediately
    fail::cfg("jsonrpc_api::send_tx::after_reserve", "return(2000)")
        .expect("Failed to configure failpoint");

    // Wrap client in Arc to share across tasks
    let client = Arc::new(client);

    // Launch 3 parallel requests
    let client1 = Arc::clone(&client);
    let client2 = Arc::clone(&client);
    let client3 = Arc::clone(&client);
    let signer1 = signer.clone();
    let signer2 = signer.clone();
    let signer3 = signer.clone();

    let task1 = tokio::spawn(async move {
        let tx = create_signed_tx(1, &signer1, 1);
        let start = std::time::Instant::now();
        let result: Result<B256, _> = client1.request("interop_sendTx", rpc_params![tx]).await;
        (result, start.elapsed())
    });

    let task2 = tokio::spawn(async move {
        let tx = create_signed_tx(1, &signer2, 2);
        let start = std::time::Instant::now();
        let result: Result<B256, _> = client2.request("interop_sendTx", rpc_params![tx]).await;
        (result, start.elapsed())
    });

    // Give tasks 1 and 2 a moment to start and reserve their slots
    tokio::time::sleep(Duration::from_millis(100)).await;

    let task3 = tokio::spawn(async move {
        let tx = create_signed_tx(1, &signer3, 3);
        let start = std::time::Instant::now();
        let result: Result<B256, _> = client3.request("interop_sendTx", rpc_params![tx]).await;
        (result, start.elapsed())
    });

    // Wait for all tasks to complete
    let (result1, _duration1) = task1.await.unwrap();
    let (result2, _duration2) = task2.await.unwrap();
    let (result3, duration3) = task3.await.unwrap();

    // Disable the fail point for cleanup
    fail::cfg("jsonrpc_api::send_tx::after_reserve", "off")
        .expect("Failed to disable failpoint");

    // Task 3 should be rejected immediately (no slot available)
    // Should complete much faster than tasks 1 and 2 (which are delayed by fail point)
    assert!(
        result3.is_err(),
        "Third parallel request should be rate limited"
    );
    let error3 = result3.unwrap_err();
    assert!(
        error3.to_string().contains("rate limit") || error3.to_string().contains("limited") || error3.to_string().contains("disabled"),
        "Third request should be rate limited, but got: {}",
        error3
    );
    assert!(
        duration3.as_millis() < 1000,
        "Third request should be rejected immediately (took {}ms, expected <1000ms)",
        duration3.as_millis()
    );

    // Tasks 1 and 2 should take approximately 2000ms due to the fail point delay
    // (They may fail for other reasons, but should not be rate limited)
    if let Err(e) = result1 {
        assert!(
            !e.to_string().contains("rate limit") && !e.to_string().contains("limited"),
            "First parallel request should not be rate limited: {}",
            e
        );
    }
    if let Err(e) = result2 {
        assert!(
            !e.to_string().contains("rate limit") && !e.to_string().contains("limited"),
            "Second parallel request should not be rate limited: {}",
            e
        );
    }

    handle.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}

/// Test that send_tx can be completely disabled via rate limiting
#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(60))]
async fn send_tx_rate_limit_disabled() {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    let mut config = agglayer_config::Config::new(&tmp_dir.path);

    // Configure rate limiting: 0 requests allowed (disabled)
    config.rate_limiting =
        RateLimitingConfig::new(TimeRateLimit::limited(0, Duration::from_secs(10)));

    // Set up a proof signer for rollup_id 1
    let signer = get_signer(0);
    config.proof_signers.insert(1, signer.address().into());

    let (_server, zkevm_url) = start_mock_zkevm_server().await;
    config.full_node_rpcs.insert(1, zkevm_url.parse().unwrap());

    let handle = CancellationToken::new();
    let (agglayer_shutdowned, _l1, client) =
        setup_network(&tmp_dir.path, Some(config), Some(handle.clone())).await;

    // With rate limit set to 0, even the first request should be rejected
    let tx = create_signed_tx(1, &signer, 1);

    let result: Result<B256, _> = client.request("interop_sendTx", rpc_params![tx]).await;

    assert!(
        result.is_err(),
        "Request should fail immediately when send_tx is disabled (rate limit = 0)"
    );
    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("disabled") || error.to_string().contains("rate limit") || error.to_string().contains("limited"),
        "Request should be rejected as disabled, but got: {}",
        error
    );

    handle.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}
