use ulid::Ulid;

use crate::{
    schema::Codec as _,
    types::{
        generated::agglayer::storage::v0::{
            Address, Calldata, EthValue, SettlementJob, Uint128, Uint256,
        },
        settlement::job::{Key, Value},
    },
};

#[test]
fn settlement_job_roundtrip_codec() {
    let key = Ulid::from(42u128);
    let value = mk_settlement_job();

    let encoded_key = key.encode().expect("Unable to encode key");
    let decoded_key = Key::decode(&encoded_key).expect("Unable to decode key");
    assert_eq!(decoded_key, key);

    let encoded_value = value.encode().expect("Unable to encode value");
    let decoded_value = Value::decode(&encoded_value).expect("Unable to decode value");
    assert_eq!(decoded_value, value);
}

fn mk_settlement_job() -> SettlementJob {
    SettlementJob {
        contract_address: Some(Address {
            address: vec![0x11; 20].into(),
        }),
        calldata: Some(Calldata {
            data: vec![0xaa, 0xbb, 0xcc].into(),
        }),
        eth_value: Some(EthValue {
            value: Some(Uint256 {
                value: vec![0x01; 32].into(),
            }),
        }),
        gas_limit: Some(Uint128 {
            value: vec![0x02; 16].into(),
        }),
        max_fee_per_gas_ceiling: Some(Uint128 {
            value: vec![0x03; 16].into(),
        }),
        max_fee_per_gas_floor: Some(Uint128 {
            value: vec![0x04; 16].into(),
        }),
        max_fee_per_gas_increase_percents: 10,
        max_priority_fee_per_gas_ceiling: Some(Uint128 {
            value: vec![0x05; 16].into(),
        }),
        max_priority_fee_per_gas_floor: Some(Uint128 {
            value: vec![0x06; 16].into(),
        }),
        max_priority_fee_per_gas_increase_percents: 20,
    }
}
