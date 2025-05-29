use ethers::{
    providers::Middleware,
    types::{Address, Bytes},
};
use tracing::error;

use crate::{aggchain_base::AggchainBase, L1RpcClient, L1RpcError};

#[derive(PartialEq, Eq)]
pub struct AggchainVkeyHash([u8; 32]);

impl AggchainVkeyHash {
    pub fn new(vkey: [u8; 32]) -> Self {
        Self(vkey)
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

#[async_trait::async_trait]
pub trait AggchainContract {
    type M: Middleware;
    async fn get_aggchain_vkey_hash(
        &self,
        rollup_address: Address,
        aggchain_vkey_selector: u16,
    ) -> Result<AggchainVkeyHash, L1RpcError>;

    async fn get_aggchain_hash(
        &self,
        rollup_address: Address,
        aggchain_data: Bytes,
    ) -> Result<[u8; 32], L1RpcError>;
}

#[async_trait::async_trait]
impl<RpcProvider> AggchainContract for L1RpcClient<RpcProvider>
where
    RpcProvider: Middleware + 'static,
{
    type M = RpcProvider;

    async fn get_aggchain_vkey_hash(
        &self,
        rollup_address: Address,
        aggchain_vkey_selector: u16,
    ) -> Result<AggchainVkeyHash, L1RpcError> {
        let aggchain_selector = (((aggchain_vkey_selector as u32) << 16) | 1u32).to_be_bytes();

        let client = AggchainBase::new(rollup_address, self.rpc.clone());

        client
            .get_aggchain_v_key(aggchain_selector)
            .await
            .map(AggchainVkeyHash)
            .map_err(|error| {
                error!("Error fetching aggchain vkey: {:?}", error);

                L1RpcError::AggchainVkeyFetchFailed
            })
    }

    async fn get_aggchain_hash(
        &self,
        rollup_address: Address,
        aggchain_data: Bytes,
    ) -> Result<[u8; 32], L1RpcError> {
        AggchainBase::new(rollup_address, self.rpc.clone())
            .get_aggchain_hash(aggchain_data)
            .await
            .map_err(|error| {
                error!("Error fetching aggchain hash: {:?}", error);

                L1RpcError::AggchainHashFetchFailed
            })
    }
}
