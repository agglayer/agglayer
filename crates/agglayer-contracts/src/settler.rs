use alloy::{
    contract::Error as ContractError,
    primitives::{Bytes, B256},
    providers::Provider,
};

use crate::{L1RpcClient, L1RpcError};

#[async_trait::async_trait]
pub trait Settler {
    async fn transaction_exists(&self, tx_hash: B256) -> Result<bool, L1RpcError>;

    fn decode_contract_revert(error: &ContractError) -> Option<String>;

    async fn build_verify_pessimistic_trusted_aggregator_call(
        &self,
        rollup_id: u32,
        l_1_info_tree_leaf_count: u32,
        new_local_exit_root: [u8; 32],
        new_pessimistic_root: [u8; 32],
        proof: Bytes,
        custom_chain_data: Bytes,
    ) -> Result<(), ContractError>;
}

#[async_trait::async_trait]
impl<RpcProvider> Settler for L1RpcClient<RpcProvider>
where
    RpcProvider: Provider + Clone + 'static,
{
    async fn transaction_exists(&self, tx_hash: B256) -> Result<bool, L1RpcError> {
        self.rpc
            .get_transaction_by_hash(tx_hash)
            .await
            .map_err(|e| L1RpcError::UnableToGetTransaction {
                source: Box::new(anyhow::Error::new(e)),
            })
            .map(|v| v.is_some())
    }

    fn decode_contract_revert(error: &ContractError) -> Option<String> {
        // Try to get raw revert data and decode it manually if the interface method
        // fails
        if let Some(revert_data) = error.as_revert_data() {
            // If specific error decoding fails, try to extract a revert reason string
            if let Some(reason) = alloy::sol_types::decode_revert_reason(revert_data.as_ref()) {
                return Some(reason);
            }

            // Fall back to hex representation of revert data
            return Some(format!("0x{}", hex::encode(revert_data)));
        }

        // For non-revert errors, return the debug representation
        Some(format!("{error:?}"))
    }

    async fn build_verify_pessimistic_trusted_aggregator_call(
        &self,
        rollup_id: u32,
        l_1_info_tree_leaf_count: u32,
        new_local_exit_root: [u8; 32],
        new_pessimistic_root: [u8; 32],
        proof: Bytes,
        custom_chain_data: Bytes,
    ) -> Result<(), ContractError> {
        self.inner
            .verifyPessimisticTrustedAggregator(
                rollup_id,
                l_1_info_tree_leaf_count,
                new_local_exit_root.into(),
                new_pessimistic_root.into(),
                proof,
                custom_chain_data,
            )
            .call()
            .await
            .map(|_| ())
    }
}
