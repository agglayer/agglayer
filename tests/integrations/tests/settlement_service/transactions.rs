use std::time::Duration;

use agglayer_contracts::contracts::PolygonRollupManager;
use agglayer_storage::tests::TempDBDir;
use agglayer_types::CertificateId;
use alloy::{
    network::{Ethereum, EthereumWallet},
    primitives::{Address, Bytes, FixedBytes},
    providers::{Provider, ProviderBuilder, RootProvider},
    signers::local::PrivateKeySigner,
};
use fail::FailScenario;
use integrations::agglayer_setup::{get_signer, setup_network, start_l1};
use jsonrpsee::{core::client::ClientT as _, rpc_params};
use rstest::rstest;
use tracing::info;

#[test_log::test(tokio::test)]
async fn start_l1_network() {
    // Test l1 node start and block production
    let _tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    info!("Starting L1 network...");
    let l1 = start_l1().await;

    info!("L1 network started, waiting for blocks to be mined...");

    // Create a provider to check the L1 chain
    let provider = RootProvider::<Ethereum>::new_http(reqwest::Url::parse(&l1.rpc).unwrap());

    // Get current block number
    let current_block = provider.get_block_number().await.unwrap();
    info!("Current block number: {}", current_block);

    info!("Test completed successfully");
    scenario.teardown();
}

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(180))]
#[case::type_0_ecdsa(crate::common::type_0_ecdsa_forest())]
async fn deconstruct_reconstruct_transaction(#[case] state: Forest) {
    let tmp_dir = TempDBDir::new();
    let scenario = FailScenario::setup();

    // L1 is a RAII guard
    let (_handle, l1, client) = setup_network(&tmp_dir.path, None, None).await;

    // Get a signer to send transactions
    // Use signer index 1 which is the trusted aggregator
    // (0x70997970C51812dc3A010C7d01b50e0d17dc79C8)
    let signer: PrivateKeySigner = get_signer(1);
    info!("Using signer address: {:?}", signer.address());
    let wallet = EthereumWallet::from(signer);

    // Create a provider with signer to interact with L1
    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect_http(reqwest::Url::parse(&l1.rpc).unwrap());

    // PolygonRollupManager contract address from the test setup
    let rollup_manager_address: Address = "0x0B306BF915C4d645ff596e518fAf3F9669b97016"
        .parse()
        .unwrap();

    // Create contract instance
    let rollup_manager = PolygonRollupManager::new(rollup_manager_address, provider);

    let withdrawals = vec![];
    let certificate = state.clone().apply_events(&[], &withdrawals);

    // Calculate the certificate ID manually by hashing the certificate
    let certificate_id: CertificateId = certificate.hash();
    info!("Manually calculated certificate_id: {}", certificate_id);

    // MANUALLY CRAFT THE verifyPessimisticTrustedAggregator CALL
    // instead of using interop_sendCertificate RPC

    // Prepare the arguments for verifyPessimisticTrustedAggregator
    let rollup_id: u32 = certificate.network_id.to_u32();
    // Use 0 as the default L1 info tree leaf count
    let l_1_info_tree_leaf_count: u32 = certificate.l1_info_tree_leaf_count.unwrap_or(0);
    let new_local_exit_root: FixedBytes<32> =
        FixedBytes::from_slice(certificate.new_local_exit_root.as_ref());

    // For testing purposes, we'll use dummy/placeholder values for some parameters
    // In a real scenario, these would come from the actual proof generation
    let new_pessimistic_root: FixedBytes<32> = FixedBytes::from([0u8; 32]); // Placeholder
    let proof: Bytes = Bytes::from(vec![0u8; 64]); // Placeholder proof (minimum size)
    let custom_chain_data: Bytes = Bytes::from(certificate.custom_chain_data.clone());

    info!("Building verifyPessimisticTrustedAggregator transaction with parameters:");
    info!("  rollup_id: {}", rollup_id);
    info!("  l_1_info_tree_leaf_count: {}", l_1_info_tree_leaf_count);
    info!(
        "  new_local_exit_root: 0x{}",
        hex::encode(new_local_exit_root)
    );
    info!(
        "  new_pessimistic_root: 0x{}",
        hex::encode(new_pessimistic_root)
    );
    info!("  proof length: {} bytes", proof.len());
    info!(
        "  custom_chain_data length: {} bytes",
        custom_chain_data.len()
    );

    // Create the transaction call
    let tx_call = rollup_manager.verifyPessimisticTrustedAggregator(
        rollup_id,
        l_1_info_tree_leaf_count,
        new_local_exit_root,
        new_pessimistic_root,
        proof.clone(),
        custom_chain_data.clone(),
    );

    // Extract and log the calldata
    let calldata = tx_call.calldata();
    info!("Transaction calldata: 0x{}", hex::encode(calldata));
    info!("Calldata length: {} bytes", calldata.len());

    // Send the manually crafted transaction directly to L1
    // WITHOUT calling interop_sendCertificate
    info!("Sending manually crafted verifyPessimisticTrustedAggregator transaction to L1...");
    let pending_tx = tx_call.send().await.unwrap();
    info!("Transaction sent, hash: {:?}", pending_tx.tx_hash());

    // Wait for the transaction to be mined
    let receipt = pending_tx.get_receipt().await.unwrap();
    info!("Transaction mined in block: {:?}", receipt.block_number);
    info!("Transaction status: {:?}", receipt.status());

    assert!(receipt.status(), "Transaction should succeed");

    scenario.teardown();
}
