use agglayer_types::SettlementTxHash;
use alloy::{
    network::TransactionResponse as _, primitives::Address, providers::Provider,
    transports::TransportResult,
};

use crate::settlement_task::Nonce;

/// Returns the [`SettlementTxHash`] for a mined transaction matching the
/// given wallet and nonce, or `None` if no such mined transaction exists.
///
/// Mempool-only transactions are ignored.
pub async fn tx_hash_on_l1_for_nonce(
    provider: &impl Provider,
    wallet: Address,
    nonce: Nonce,
) -> TransportResult<Option<SettlementTxHash>> {
    let result = provider
        .get_transaction_by_sender_nonce(wallet, nonce.0)
        .await?;
    let Some(tx) = result else {
        return Ok(None);
    };
    Ok(tx
        .block_number()
        .is_some()
        .then(|| SettlementTxHash::from(tx.tx_hash())))
}

#[cfg(test)]
mod tests {
    use alloy::{
        consensus::Transaction as _,
        network::EthereumWallet,
        node_bindings::{Anvil, AnvilInstance},
        primitives::U256,
        providers::{Provider, ProviderBuilder},
        rpc::types::TransactionRequest,
        signers::local::PrivateKeySigner,
    };

    use super::*;

    // Existing single-endpoint variable used across the repository.
    const L1_RPC_ENV: &str = "L1_RPC_ENDPOINT";
    const MAX_SCAN_BLOCKS: u64 = 2_048;

    fn build_provider(anvil: &AnvilInstance) -> impl Provider {
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
        ProviderBuilder::new()
            .wallet(EthereumWallet::from(signer))
            .connect_http(anvil.endpoint_url())
    }

    fn external_rpc_url_from_env() -> Option<String> {
        match std::env::var(L1_RPC_ENV) {
            Ok(url) if url.trim().is_empty() => {
                println!("{L1_RPC_ENV} is set but empty; failing test");
                panic!("{L1_RPC_ENV} is defined but empty");
            }
            Ok(url) => {
                println!("{L1_RPC_ENV} is set; running external RPC compatibility check");
                Some(url)
            }
            Err(_) => {
                println!("{L1_RPC_ENV} is not set; skipping external RPC compatibility check");
                None
            }
        }
    }

    async fn find_recent_mined_transaction(
        provider: &impl Provider,
    ) -> TransportResult<Option<(Address, u64, SettlementTxHash)>> {
        let latest_block = provider.get_block_number().await?;
        let blocks_to_scan = latest_block.saturating_add(1).min(MAX_SCAN_BLOCKS);

        println!("Scanning up to {blocks_to_scan} block(s) for a mined transaction sample");

        for offset in 0..blocks_to_scan {
            let block_number = latest_block - offset;
            let Some(block) = provider
                .get_block_by_number(block_number.into())
                .full()
                .await?
            else {
                continue;
            };

            let Some(tx) = block.transactions.first_transaction() else {
                continue;
            };

            println!(
                "Found sample transaction in block {block_number} at nonce {}",
                tx.nonce()
            );

            return Ok(Some((
                tx.from(),
                tx.nonce(),
                SettlementTxHash::from(tx.tx_hash()),
            )));
        }

        println!("No mined transaction sample found in scan range");

        Ok(None)
    }

    #[tokio::test]
    async fn tx_hash_on_l1_for_nonce_returns_mined_tx() {
        let anvil = Anvil::new().spawn();
        let sender = anvil.addresses()[0];
        let provider = build_provider(&anvil);

        let tx = TransactionRequest::default()
            .to(anvil.addresses()[1])
            .value(U256::from(1));
        let receipt = provider
            .send_transaction(tx)
            .await
            .unwrap()
            .get_receipt()
            .await
            .unwrap();

        let result = tx_hash_on_l1_for_nonce(&provider, sender, Nonce(0))
            .await
            .unwrap();
        assert_eq!(
            result,
            Some(SettlementTxHash::from(receipt.transaction_hash))
        );
    }

    #[tokio::test]
    async fn tx_hash_on_l1_for_nonce_ignores_mempool_only_tx() {
        let anvil = Anvil::new().arg("--no-mining").spawn();
        let sender = anvil.addresses()[0];
        let provider = build_provider(&anvil);

        let tx = TransactionRequest::default()
            .to(anvil.addresses()[1])
            .value(U256::from(1));
        let _ = provider.send_transaction(tx).await.unwrap();

        let result = tx_hash_on_l1_for_nonce(&provider, sender, Nonce(0))
            .await
            .unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn tx_hash_on_l1_for_nonce_returns_none_for_non_submitted_nonce() {
        let anvil = Anvil::new().spawn();
        let sender = anvil.addresses()[0];
        let provider = build_provider(&anvil);

        let result = tx_hash_on_l1_for_nonce(&provider, sender, Nonce(0))
            .await
            .unwrap();
        assert_eq!(result, None);
    }

    // Manual run for any custom L1 RPC endpoint:
    // L1_RPC_ENDPOINT="https://<your-rpc-url>" cargo test -p agglayer-settlement-service tx_hash_on_l1_for_nonce_supports_external_l1_rpc_when_configured -- --nocapture
    #[tokio::test]
    async fn tx_hash_on_l1_for_nonce_supports_external_l1_rpc_when_configured() {
        println!("Starting external L1 RPC sender+nonce lookup test");

        let Some(rpc_url) = external_rpc_url_from_env() else {
            return;
        };

        let parsed_rpc_url = match rpc_url.parse() {
            Ok(url) => url,
            Err(_) => panic!("{L1_RPC_ENV} is invalid"),
        };

        println!("Parsed RPC URL; creating HTTP provider");

        let provider = ProviderBuilder::new().connect_http(parsed_rpc_url);

        println!("Fetching a mined transaction sample from recent blocks");

        let sample = match find_recent_mined_transaction(&provider).await {
            Ok(sample) => sample,
            Err(_) => panic!("Failed to query recent blocks through {L1_RPC_ENV}"),
        };

        let Some((sender, nonce, expected_hash)) = sample else {
            panic!(
                "No mined transactions found in the last {} blocks through {}; submit at least one \
                 transaction and retry",
                MAX_SCAN_BLOCKS,
                L1_RPC_ENV,
            );
        };

        println!("Querying tx hash via wallet + nonce RPC");

        let result = match tx_hash_on_l1_for_nonce(&provider, sender, Nonce(nonce)).await {
            Ok(result) => result,
            Err(_) => panic!("{L1_RPC_ENV} rejected eth_getTransactionBySenderAndNonce"),
        };

        println!("Comparing RPC result with sampled transaction hash");

        assert_eq!(
            result,
            Some(expected_hash),
            "Unexpected tx hash when querying by wallet + nonce through {}",
            L1_RPC_ENV,
        );

        println!("External L1 RPC sender+nonce lookup validated");
    }
}
