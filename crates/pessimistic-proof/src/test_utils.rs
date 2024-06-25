use std::{fs::File, io::BufReader};

use base64::{engine::general_purpose::STANDARD, Engine};
use reth_primitives::U256;
use serde::{Deserialize, Deserializer};
use serde_json::Number;

use crate::{TokenInfo, Withdrawal};

pub fn parse_json_file<T>(json_file_path: &str) -> T
where
    T: for<'de> Deserialize<'de>,
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

#[derive(Debug, Deserialize)]
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

impl From<DepositEventData> for Withdrawal {
    fn from(deposit_event_data: DepositEventData) -> Self {
        Self {
            leaf_type: deposit_event_data.leaf_type,
            token_info: TokenInfo {
                origin_network: deposit_event_data.origin_network.into(),
                origin_token_address: deposit_event_data.origin_address.parse().unwrap(),
            },
            dest_network: deposit_event_data.destination_network.into(),
            dest_address: deposit_event_data.destination_address.parse().unwrap(),
            amount: deposit_event_data.amount,
            metadata: STANDARD.decode(deposit_event_data.metadata).unwrap(),
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
