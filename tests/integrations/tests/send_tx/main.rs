use std::time::Duration;

use agglayer_config::rate_limiting::{RateLimitingConfig, TimeRateLimit};
use agglayer_storage::tests::TempDBDir;
use alloy::{
    primitives::{keccak256, B256, U64},
    signers::{local::LocalSigner, SignerSync},
};
use fail::FailScenario;
use integrations::agglayer_setup::{get_signer, setup_network};
use jsonrpsee::{core::client::ClientT as _, rpc_params};
use rstest::rstest;
use tokio_util::sync::CancellationToken;

// Helper to create a test SignedTx
fn create_signed_tx(
    rollup_id: u32,
    signer: &LocalSigner<alloy::signers::k256::ecdsa::SigningKey>,
) -> serde_json::Value {
    let last_verified_batch = U64::from(0);
    let new_verified_batch = U64::from(1);
    let new_state_root = B256::with_last_byte(1);
    let new_local_exit_root = B256::with_last_byte(2);
    let proof = vec![0u8; 32 * 24]; // 24 hashes of 32 bytes each

    // Calculate hash as done in SignedTx::hash()
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

    // Configure full node RPC (required for send_tx verification)
    config
        .full_node_rpcs
        .insert(1, "http://localhost:8545".parse().unwrap());

    let handle = CancellationToken::new();
    let (agglayer_shutdowned, _l1, client) =
        setup_network(&tmp_dir.path, Some(config), Some(handle.clone())).await;

    // Create multiple signed transactions
    let tx1 = create_signed_tx(1, &signer);
    let tx2 = create_signed_tx(1, &signer);
    let tx3 = create_signed_tx(1, &signer);

    // All requests should succeed with unlimited rate limiting
    // Note: These will fail with RollupNotRegistered or other errors
    // but the important part is they won't fail with rate limit errors
    let result1: Result<B256, _> = client.request("interop_sendTx", rpc_params![tx1]).await;
    let result2: Result<B256, _> = client.request("interop_sendTx", rpc_params![tx2]).await;
    let result3: Result<B256, _> = client.request("interop_sendTx", rpc_params![tx3]).await;

    // Check that none of the errors are rate limit errors
    if let Err(e) = &result1 {
        assert!(
            !e.to_string().contains("rate limit"),
            "Should not be rate limited: {}",
            e
        );
    }
    if let Err(e) = &result2 {
        assert!(
            !e.to_string().contains("rate limit"),
            "Should not be rate limited: {}",
            e
        );
    }
    if let Err(e) = &result3 {
        assert!(
            !e.to_string().contains("rate limit"),
            "Should not be rate limited: {}",
            e
        );
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

    // Configure full node RPC (required for send_tx verification)
    config
        .full_node_rpcs
        .insert(1, "http://localhost:8545".parse().unwrap());

    let handle = CancellationToken::new();
    let (agglayer_shutdowned, _l1, client) =
        setup_network(&tmp_dir.path, Some(config), Some(handle.clone())).await;

    // Create multiple signed transactions
    let tx1 = create_signed_tx(1, &signer);
    let tx2 = create_signed_tx(1, &signer);
    let tx3 = create_signed_tx(1, &signer);

    // First two requests should not be rate limited (they may fail for other
    // reasons)
    let result1: Result<B256, _> = client.request("interop_sendTx", rpc_params![tx1]).await;
    let result2: Result<B256, _> = client.request("interop_sendTx", rpc_params![tx2]).await;

    // Check that first two are not rate limited
    if let Err(e) = &result1 {
        assert!(
            !e.to_string().contains("rate limit") && !e.to_string().contains("limited"),
            "First request should not be rate limited: {}",
            e
        );
    }
    if let Err(e) = &result2 {
        assert!(
            !e.to_string().contains("rate limit") && !e.to_string().contains("limited"),
            "Second request should not be rate limited: {}",
            e
        );
    }

    // Third request should be rate limited
    let result3: Result<B256, _> = client.request("interop_sendTx", rpc_params![tx3]).await;

    assert!(
        result3.is_err(),
        "Third request should fail due to rate limiting"
    );
    let error = result3.unwrap_err();
    assert!(
        error.to_string().contains("rate limit") || error.to_string().contains("limited"),
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

    // Configure full node RPCs
    config
        .full_node_rpcs
        .insert(1, "http://localhost:8545".parse().unwrap());
    config
        .full_node_rpcs
        .insert(2, "http://localhost:8545".parse().unwrap());

    let handle = CancellationToken::new();
    let (agglayer_shutdowned, _l1, client) =
        setup_network(&tmp_dir.path, Some(config), Some(handle.clone())).await;

    // Create transactions for different networks
    let tx1_network1 = create_signed_tx(1, &signer1);
    let tx2_network2 = create_signed_tx(2, &signer2);
    let tx3_network1 = create_signed_tx(1, &signer1);

    // First request to network 1 should not be rate limited
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

    // First request to network 2 should not be rate limited (different network)
    let result2: Result<B256, _> = client
        .request("interop_sendTx", rpc_params![tx2_network2])
        .await;
    if let Err(e) = &result2 {
        assert!(
            !e.to_string().contains("rate limit") && !e.to_string().contains("limited"),
            "First request to network 2 should not be rate limited: {}",
            e
        );
    }

    // Second request to network 1 should be rate limited
    let result3: Result<B256, _> = client
        .request("interop_sendTx", rpc_params![tx3_network1])
        .await;
    assert!(
        result3.is_err(),
        "Second request to network 1 should fail due to rate limiting"
    );
    let error = result3.unwrap_err();
    assert!(
        error.to_string().contains("rate limit") || error.to_string().contains("limited"),
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

    // Configure full node RPCs
    config
        .full_node_rpcs
        .insert(1, "http://localhost:8545".parse().unwrap());
    config
        .full_node_rpcs
        .insert(2, "http://localhost:8545".parse().unwrap());

    let handle = CancellationToken::new();
    let (agglayer_shutdowned, _l1, client) =
        setup_network(&tmp_dir.path, Some(config), Some(handle.clone())).await;

    // Create transactions for different networks
    let tx1_network1 = create_signed_tx(1, &signer1);
    let tx2_network1 = create_signed_tx(1, &signer1);
    let tx1_network2 = create_signed_tx(2, &signer2);
    let tx2_network2 = create_signed_tx(2, &signer2);
    let tx3_network2 = create_signed_tx(2, &signer2);

    // First request to network 1 should succeed
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

    // Second request to network 1 should be rate limited
    let result2: Result<B256, _> = client
        .request("interop_sendTx", rpc_params![tx2_network1])
        .await;
    assert!(
        result2.is_err(),
        "Second request to network 1 should fail due to rate limiting"
    );
    let error = result2.unwrap_err();
    assert!(
        error.to_string().contains("rate limit") || error.to_string().contains("limited"),
        "Second request to network 1 should be rate limited, but got: {}",
        error
    );

    // All requests to network 2 should not be rate limited (unlimited override)
    let result3: Result<B256, _> = client
        .request("interop_sendTx", rpc_params![tx1_network2])
        .await;
    let result4: Result<B256, _> = client
        .request("interop_sendTx", rpc_params![tx2_network2])
        .await;
    let result5: Result<B256, _> = client
        .request("interop_sendTx", rpc_params![tx3_network2])
        .await;

    if let Err(e) = &result3 {
        assert!(
            !e.to_string().contains("rate limit") && !e.to_string().contains("limited"),
            "Network 2 should not be rate limited: {}",
            e
        );
    }
    if let Err(e) = &result4 {
        assert!(
            !e.to_string().contains("rate limit") && !e.to_string().contains("limited"),
            "Network 2 should not be rate limited: {}",
            e
        );
    }
    if let Err(e) = &result5 {
        assert!(
            !e.to_string().contains("rate limit") && !e.to_string().contains("limited"),
            "Network 2 should not be rate limited: {}",
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

    // Configure full node RPC (required for send_tx verification)
    config
        .full_node_rpcs
        .insert(1, "http://localhost:8545".parse().unwrap());

    let handle = CancellationToken::new();
    let (agglayer_shutdowned, _l1, client) =
        setup_network(&tmp_dir.path, Some(config), Some(handle.clone())).await;

    // Create a signed transaction
    let tx = create_signed_tx(1, &signer);

    // Request should be rejected immediately
    let result: Result<B256, _> = client.request("interop_sendTx", rpc_params![tx]).await;

    assert!(
        result.is_err(),
        "Request should fail when send_tx is disabled"
    );
    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("disabled") || error.to_string().contains("rate limit"),
        "Request should be rejected as disabled, but got: {}",
        error
    );

    handle.cancel();
    _ = agglayer_shutdowned.await;

    scenario.teardown();
}
