pub use agglayer_primitives::vkey_hash::VKeyHash;
use agglayer_primitives::{Address, U256};
use alloy::{
    eips::BlockId,
    primitives::{Bytes, TxHash},
};
use tracing::{error, info};

use crate::{block_pinning::block_before_tx, contracts::AggchainBase, L1RpcClient, L1RpcError};

#[async_trait::async_trait]
pub trait AggchainContract {
    async fn get_aggchain_vkey_hash(
        &self,
        rollup_address: Address,
        aggchain_vkey_selector: u16,
    ) -> Result<VKeyHash, L1RpcError>;

    /// Fetch the aggchain hash for `aggchain_data` from the rollup's aggchain
    /// contract on L1.
    ///
    /// When `before_tx_hash` is `Some`, the call is pinned to the L1 block
    /// immediately preceding that transaction's inclusion block. This is used
    /// to reconcile an already-settled certificate: stateful aggchain
    /// contracts (e.g. `AggchainFEP`) revert `getAggchainHash` once their
    /// `nextBlockNumber` has advanced past the certificate's range, so the
    /// hash must be queried at the pre-settlement state where it is still
    /// served. When `None`, or when the transaction is not yet mined
    /// successfully, the query targets `latest`.
    async fn get_aggchain_hash(
        &self,
        rollup_address: Address,
        aggchain_data: Bytes,
        before_tx_hash: Option<TxHash>,
    ) -> Result<[u8; 32], L1RpcError>;

    async fn get_multisig_context(
        &self,
        rollup_address: Address,
    ) -> Result<(Vec<Address>, usize), L1RpcError>;
}

/// Builds the aggchain vkey selector `[version (2B) | AGGCHAIN_TYPE (2B)]`.
fn build_aggchain_selector(aggchain_vkey_version: u16, aggchain_type: u16) -> [u8; 4] {
    (((aggchain_vkey_version as u32) << 16) | aggchain_type as u32).to_be_bytes()
}

impl<RpcProvider> L1RpcClient<RpcProvider>
where
    RpcProvider: alloy::providers::Provider + Clone + 'static,
{
    /// Returns the `AGGCHAIN_TYPE` of the rollup's aggchain contract, fetching
    /// it from L1 on first use and caching it per rollup address.
    async fn get_aggchain_type(&self, rollup_address: Address) -> Result<u16, L1RpcError> {
        let key: alloy::primitives::Address = rollup_address.into();

        if let Some(cached) = self
            .aggchain_types
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get(&key)
        {
            return Ok(*cached);
        }

        let aggchain_type = AggchainBase::new(key, self.rpc.clone())
            .AGGCHAIN_TYPE()
            .call()
            .await
            .map(|bytes| u16::from_be_bytes(bytes.0))
            .map_err(|error| {
                error!(?error, %rollup_address, "Unable to fetch the aggchain type");

                L1RpcError::AggchainVkeyFetchFailed
            })?;

        info!(
            %rollup_address,
            aggchain_type = format_args!("{aggchain_type:#06x}"),
            "Fetched AGGCHAIN_TYPE from rollup contract"
        );

        self.aggchain_types
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .insert(key, aggchain_type);

        Ok(aggchain_type)
    }
}

