use ethers::{providers::Middleware, types::Address};
use tracing::error;

use crate::{aggchain_base::AggchainBase, L1RpcClient, L1RpcError};

#[async_trait::async_trait]
pub trait AggchainContract {
    type M: Middleware;
    async fn get_aggchain_vkey(
        &self,
        rollup_address: Address,
        aggchain_vkey_selector: u16,
    ) -> Result<[u8; 32], L1RpcError>;
}

#[async_trait::async_trait]
impl<RpcProvider> AggchainContract for L1RpcClient<RpcProvider>
where
    RpcProvider: Middleware + 'static,
{
    type M = RpcProvider;

    async fn get_aggchain_vkey(
        &self,
        rollup_address: Address,
        aggchain_vkey_selector: u16,
    ) -> Result<[u8; 32], L1RpcError> {
        let aggchain_selector = (((aggchain_vkey_selector as u32) << 16) | 1u32).to_be_bytes();

        let client = AggchainBase::new(rollup_address, self.rpc.clone());

        client
            .get_aggchain_v_key(aggchain_selector)
            .await
            .map_err(|error| {
                error!("Error fetching aggchain vkey: {:?}", error);

                L1RpcError::AggchainVkeyFetchFailed
            })
    }
}
