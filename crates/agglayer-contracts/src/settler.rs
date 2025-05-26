use ethers::{
    providers::{Middleware, PendingTransaction},
    types::H256,
};
use ethers_contract::{ContractCall, ContractError};

use crate::{polygon_rollup_manager::PolygonRollupManagerErrors, L1RpcClient, L1RpcError};

#[async_trait::async_trait]
pub trait Settler {
    type M: Middleware;
    async fn transaction_exists(&self, tx_hash: H256) -> Result<bool, L1RpcError>;
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
        custom_chain_data: ::ethers::core::types::Bytes,
    ) -> ContractCall<Self::M, ()>;
}

#[async_trait::async_trait]
impl<RpcProvider> Settler for L1RpcClient<RpcProvider>
where
    RpcProvider: Middleware + 'static,
{
    type M = RpcProvider;

    async fn transaction_exists(&self, tx_hash: H256) -> Result<bool, L1RpcError> {
        self.rpc
            .get_transaction(tx_hash)
            .await
            .map_err(|e| L1RpcError::UnableToGetTransaction {
                source: Box::new(anyhow::Error::new(e)),
            })
            .map(|v| v.is_some())
    }

    fn build_pending_transaction(
        &self,
        tx_hash: H256,
    ) -> PendingTransaction<'_, <Self::M as Middleware>::Provider> {
        PendingTransaction::new(tx_hash, self.rpc.as_ref().provider())
    }

    fn decode_contract_revert(error: &ContractError<Self::M>) -> Option<String> {
        error
            .decode_contract_revert::<PolygonRollupManagerErrors>()
            .map(|err| format!("{err:?}"))
    }

    fn build_verify_pessimistic_trusted_aggregator_call(
        &self,
        rollup_id: u32,
        l_1_info_tree_leaf_count: u32,
        new_local_exit_root: [u8; 32],
        new_pessimistic_root: [u8; 32],
        proof: ::ethers::core::types::Bytes,
        custom_chain_data: ::ethers::core::types::Bytes,
    ) -> ContractCall<Self::M, ()> {
        self.inner.verify_pessimistic_trusted_aggregator(
            rollup_id,
            l_1_info_tree_leaf_count,
            new_local_exit_root,
            new_pessimistic_root,
            proof,
            custom_chain_data,
        )
    }
}
