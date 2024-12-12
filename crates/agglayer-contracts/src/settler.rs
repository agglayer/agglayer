use ethers::{
    providers::{Middleware, PendingTransaction},
    types::H256,
};
use ethers_contract::{ContractCall, ContractError};

#[async_trait::async_trait]
pub trait Settler {
    type M: Middleware;
    async fn transaction_exists(&self, tx_hash: H256) -> Result<bool, String>;
    fn build_pending_transaction(
        &self,
        tx_hash: H256,
    ) -> PendingTransaction<'_, <Self::M as ethers::providers::Middleware>::Provider>;
    fn decode_contract_revert(error: &ContractError<Self::M>) -> Option<String>;
    fn build_verify_pessimistic_trusted_aggregator_call(
        &self,
        rollup_id: u32,
        l_1_info_tree_leaf_count: u32,
        new_local_exit_root: [u8; 32],
        new_pessimistic_root: [u8; 32],
        proof: ::ethers::core::types::Bytes,
    ) -> ContractCall<Self::M, ()>;
}
