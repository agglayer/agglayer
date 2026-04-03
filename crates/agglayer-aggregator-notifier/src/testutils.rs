use agglayer_contracts::{L1RpcError, Settler};
use agglayer_primitives::vkey_hash::VKeyHash;
use agglayer_types::{Address, SettlementTxHash};
use alloy::{
    contract::Error as ContractError,
    network::Ethereum,
    primitives::{Bytes, TxHash},
    rpc::types::TransactionReceipt,
};

mockall::mock! {
    pub L1Rpc {}

    #[async_trait::async_trait]
    impl agglayer_contracts::RollupContract for L1Rpc {
        async fn get_trusted_sequencer_address(
            &self,
            rollup_id: u32,
            proof_signers: std::collections::HashMap<u32, Address>,
        ) -> Result<Address, L1RpcError>;

        async fn get_rollup_contract_address(
            &self,
            rollup_id: u32,
        ) -> Result<Address, L1RpcError>;

        async fn get_l1_info_root(
            &self,
            l1_leaf_count: u32,
        ) -> Result<[u8; 32], L1RpcError>;

        fn default_l1_info_tree_entry(&self) -> (u32, [u8; 32]);

        async fn get_prev_pessimistic_root(
            &self,
            rollup_id: u32,
            before_tx: Option<TxHash>,
        ) -> Result<[u8; 32], L1RpcError>;

        async fn get_verifier_type(
            &self,
            rollup_id: u32,
        ) -> Result<agglayer_contracts::rollup::VerifierType, L1RpcError>;

        fn get_rollup_manager_address(&self) -> Address;

        fn get_event_filter_block_range(&self) -> u64;
    }

    #[async_trait::async_trait]
    impl agglayer_contracts::AggchainContract for L1Rpc {
        async fn get_aggchain_vkey_hash(
            &self,
            rollup_address: Address,
            aggchain_vkey_selector: u16,
        ) -> Result<VKeyHash, L1RpcError>;

        async fn get_aggchain_hash(
            &self,
            rollup_address: Address,
            aggchain_data: Bytes,
        ) -> Result<[u8; 32], L1RpcError>;

        async fn get_multisig_context(
            &self,
            rollup_address: Address,
        ) -> Result<(Vec<Address>, usize), L1RpcError>;
    }

    #[async_trait::async_trait]
    impl agglayer_contracts::L1TransactionFetcher for L1Rpc {
        type Provider = alloy::providers::RootProvider<Ethereum>;

        async fn fetch_transaction_receipt(
            &self,
            tx_hash: SettlementTxHash,
        ) -> Result<Option<TransactionReceipt>, L1RpcError>;

        fn get_provider(
            &self,
        ) -> &<Self as agglayer_contracts::L1TransactionFetcher>::Provider;
    }

    #[async_trait::async_trait]
    impl Settler for L1Rpc {
        fn decode_contract_revert(error: &ContractError) -> Option<String>;

        async fn verify_pessimistic_trusted_aggregator(
            &self,
            rollup_id: u32,
            l_1_info_tree_leaf_count: u32,
            new_local_exit_root: [u8; 32],
            new_pessimistic_root: [u8; 32],
            proof: Bytes,
            custom_chain_data: Bytes,
            nonce: Option<(u64, u128, Option<u128>)>,
        ) -> Result<
            alloy::providers::PendingTransactionBuilder<Ethereum>,
            ContractError,
        >;
    }
}
