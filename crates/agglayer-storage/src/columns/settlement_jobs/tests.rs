use agglayer_types::SettlementJobId;

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
    let key = SettlementJobId::from(42u128);
    let value = mk_settlement_job();

    let encoded_key = key.encode().expect("Unable to encode key");
    assert_eq!(encoded_key.len(), Key::BYTE_LEN);
    assert_eq!(encoded_key, key.to_be_bytes());

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
    }
}