#[async_trait::async_trait]
impl<RpcProvider> AggchainContract for L1RpcClient<RpcProvider>
where
    RpcProvider: alloy::providers::Provider + Clone + 'static,
{
    async fn get_aggchain_vkey_hash(
        &self,
        rollup_address: Address,
        aggchain_vkey_selector: u16,
    ) -> Result<VKeyHash, L1RpcError> {
        let aggchain_type = self.get_aggchain_type(rollup_address).await?;
        let aggchain_selector = build_aggchain_selector(aggchain_vkey_selector, aggchain_type);

        let client = AggchainBase::new(rollup_address.into(), self.rpc.clone());

        client
            .getAggchainVKey(alloy::primitives::FixedBytes(aggchain_selector))
            .call()
            .await
            .map(VKeyHash::from)
            .map_err(|error| {
                error!(?error, "Unable to fetch the aggchain vkey");

                L1RpcError::AggchainVkeyFetchFailed
            })
    }

    async fn get_aggchain_hash(
        &self,
        rollup_address: Address,
        aggchain_data: Bytes,
        before_tx_hash: Option<TxHash>,
    ) -> Result<[u8; 32], L1RpcError> {
        let at_block = match before_tx_hash {
            // A transaction that did not successfully advance the state we depend
            // on (not mined, reverted, or whose receipt could not be fetched)
            // leaves the current state as the one to query.
            Some(tx_hash) => block_before_tx(&self.rpc, tx_hash)
                .await
                .unwrap_or_else(|_| BlockId::latest()),
            None => BlockId::latest(),
        };

        AggchainBase::new(rollup_address.into(), self.rpc.clone())
            .getAggchainHash(aggchain_data)
            .block(at_block)
            .call()
            .await
            .map(Into::into)
            .map_err(|error| {
                error!(?error, ?at_block, "Unable to fetch the aggchain hash");

                L1RpcError::AggchainHashFetchFailed
            })
    }

    async fn get_multisig_context(
        &self,
        rollup_address: Address,
    ) -> Result<(Vec<Address>, usize), L1RpcError> {
        let client = AggchainBase::new(rollup_address.into(), self.rpc.clone());

        let signers = client
            .getAggchainSigners()
            .call()
            .await
            .map(|alloy_vec| alloy_vec.into_iter().map(Address::from_alloy).collect())
            .map_err(L1RpcError::MultisigSignersFetchFailed)?;

        let threshold: usize = {
            let threshold_u256: U256 = client
                .getThreshold()
                .call()
                .await
                .map_err(L1RpcError::MultisigThresholdFetchFailed)?;

            threshold_u256
                .try_into()
                .map_err(|_| L1RpcError::ThresholdTypeOverflow {
                    fetched: threshold_u256,
                })?
        };

        Ok((signers, threshold))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use alloy::{
        primitives::B256,
        providers::{mock::Asserter, ProviderBuilder},
    };

    use super::*;
    use crate::{contracts::PolygonRollupManager, GasPriceParams};

    type AlloyAddress = alloy::primitives::Address;

    fn mock_client(asserter: Asserter) -> L1RpcClient<impl alloy::providers::Provider + Clone> {
        let rpc = ProviderBuilder::new().connect_mocked_client(asserter);
        L1RpcClient::new(
            Arc::new(rpc.clone()),
            PolygonRollupManager::new(AlloyAddress::ZERO, rpc),
            AlloyAddress::ZERO,
            (0, [0; 32]),
            100,
            GasPriceParams::default(),
            10_000,
        )
    }

    /// ABI-encoded `bytes2` return value (left-aligned in a 32-byte word).
    fn bytes2_word(value: u16) -> B256 {
        let mut word = [0u8; 32];
        word[..2].copy_from_slice(&value.to_be_bytes());
        B256::from(word)
    }

    #[test]
    fn selector_for_qbft_type() {
        assert_eq!(build_aggchain_selector(1, 0x0002), [0x00, 0x01, 0x00, 0x02]);
        assert_eq!(
            u32::from_be_bytes(build_aggchain_selector(1, 2)),
            0x0001_0002
        );
    }

    #[test]
    fn selector_for_fep_type_is_unchanged() {
        assert_eq!(build_aggchain_selector(1, 0x0001), [0x00, 0x01, 0x00, 0x01]);
        assert_eq!(build_aggchain_selector(0, 0x0001), [0x00, 0x00, 0x00, 0x01]);
    }

    #[test_log::test(tokio::test)]
    async fn aggchain_type_is_fetched_once_and_cached() {
        let asserter = Asserter::new();
        let client = mock_client(asserter.clone());
        let rollup = Address::from(AlloyAddress::repeat_byte(7));

        asserter.push_success(&bytes2_word(0x0002));
        assert_eq!(client.get_aggchain_type(rollup).await.unwrap(), 0x0002);

        // Second call must be served from the cache: the mock has no responses
        // left, so any RPC call would fail.
        assert_eq!(client.get_aggchain_type(rollup).await.unwrap(), 0x0002);
        assert!(asserter.read_q().is_empty());

        // A different rollup address is fetched independently.
        let other = Address::from(AlloyAddress::repeat_byte(8));
        asserter.push_success(&bytes2_word(0x0001));
        assert_eq!(client.get_aggchain_type(other).await.unwrap(), 0x0001);
        assert_eq!(client.get_aggchain_type(rollup).await.unwrap(), 0x0002);
    }

    #[test_log::test(tokio::test)]
    async fn aggchain_type_fetch_failure_is_not_cached() {
        let asserter = Asserter::new();
        let client = mock_client(asserter.clone());
        let rollup = Address::from(AlloyAddress::repeat_byte(7));

        asserter.push_failure_msg("boom");
        assert!(client.get_aggchain_type(rollup).await.is_err());

        asserter.push_success(&bytes2_word(0x0002));
        assert_eq!(client.get_aggchain_type(rollup).await.unwrap(), 0x0002);
    }
}
