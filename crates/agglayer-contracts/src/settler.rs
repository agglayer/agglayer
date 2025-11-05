use alloy::{
    contract::Error as ContractError,
    eips::eip1559::Eip1559Estimation,
    primitives::Bytes,
    providers::{PendingTransactionBuilder, Provider},
};
use tracing::debug;

use crate::{adjust_gas_estimate, L1RpcClient};

const DEFAULT_GAS_PRICE_REPEAT_TX_INCREASE_FACTOR: u128 = 150; //1.5X

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
        nonce_info: Option<(u64, u128, Option<u128>)>, /* nonce, previous_max_fee_per_gas,
                                                        * optional previous_max_priority_fee_per_gas */
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
        nonce_info: Option<(u64, u128, Option<u128>)>, /* nonce, previous_max_fee_per_gas,
                                                        * optional previous_max_priority_fee_per_gas */
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
            let estimate = self.rpc.estimate_eip1559_fees().await?;

            let adjusted_fees =
                if let Some((nonce, previous_max_fee_per_gas, previous_max_priority_fee_per_gas)) =
                    nonce_info
                {
                    // This is repeated transaction, increase the previous max_fee_per_gas and
                    // max_priority_fee_per_gas by a factor
                    // If previous_max_priority_fee_per_gas is None, set it to estimated.
                    let adjust = Eip1559Estimation {
                        max_fee_per_gas: {
                            let mut new_max_fee_per_gas = previous_max_fee_per_gas
                                .saturating_mul(DEFAULT_GAS_PRICE_REPEAT_TX_INCREASE_FACTOR)
                                .div_ceil(100)
                                .max(self.gas_price_params.floor)
                                .min(self.gas_price_params.ceiling);
                            // In the corner case that the previous fee is the same as the new fee
                            // due to rounding, multiply it by 2 to
                            // ensure progress
                            if new_max_fee_per_gas == previous_max_fee_per_gas {
                                new_max_fee_per_gas =
                                    (new_max_fee_per_gas * 2).min(self.gas_price_params.ceiling);
                            }
                            new_max_fee_per_gas
                        },
                        max_priority_fee_per_gas: previous_max_priority_fee_per_gas
                            .map(|previous| {
                                let mut new_max_priority_fee_per_gas = previous
                                    .saturating_mul(DEFAULT_GAS_PRICE_REPEAT_TX_INCREASE_FACTOR)
                                    .div_ceil(100);
                                // In the corner case that the previous priority fee is the same as
                                // the new fee due to rounding,
                                // multiply it by 2 to ensure progress
                                if new_max_priority_fee_per_gas == previous {
                                    new_max_priority_fee_per_gas *= 2;
                                }
                                new_max_priority_fee_per_gas
                            })
                            .unwrap_or(estimate.max_priority_fee_per_gas)
                            .max(self.gas_price_params.floor)
                            .min(self.gas_price_params.ceiling),
                    };
                    debug!(
                        provided_nonce_info = ?nonce_info,
                        adjusted_max_fees = ?adjust,
                        %rollup_id,
                        "Nonce provided, increasing  previous max_fee_per_gas and \
                         max_priority_fee_per_gas"
                    );
                    // Set the nonce for the transaction
                    tx_call = tx_call.nonce(nonce);
                    adjust
                } else {
                    // Adjust the gas fees based on the configuration.
                    adjust_gas_estimate(&estimate, &self.gas_price_params)
                };

            tx_call
                .max_priority_fee_per_gas(adjusted_fees.max_priority_fee_per_gas)
                .max_fee_per_gas(adjusted_fees.max_fee_per_gas)
        };

        tx_call.send().await
    }
}
