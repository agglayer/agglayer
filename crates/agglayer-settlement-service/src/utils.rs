use agglayer_types::{Nonce, SettlementTxHash};
use alloy::{
    network::TransactionResponse as _, primitives::Address, providers::Provider,
    transports::TransportResult,
};

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
        network::EthereumWallet,
        node_bindings::{Anvil, AnvilInstance},
        primitives::U256,
        providers::ProviderBuilder,
        rpc::types::TransactionRequest,
        signers::local::PrivateKeySigner,
    };

    use super::*;

    fn build_provider(anvil: &AnvilInstance) -> impl Provider {
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
        ProviderBuilder::new()
            .wallet(EthereumWallet::from(signer))
            .connect_http(anvil.endpoint_url())
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
}
