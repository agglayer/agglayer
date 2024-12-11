use std::{fs::File, io::BufReader};

use base64::{engine::general_purpose::STANDARD, Engine};
use pessimistic_proof::{
    bridge_exit::{BridgeExit, TokenInfo},
    keccak::keccak256,
};
use reth_primitives::U256;
use serde::{de::DeserializeOwned, Deserialize, Deserializer};
use serde_json::Number;

/// Load a data file from the test suite data folder. It is expected to be
/// a json representation of an object of type `T`.
pub fn load_json_data_file<T: DeserializeOwned>(filename: impl AsRef<str>) -> T {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join(filename.as_ref());
    parse_json_file(path)
}

/// Load a json file from the specified path.
pub fn parse_json_file<T>(json_file_path: impl AsRef<std::path::Path>) -> T
where
    T: DeserializeOwned,
{
    let json_file = File::open(json_file_path).unwrap();
    let reader = BufReader::new(json_file);

    serde_json::from_reader(reader).unwrap()
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct BridgeEvent {
    pub removed: bool,
    pub block_number: u64,
    pub transaction_index: u64,
    pub log_index: u64,
    pub transaction_hash: String,
    pub event_type: u8,
    pub event_data: EventData,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum EventData {
    // Mainnet exit root update event
    #[serde(rename_all = "camelCase")]
    UpdateL1InfoTree {
        mainnet_exit_root: [u8; 32],
        rollup_exit_root: [u8; 32],
    },
    // Deposit event
    Deposit(DepositEventData),
    Claim(ClaimEventData),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositEventData {
    pub leaf_type: u8,
    pub origin_network: u32,
    pub origin_address: String,
    pub destination_network: u32,
    pub destination_address: String,
    #[serde(deserialize_with = "u256_from_number")]
    pub amount: U256,
    pub metadata: String,
    pub deposit_count: u32,
}

impl From<DepositEventData> for BridgeExit {
    fn from(deposit_event_data: DepositEventData) -> Self {
        Self {
            leaf_type: deposit_event_data.leaf_type.try_into().unwrap(),
            token_info: TokenInfo {
                origin_network: deposit_event_data.origin_network.into(),
                origin_token_address: deposit_event_data.origin_address.parse().unwrap(),
            },
            dest_network: deposit_event_data.destination_network.into(),
            dest_address: deposit_event_data.destination_address.parse().unwrap(),
            amount: deposit_event_data.amount,
            metadata: STANDARD
                .decode(deposit_event_data.metadata)
                .ok()
                .as_ref()
                .map(|data| keccak256(data)),
        }
    }
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimEventData {
    #[serde(deserialize_with = "u256_from_number")]
    #[serde(rename = "index")]
    pub global_index: U256,
    pub origin_network: u32,
    pub origin_address: String,
    pub destination_address: String,
    #[serde(deserialize_with = "u256_from_number")]
    pub amount: U256,
}

fn u256_from_number<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let n = Number::deserialize(deserializer)?;

    Ok(U256::from_str_radix(n.as_str(), 10).unwrap())
}
