use alloy::{
    contract::Error as ContractError,
    eips::eip1559::Eip1559Estimation,
    primitives::Bytes,
    providers::{PendingTransactionBuilder, Provider},
    rpc::types::TransactionRequest,
};
use tracing::debug;

use crate::{GasPriceParams, L1RpcClient};

const DEFAULT_GAS_PRICE_REPEAT_TX_INCREASE_FACTOR: u128 = 150; //1.5X

#[async_trait::async_trait]
pub trait Settler {
    fn decode_contract_revert(error: &ContractError) -> Option<String>;

    async fn build_verify_pessimistic_trusted_aggregator(
        &self,
        rollup_id: u32,
        l_1_info_tree_leaf_count: u32,
        new_local_exit_root: [u8; 32],
        new_pessimistic_root: [u8; 32],
        proof: Bytes,
        custom_chain_data: Bytes,
    ) -> Result<TransactionRequest, ContractError>;

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

    async fn build_verify_pessimistic_trusted_aggregator(
        &self,
        rollup_id: u32,
        l_1_info_tree_leaf_count: u32,
        new_local_exit_root: [u8; 32],
        new_pessimistic_root: [u8; 32],
        proof: Bytes,
        custom_chain_data: Bytes,
    ) -> Result<TransactionRequest, ContractError> {
        let mut tx_call = self.inner.verifyPessimisticTrustedAggregator(
            rollup_id,
            l_1_info_tree_leaf_count,
            new_local_exit_root.into(),
            new_pessimistic_root.into(),
            proof,
            custom_chain_data,
        );

        debug!(
            "Building the L1 settlement tx with calldata: {:?}",
            tx_call.calldata()
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
            // Adjust the gas limit based on the configuration.
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

        #[cfg(feature = "testutils")]
        if fail::eval(
            "notifier::packer::settle_certificate::gas_estimate::zero_gas",
            |_| true,
        )
        .unwrap_or(false)
        {
            tracing::warn!(
                "FAIL POINT ACTIVE: zero gas fail point active for rollup_id: {}",
                rollup_id
            );
            tx_call = tx_call.gas(0);
        }

        Ok(tx_call.into_transaction_request())
    }

    #[tracing::instrument(skip(self, proof))]
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
        let tx = self
            .build_verify_pessimistic_trusted_aggregator(
                rollup_id,
                l_1_info_tree_leaf_count,
                new_local_exit_root,
                new_pessimistic_root,
                proof,
                custom_chain_data,
            )
            .await?;

        Ok(self.rpc.send_transaction(tx).await?)
    }
}

fn adjust_gas_estimate(estimate: &Eip1559Estimation, params: &GasPriceParams) -> Eip1559Estimation {
    let GasPriceParams {
        floor,
        ceiling,
        multiplier_per_1000,
    } = params;

    // Apply gas price multiplier and floor/ceiling constraints
    let adjust = |fee: u128| -> u128 {
        // Multiply by multiplier_per_1000 and divide by 1000
        fee.saturating_mul(*multiplier_per_1000 as u128) / 1000
    };

    let mut max_fee_per_gas = adjust(estimate.max_fee_per_gas).max(*floor);
    if max_fee_per_gas > *ceiling {
        tracing::warn!(
            max_fee_per_gas_estimated = estimate.max_fee_per_gas,
            max_fee_per_gas_adjusted = max_fee_per_gas,
            max_fee_per_gas_ceiling = ceiling,
            "Exceeded configured gas ceiling, clamping",
        );
        max_fee_per_gas = *ceiling;
    }

    let max_priority_fee_per_gas = adjust(estimate.max_priority_fee_per_gas).min(*ceiling);

    let adjusted = Eip1559Estimation {
        max_fee_per_gas,
        max_priority_fee_per_gas,
    };

    if &adjusted != estimate {
        debug!(
            "Applied gas price adjustment. Estimated {}, {} priority. Adjusted to {}, {} priority.",
            estimate.max_fee_per_gas,
            estimate.max_priority_fee_per_gas,
            max_fee_per_gas,
            max_priority_fee_per_gas
        );
    }

    adjusted
}

#[cfg(test)]
mod test {
    use alloy::eips::eip1559::Eip1559Estimation;

    use super::{adjust_gas_estimate, GasPriceParams};

    #[rstest::rstest]
    fn test_adjust_gas_estimate_respects_floor_and_ceiling(
        #[values(500, 1000, 1500, 2000)] multiplier_per_1000: u64,
        #[values(10_000_000, 50_000_000)] floor: u128,
        #[values(100_000_000, 200_000_000)] ceiling: u128,
        #[values(10_000_000, 100_000_000, 200_000_000)] max_fee_per_gas: u128,
        #[values(5_000_000, 50_000_000, 100_000_000)] max_priority_fee_per_gas: u128,
    ) {
        let estimate = Eip1559Estimation {
            max_fee_per_gas,
            max_priority_fee_per_gas,
        };
        let params = GasPriceParams {
            multiplier_per_1000,
            floor,
            ceiling,
        };

        let adjusted = adjust_gas_estimate(&estimate, &params);

        let acceptable_fee = floor..=ceiling;
        assert!(
            acceptable_fee.contains(&adjusted.max_fee_per_gas),
            "max_fee_per_gas {} is out of range {acceptable_fee:?}",
            adjusted.max_fee_per_gas,
        );

        let acceptable_priority_fee = 0..=ceiling;
        assert!(
            acceptable_priority_fee.contains(&adjusted.max_priority_fee_per_gas),
            "max_priority_fee_per_gas {} out of range {acceptable_priority_fee:?}",
            adjusted.max_priority_fee_per_gas,
        );

        // Some extra tests for scaling factor = 1.0
        if multiplier_per_1000 == 1000 {
            let acceptable_fee = [floor, max_fee_per_gas, ceiling];
            assert!(acceptable_fee.contains(&adjusted.max_fee_per_gas));

            let acceptable_priority_fee = [max_priority_fee_per_gas, ceiling];
            assert!(acceptable_priority_fee.contains(&adjusted.max_priority_fee_per_gas));
        }
    }
}
