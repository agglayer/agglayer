//! Agglayer smart-contract bindings.

use ethers::providers::Middleware;
use ethers_contract::{ContractCall, ContractError};
use polygon_rollup_manager::PolygonRollupManagerErrors;

#[rustfmt::skip]
#[allow(warnings)]
pub mod polygon_rollup_manager;

#[rustfmt::skip]
#[allow(warnings)]
pub mod polygon_zk_evm;

pub trait Settler {
    type M: Middleware;

    fn decode_contract_revert(error: &ContractError<Self::M>) -> Option<String>;
    fn build_verify_pessimistic_trusted_aggregator_call(
        &self,
        rollup_id: u32,
        l_1_info_tree_leaf_count: u32,
        new_local_exit_root: [u8; 32],
        new_pessimistic_root: [u8; 32],
        proof: ::ethers::core::types::Bytes,
    ) -> ContractCall<Self::M, ()>;
}

pub struct L1RpcClient<RpcProvider> {
    inner: polygon_rollup_manager::PolygonRollupManager<RpcProvider>,
}

impl<RpcProvider> L1RpcClient<RpcProvider> {
    pub fn new(inner: polygon_rollup_manager::PolygonRollupManager<RpcProvider>) -> Self {
        Self { inner }
    }
}

impl<RpcProvider> Settler for L1RpcClient<RpcProvider>
where
    RpcProvider: Middleware + 'static,
{
    type M = RpcProvider;

    fn decode_contract_revert(error: &ContractError<Self::M>) -> Option<String> {
        error
            .decode_contract_revert::<PolygonRollupManagerErrors>()
            .map(|err| format!("{:?}", err))
    }

    fn build_verify_pessimistic_trusted_aggregator_call(
        &self,
        rollup_id: u32,
        l_1_info_tree_leaf_count: u32,
        new_local_exit_root: [u8; 32],
        new_pessimistic_root: [u8; 32],
        proof: ::ethers::core::types::Bytes,
    ) -> ContractCall<Self::M, ()> {
        self.inner.verify_pessimistic_trusted_aggregator(
            rollup_id,
            l_1_info_tree_leaf_count,
            new_local_exit_root,
            new_pessimistic_root,
            proof,
        )
    }
}
