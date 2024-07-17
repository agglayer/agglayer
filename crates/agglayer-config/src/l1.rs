use ethers::types::Address;
use serde::{Deserialize, Serialize};
use url::Url;

/// The L1 configuration.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct L1 {
    #[serde(rename = "ChainID")]
    pub chain_id: u64,
    #[serde(rename = "NodeURL")]
    pub node_url: Url,
    #[serde(rename = "RollupManagerContract")]
    pub rollup_manager_contract: Address,
}

impl Default for L1 {
    fn default() -> Self {
        // Values are coming from https://github.com/0xPolygon/agglayer/blob/main/config/default.go#L11
        Self {
            chain_id: 1337,
            node_url: "http://zkevm-mock-l1-network:8545".parse().unwrap(),
            rollup_manager_contract: "0xB7f8BC63BbcaD18155201308C8f3540b07f84F5e"
                .parse()
                .unwrap(),
        }
    }
}
