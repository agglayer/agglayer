use std::time::Duration;

use ethers::types::Address;
use serde::{Deserialize, Serialize};
use url::Url;

/// The L1 configuration.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct L1 {
    pub chain_id: u64,
    pub node_url: Url,
    #[serde(default = "default_ws_node_url")]
    pub ws_node_url: Url,
    #[serde(
        default = "default_max_reconnection_elapsed_time",
        with = "crate::with::HumanDuration"
    )]
    pub max_reconnection_elapsed_time: Duration,
    #[serde(alias = "RollupManagerContract")]
    pub rollup_manager_contract: Address,
    #[serde(alias = "PolygonZkEVMGlobalExitRootV2Contract")]
    pub polygon_zkevm_global_exit_root_v2_contract: Address,
    #[serde(default = "L1::default_rpc_timeout")]
    #[serde(with = "crate::with::HumanDuration")]
    pub rpc_timeout: Duration,
}

impl L1 {
    const fn default_rpc_timeout() -> Duration {
        Duration::from_secs(45)
    }
}

impl Default for L1 {
    fn default() -> Self {
        // Values are coming from https://github.com/0xPolygon/agglayer/blob/main/config/default.go#L11
        Self {
            chain_id: 1337,
            node_url: "http://zkevm-mock-l1-network:8545".parse().unwrap(),
            max_reconnection_elapsed_time: default_max_reconnection_elapsed_time(),
            ws_node_url: default_ws_node_url(),
            rollup_manager_contract: "0xB7f8BC63BbcaD18155201308C8f3540b07f84F5e"
                .parse()
                .unwrap(),
            polygon_zkevm_global_exit_root_v2_contract:
                "0xB7f8BC63BbcaD18155201308C8f3540b07f84F5e"
                    .parse()
                    .unwrap(),
            rpc_timeout: Self::default_rpc_timeout(),
        }
    }
}

fn default_ws_node_url() -> Url {
    "ws://zkevm-mock-l1-network:8546".parse().unwrap()
}

const fn default_max_reconnection_elapsed_time() -> Duration {
    Duration::from_secs(10)
}
