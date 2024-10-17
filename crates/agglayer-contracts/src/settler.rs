use ethers::providers::Middleware;
use ethers_contract::{ContractCall, ContractError};

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
