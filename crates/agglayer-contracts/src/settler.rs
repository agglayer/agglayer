use alloy::{
    contract::Error as ContractError,
    eips::eip1559::Eip1559Estimation,
    primitives::Bytes,
    providers::{PendingTransactionBuilder, Provider},
};
use tracing::debug;

use crate::{GasPriceParams, L1RpcClient};

const DEFAULT_GAS_PRICE_RETRY_INCREASE_FACTOR: u128 = 120; // 1.2x

#[async_trait::async_trait]
pub trait Settler {
    fn decode_contract_revert(error: &ContractError) -> Option<String>;

    #[allow(clippy::too_many_arguments)]
    async fn verify_pessimistic_trusted_aggregator(
        &self,
        rollup_id: u32,
        l_1_info_tree_leaf_count: u32,
        new_local_exit_root: [u8; 32],
        new_pessimistic_root: [u8; 32],
        proof: Bytes,
        custom_chain_data: Bytes,
        nonce_info: Option<(u64, u64)>, // (previous nonce, number_of_retries)
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

    #[tracing::instrument(skip(self, proof))]
    async fn verify_pessimistic_trusted_aggregator(
        &self,
        rollup_id: u32,
        l_1_info_tree_leaf_count: u32,
        new_local_exit_root: [u8; 32],
        new_pessimistic_root: [u8; 32],
        proof: Bytes,
        custom_chain_data: Bytes,
        nonce_info: Option<(u64, u64)>, // (nonce, number_of_retries)
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

        let tx_call = {
            // Adjust the gas fees based on the configuration.
            let estimate = self.rpc.estimate_eip1559_fees().await?;
            // If nonce info is provided, pass the number of retries to adjust the gas price
            // accordingly (increase it
            // DEFAULT_GAS_PRICE_RETRY_INCREASE_FACTOR/100 per retry)
            let adjusted = adjust_gas_price_estimate(
                &estimate,
                &self.gas_price_params,
                nonce_info.map(|(_, retries)| retries),
            );

            // Set the nonce if provided
            if let Some((nonce, _number_of_retries)) = nonce_info {
                debug!(
                    "Nonce provided, increasing max_fee_per_gas and max_priority_fee_per_gas \
                     {DEFAULT_GAS_PRICE_RETRY_INCREASE_FACTOR} % for rollup_id: {rollup_id}"
                );
                tx_call = tx_call.nonce(nonce);
            }

            debug!(
                "Calculated adjusted gas estimation for rollup_id: {rollup_id}: \
                 max_priority_fee_per_gas: {}, max_fee_per_gas: {}. Nonce info: {:?} Original \
                 estimate: {:?} ",
                adjusted.max_priority_fee_per_gas, adjusted.max_fee_per_gas, nonce_info, estimate
            );

            tx_call
                .max_priority_fee_per_gas(adjusted.max_priority_fee_per_gas)
                .max_fee_per_gas(adjusted.max_fee_per_gas)
        };

        tx_call.send().await
    }
}

fn adjust_gas_price_estimate(
    estimate: &Eip1559Estimation,
    params: &GasPriceParams,
    number_of_retries: Option<u64>,
) -> Eip1559Estimation {
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
    if let Some(retries) = number_of_retries {
        if retries > 0 {
            max_fee_per_gas = max_fee_per_gas
                .saturating_mul(retries as u128 * DEFAULT_GAS_PRICE_RETRY_INCREASE_FACTOR)
                .saturating_div(100);
        }
    }
    if max_fee_per_gas > *ceiling {
        tracing::warn!(
            max_fee_per_gas_estimated = estimate.max_fee_per_gas,
            max_fee_per_gas_adjusted = max_fee_per_gas,
            max_fee_per_gas_ceiling = ceiling,
            "Exceeded configured gas ceiling, clamping",
        );
        max_fee_per_gas = *ceiling;
    }

    let mut max_priority_fee_per_gas = adjust(estimate.max_priority_fee_per_gas);
    if let Some(retries) = number_of_retries {
        if retries > 0 {
            max_priority_fee_per_gas = max_priority_fee_per_gas
                .saturating_mul(retries as u128 * DEFAULT_GAS_PRICE_RETRY_INCREASE_FACTOR)
                .saturating_div(100);
        }
    }
    max_priority_fee_per_gas = max_priority_fee_per_gas.min(*ceiling);

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

    use super::{adjust_gas_price_estimate, GasPriceParams};

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

        let adjusted = adjust_gas_price_estimate(&estimate, &params, None);

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

    #[rstest::rstest]
    fn test_adjust_gas_estimate_with_retries(
        #[values(1, 2, 3, 5)] number_of_retries: u64,
        #[values(1000)] multiplier_per_1000: u64,
        #[values(10_000_000)] floor: u128,
        #[values(500_000_000)] ceiling: u128,
        #[values(50_000_000)] max_fee_per_gas: u128,
        #[values(25_000_000)] max_priority_fee_per_gas: u128,
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

        let adjusted = adjust_gas_price_estimate(&estimate, &params, Some(number_of_retries));

        // Expected values after retry adjustment:
        // max_fee_per_gas should be multiplied by (retries * 1.2)
        // DEFAULT_GAS_PRICE_RETRY_INCREASE_FACTOR = 120
        let expected_max_fee = (max_fee_per_gas
            .saturating_mul(number_of_retries as u128 * 120)
            .saturating_div(100))
        .min(ceiling)
        .max(floor);

        let expected_priority_fee = (max_priority_fee_per_gas
            .saturating_mul(number_of_retries as u128 * 120)
            .saturating_div(100))
        .min(ceiling);

        assert_eq!(
            adjusted.max_fee_per_gas, expected_max_fee,
            "max_fee_per_gas mismatch for {} retries. Expected: {}, Got: {}",
            number_of_retries, expected_max_fee, adjusted.max_fee_per_gas
        );

        assert_eq!(
            adjusted.max_priority_fee_per_gas, expected_priority_fee,
            "max_priority_fee_per_gas mismatch for {} retries. Expected: {}, Got: {}",
            number_of_retries, expected_priority_fee, adjusted.max_priority_fee_per_gas
        );

        // Verify it still respects floor and ceiling
        assert!(
            adjusted.max_fee_per_gas >= floor,
            "max_fee_per_gas {} below floor {}",
            adjusted.max_fee_per_gas,
            floor
        );
        assert!(
            adjusted.max_fee_per_gas <= ceiling,
            "max_fee_per_gas {} exceeds ceiling {}",
            adjusted.max_fee_per_gas,
            ceiling
        );
        assert!(
            adjusted.max_priority_fee_per_gas <= ceiling,
            "max_priority_fee_per_gas {} exceeds ceiling {}",
            adjusted.max_priority_fee_per_gas,
            ceiling
        );
    }

    #[test]
    fn test_adjust_gas_estimate_with_zero_retries() {
        let estimate = Eip1559Estimation {
            max_fee_per_gas: 50_000_000,
            max_priority_fee_per_gas: 25_000_000,
        };
        let params = GasPriceParams {
            multiplier_per_1000: 1000,
            floor: 10_000_000,
            ceiling: 500_000_000,
        };

        // Zero retries should behave the same as None
        let adjusted_with_zero = adjust_gas_price_estimate(&estimate, &params, Some(0));
        let adjusted_with_none = adjust_gas_price_estimate(&estimate, &params, None);

        assert_eq!(
            adjusted_with_zero.max_fee_per_gas, adjusted_with_none.max_fee_per_gas,
            "Zero retries should produce same result as None for max_fee_per_gas"
        );
        assert_eq!(
            adjusted_with_zero.max_priority_fee_per_gas,
            adjusted_with_none.max_priority_fee_per_gas,
            "Zero retries should produce same result as None for max_priority_fee_per_gas"
        );
    }

    #[test]
    fn test_adjust_gas_estimate_retries_hit_ceiling() {
        let estimate = Eip1559Estimation {
            max_fee_per_gas: 50_000_000,
            max_priority_fee_per_gas: 25_000_000,
        };
        let params = GasPriceParams {
            multiplier_per_1000: 1000,
            floor: 10_000_000,
            ceiling: 100_000_000, // Low ceiling to test clamping
        };

        // With 5 retries, the gas price would be 50M * (5 * 1.2) = 300M
        // But it should be clamped to ceiling of 100M
        let adjusted = adjust_gas_price_estimate(&estimate, &params, Some(5));

        assert_eq!(
            adjusted.max_fee_per_gas, params.ceiling,
            "max_fee_per_gas should be clamped to ceiling"
        );
        assert_eq!(
            adjusted.max_priority_fee_per_gas, params.ceiling,
            "max_priority_fee_per_gas should be clamped to ceiling"
        );
    }

    #[test]
    fn test_adjust_gas_estimate_retries_with_multiplier() {
        let estimate = Eip1559Estimation {
            max_fee_per_gas: 50_000_000,
            max_priority_fee_per_gas: 25_000_000,
        };
        let params = GasPriceParams {
            multiplier_per_1000: 1500, // 1.5x multiplier
            floor: 10_000_000,
            ceiling: 500_000_000,
        };

        // With multiplier and 2 retries:
        // max_fee_per_gas: 50M * 1.5 = 75M, then 75M * (2 * 1.2) / 100 = 1.8M (incorrect)
        // Wait, the multiplier is applied first in adjust(), then retries are applied
        // Actually looking at the code:
        // 1. adjust() applies multiplier: 50M * 1500 / 1000 = 75M
        // 2. max(floor): max(75M, 10M) = 75M
        // 3. Then retries: 75M * (2 * 120) / 100 = 75M * 240 / 100 = 180M
        let adjusted = adjust_gas_price_estimate(&estimate, &params, Some(2));

        let base_adjusted = 50_000_000_u128 * 1500 / 1000; // 75M
        let with_retries = base_adjusted * (2 * 120) / 100; // 180M
        assert_eq!(
            adjusted.max_fee_per_gas,
            with_retries.max(params.floor).min(params.ceiling),
            "Should apply both multiplier and retry factor"
        );

        let priority_base = 25_000_000_u128 * 1500 / 1000; // 37.5M
        let priority_with_retries = priority_base * (2 * 120) / 100; // 90M
        assert_eq!(
            adjusted.max_priority_fee_per_gas,
            priority_with_retries.min(params.ceiling),
            "Priority fee should apply both multiplier and retry factor"
        );
    }
}
