#![allow(unused)]
use alloy::{network::Ethereum, sol};

mod aggchain_base {
    use super::*;
    sol!(
        #[allow(missing_docs)]
        #[allow(clippy::too_many_arguments)]
        #[sol(rpc)]
        #[derive(Debug, Eq, PartialEq)]
        AggchainBase,
        "src/contracts/AggchainBase.json"
    );
}

mod agglayer_gateway {
    use super::*;
    sol!(
        #[allow(missing_docs)]
        #[allow(clippy::too_many_arguments)]
        #[sol(rpc)]
        #[derive(Debug, Eq, PartialEq)]
        AgglayerGateway,
        "src/contracts/AggLayerGateway.json"
    );
}

// Re-export the contracts
pub use aggchain_base::AggchainBase;
// Use the IAggchainSigners from one of the modules to avoid duplication
pub use aggchain_base::IAggchainSigners;
pub use agglayer_gateway::AgglayerGateway;

sol!(
    #[allow(missing_docs)]
    #[allow(clippy::too_many_arguments)]
    #[sol(rpc)]
    #[derive(Debug, Eq, PartialEq)]
    PolygonRollupManager,
    "src/contracts/PolygonRollupManager.json"
);

sol!(
    #[allow(missing_docs)]
    #[allow(clippy::too_many_arguments)]
    #[sol(rpc)]
    #[derive(Debug, Eq, PartialEq)]
    PolygonZkEvm,
    "src/contracts/PolygonZkEVM.json"
);

sol!(
    #[allow(missing_docs)]
    #[allow(clippy::too_many_arguments)]
    #[sol(rpc)]
    #[derive(Debug, Eq, PartialEq)]
    PolygonZkEvmGlobalExitRootV2,
    "src/contracts/PolygonZkEVMGlobalExitRootV2.json"
);

pub(crate) type AggchainBaseRpcClient<RpcProvider> =
    AggchainBase::AggchainBaseInstance<RpcProvider, Ethereum>;

pub(crate) type AgglayerGatewayRpcClient<RpcProvider> =
    AgglayerGateway::AgglayerGatewayInstance<RpcProvider, Ethereum>;

pub(crate) type PolygonRollupManagerRpcClient<RpcProvider> =
    PolygonRollupManager::PolygonRollupManagerInstance<RpcProvider, Ethereum>;

pub(crate) type PolygonZkEvmRpcClient<RpcProvider> =
    PolygonZkEvm::PolygonZkEvmInstance<RpcProvider, Ethereum>;

pub(crate) type PolygonZkEvmGlobalExitRootV2RpcClient<RpcProvider> =
    PolygonZkEvmGlobalExitRootV2::PolygonZkEvmGlobalExitRootV2Instance<RpcProvider, Ethereum>;
