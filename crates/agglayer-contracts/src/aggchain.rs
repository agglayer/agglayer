use agglayer_primitives::Address;
use alloy::primitives::Bytes;
use tracing::error;

use crate::{contracts::AggchainBase, L1RpcClient, L1RpcError};

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
    type M: alloy::providers::Provider;
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
    RpcProvider: alloy::providers::Provider + Clone + 'static,
{
    type M = RpcProvider;

    async fn get_aggchain_vkey_hash(
        &self,
        rollup_address: Address,
        aggchain_vkey_selector: u16,
    ) -> Result<AggchainVkeyHash, L1RpcError> {
        let aggchain_selector = (((aggchain_vkey_selector as u32) << 16) | 1u32).to_be_bytes();

        let client = AggchainBase::new(rollup_address.into(), self.rpc.clone());

        client
            .getAggchainVKey(alloy::primitives::FixedBytes(aggchain_selector))
            .call()
            .await
            .map(|arg0| AggchainVkeyHash::new(arg0.0))
            .map_err(|error| {
                error!(?error, "Unable to fetch the aggchain vkey");

                L1RpcError::AggchainVkeyFetchFailed
            })
    }

    async fn get_aggchain_hash(
        &self,
        rollup_address: Address,
        aggchain_data: Bytes,
    ) -> Result<[u8; 32], L1RpcError> {
        AggchainBase::new(rollup_address.into(), self.rpc.clone())
            .getAggchainHash(aggchain_data)
            .call()
            .await
            .map(Into::into)
            .map_err(|error| {
                error!(?error, "Unable to fetch the aggchain hash");

                L1RpcError::AggchainHashFetchFailed
            })
    }
}
