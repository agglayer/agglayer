use agglayer_contracts::L1RpcError;
use agglayer_primitives::vkey_hash::VKeyHash;
use agglayer_types::{Height, LocalNetworkStateData, NetworkId};
use alloy::{
    network::Ethereum,
    primitives::{Bytes, TxHash, B256},
    rpc::types::TransactionReceipt,
};
use mockall::mock;
use pessimistic_proof::{multi_batch_header::MultiBatchHeader, LocalNetworkState};

use crate::{error::CertificationError, Certifier, CertifierOutput};

mock! {
    pub Certifier {}

    #[async_trait::async_trait]
    impl Certifier for Certifier {
        async fn certify(
            &self,
            state: agglayer_types::LocalNetworkStateData,
            network_id: NetworkId,
            height: Height,
        ) -> Result<CertifierOutput, CertificationError>;

        async fn witness_generation(
            &self,
            certificate: &agglayer_types::Certificate,
            state: &mut LocalNetworkStateData,
            certificate_tx_hash: Option<agglayer_types::Digest>,
        ) -> Result<(MultiBatchHeader, LocalNetworkState, pessimistic_proof::PessimisticProofOutput), CertificationError>;
    }
}

mock! {
    pub L1Rpc {}
    #[async_trait::async_trait]
    impl agglayer_contracts::RollupContract for L1Rpc {
        async fn get_trusted_sequencer_address(
            &self,
            rollup_id: u32,
            proof_signers: std::collections::HashMap<u32, agglayer_types::Address>,
        ) -> Result<agglayer_types::Address, L1RpcError>;

        async fn get_rollup_contract_address(&self, rollup_id: u32) -> Result<agglayer_types::Address, L1RpcError>;

        async fn get_l1_info_root(&self, l1_leaf_count: u32) -> Result<[u8; 32], L1RpcError>;
        fn default_l1_info_tree_entry(&self) -> (u32, [u8; 32]);
        async fn get_prev_pessimistic_root(&self, rollup_id: u32, before_tx: Option<TxHash>) -> Result<[u8; 32], L1RpcError>;
        async fn get_verifier_type(&self, rollup_id: u32) -> Result<agglayer_contracts::rollup::VerifierType, L1RpcError>;
        fn get_rollup_manager_address(&self) -> agglayer_types::Address;
        fn get_event_filter_block_range(&self) -> u64;
    }

    #[async_trait::async_trait]
    impl agglayer_contracts::AggchainContract for L1Rpc {
        async fn get_aggchain_vkey_hash(
            &self,
            rollup_address: agglayer_types::Address,
            aggchain_vkey_selector: u16,
        ) -> Result<VKeyHash, L1RpcError>;

        async fn get_aggchain_hash(
            &self,
            rollup_address: agglayer_types::primitives::Address,
            aggchain_data: Bytes,
        ) -> Result<[u8; 32], L1RpcError>;

        async fn get_multisig_context(
            &self,
            rollup_address: agglayer_types::Address,
        ) -> Result<(Vec<agglayer_types::Address>, usize), L1RpcError>;
    }

    #[async_trait::async_trait]
    impl agglayer_contracts::L1TransactionFetcher for L1Rpc {
        type Provider = alloy::providers::RootProvider<Ethereum>;

        async fn fetch_transaction_receipt(&self, tx_hash: B256) -> Result<Option<TransactionReceipt>, L1RpcError>;

        fn get_provider(&self) -> &<Self as agglayer_contracts::L1TransactionFetcher>::Provider;
    }
}
