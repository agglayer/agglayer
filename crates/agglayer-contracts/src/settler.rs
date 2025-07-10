use alloy::{
    contract::Error as ContractError,
    primitives::Bytes,
    providers::{PendingTransactionBuilder, Provider},
};
use tracing::debug;

use crate::L1RpcClient;

#[async_trait::async_trait]
pub trait Settler {
    fn decode_contract_revert(error: &ContractError) -> Option<String>;

    async fn verify_pessimistic_trusted_aggregator(
        &self,
        rollup_id: u32,
        l_1_info_tree_leaf_count: u32,
        new_local_exit_root: [u8; 32],
        new_pessimistic_root: [u8; 32],
        proof: Bytes,
        custom_chain_data: Bytes,
    ) -> Result<PendingTransactionBuilder<alloy::network::Ethereum>, ContractError>;
}

#[async_trait::async_trait]
impl<RpcProvider> Settler for L1RpcClient<RpcProvider>
where
    RpcProvider: Provider + Clone + 'static,
{
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

    async fn verify_pessimistic_trusted_aggregator(
        &self,
        rollup_id: u32,
        l_1_info_tree_leaf_count: u32,
        new_local_exit_root: [u8; 32],
        new_pessimistic_root: [u8; 32],
        proof: Bytes,
        custom_chain_data: Bytes,
    ) -> Result<PendingTransactionBuilder<alloy::network::Ethereum>, ContractError> {
        // Build the transaction call
        let mut tx_call = self.inner.verifyPessimisticTrustedAggregator(
            rollup_id,
            l_1_info_tree_leaf_count,
            new_local_exit_root.into(),
            new_pessimistic_root.into(),
            proof,
            custom_chain_data,
        );

        // This is a fail point for testing purposes, it simulates low gas conditions.
        // Check if the low gas fail point is active and set the low gas if it is.
        #[cfg(feature = "testutils")]
        if fail::eval(
            "notifier::packer::settle_certificate::gas_estimate::low_gas",
            |_| true,
        )
        .unwrap_or(false)
        {
            tracing::warn!(
                "FAIL POINT ACTIVE: low gas fail point active for rollup_id: {}",
                rollup_id
            );
            tx_call = tx_call.gas(30000);
        }

        // Check if a gas multiplier factor is provided
        if self.gas_multiplier_factor != 100 {
            // Apply gas multiplier if it's not the default (100).
            // First estimate gas, then multiply by the factor.
            let gas_estimate = tx_call.estimate_gas().await?;
            let adjusted_gas =
                (gas_estimate.saturating_mul(self.gas_multiplier_factor as u64)) / 100;
            debug!(
                "Applying gas multiplier factor: {} for rollup_id: {}. Estimated gas: {}, \
                 Adjusted gas: {}",
                self.gas_multiplier_factor, rollup_id, gas_estimate, adjusted_gas
            );
            tx_call = tx_call.gas(adjusted_gas);
        }

        tx_call.send().await
    }
}
