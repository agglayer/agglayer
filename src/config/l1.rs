use ethers::types::Address;
use serde::Deserialize;
use url::Url;

/// The L1 configuration.
#[derive(Deserialize, Debug)]
pub(crate) struct L1 {
    #[serde(rename = "ChainID")]
    #[allow(dead_code)]
    pub(crate) chain_id: u64,
    #[serde(rename = "NodeURL")]
    pub(crate) node_url: Url,
    #[serde(rename = "RollupManagerContract")]
    pub(crate) rollup_manager_contract: Address,
}
