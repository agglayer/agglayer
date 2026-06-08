pub use agglayer_primitives::vkey_hash::VKeyHash;
use agglayer_primitives::{Address, U256};
use alloy::{
    eips::BlockId,
    primitives::{Bytes, TxHash},
};
use tracing::error;

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
        let aggchain_selector = (((aggchain_vkey_selector as u32) << 16) | 1u32).to_be_bytes();

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
