//! Agglayer smart-contract bindings.

use std::collections::HashMap;

use ethers::prelude::*;
use ethers::providers::Middleware;
use ethers_contract::{ContractCall, ContractError};
use polygon_rollup_manager::{PolygonRollupManagerErrors, RollupIDToRollupDataReturn};


#[rustfmt::skip]
#[allow(warnings)]
pub mod polygon_rollup_manager;

#[rustfmt::skip]
#[allow(warnings)]
pub mod polygon_zk_evm;

#[rustfmt::skip]
#[allow(warnings)]
pub mod polygon_zkevm_global_exit_root_v2;

pub mod settler;

use polygon_zk_evm::PolygonZkEvm;
pub use settler::Settler;

#[async_trait::async_trait]
pub trait RollupContract {
    type M: Middleware;
    async fn get_trusted_sequencer_address(
        &self,
        rollup_id: u32,
        proof_signers: HashMap<u32, Address>,
    ) -> Result<Address, ()>;

    async fn get_l1_info_root(&self, l1_leaf_count: u32) -> Result<[u8; 32], ()>;
}

pub struct L1RpcClient<RpcProvider> {
    inner: polygon_rollup_manager::PolygonRollupManager<RpcProvider>,
    l1_info_tree: polygon_zkevm_global_exit_root_v2::PolygonZkEVMGlobalExitRootV2<RpcProvider>,
}

impl<RpcProvider> L1RpcClient<RpcProvider> {
    pub fn new(
        inner: polygon_rollup_manager::PolygonRollupManager<RpcProvider>,
        l1_info_tree: polygon_zkevm_global_exit_root_v2::PolygonZkEVMGlobalExitRootV2<RpcProvider>,
    ) -> Self {
        Self {
            inner,
            l1_info_tree,
        }
    }
}

#[async_trait::async_trait]
impl<RpcProvider> RollupContract for L1RpcClient<RpcProvider>
where
    RpcProvider: Middleware + 'static,
{
    type M = RpcProvider;

    async fn get_l1_info_root(&self, l1_leaf_count: u32) -> Result<[u8; 32], ()> {
        self.l1_info_tree
            .l_1_info_root_map(l1_leaf_count)
            .await
            .map_err(|_| ())
    }

    async fn get_trusted_sequencer_address(
        &self,
        rollup_id: u32,
        proof_signers: HashMap<u32, Address>,
    ) -> Result<Address, ()> {
        if let Some(addr) = proof_signers.get(&rollup_id) {
            Ok(*addr)
        } else {
            let rollup_data = self
                .inner
                .rollup_id_to_rollup_data(rollup_id)
                .await
                .map_err(|_| ())?;

            let rollup_metadata = RollupIDToRollupDataReturn { rollup_data };
            PolygonZkEvm::new(
                rollup_metadata.rollup_data.rollup_contract,
                self.inner.client().clone(),
            )
            .trusted_sequencer()
            .await
            .map_err(|_| ())
        }
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
