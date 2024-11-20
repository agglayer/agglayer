use std::collections::HashMap;

use async_trait::async_trait;
use ethers::{
    contract::{ContractCall, ContractError},
    providers::MockProvider,
};
pub use ethers::{middleware::NonceManagerMiddleware, prelude::Address, providers::Provider};

use super::{RollupContract, Settler};

mockall::mock! {
    pub L1Rpc {}

    #[async_trait]
    impl RollupContract for L1Rpc {
        type M = NonceManagerMiddleware<Provider<MockProvider>>;

        async fn get_trusted_sequencer_address(
            &self,
            rollup_id: u32,
            proof_signers: HashMap<u32, Address>,
        ) -> Result<Address, ()>;
    }

    impl Settler for L1Rpc {
        type M = NonceManagerMiddleware<Provider<MockProvider>>;

        fn decode_contract_revert(error: &ContractError<NonceManagerMiddleware<Provider<MockProvider>>>) -> Option<String>;
        fn build_verify_pessimistic_trusted_aggregator_call(
            &self,
            rollup_id: u32,
            l_1_info_tree_leaf_count: u32,
            new_local_exit_root: [u8; 32],
            new_pessimistic_root: [u8; 32],
            proof: ::ethers::core::types::Bytes,
        ) -> ContractCall<NonceManagerMiddleware<Provider<MockProvider>>, ()>;
    }
}
