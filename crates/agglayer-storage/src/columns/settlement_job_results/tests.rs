use agglayer_types::SettlementJobId;

use crate::{
    schema::Codec as _,
    types::{
        generated::agglayer::storage::v0::SettlementJobResult,
        settlement::job_result::{Key, Value},
    },
};

#[test]
fn settlement_job_result_roundtrip_codec() {
    let key = SettlementJobId::from(ulid::Ulid::from(7u128));
    let value = SettlementJobResult::contract_call_success_for_test(23);

    let encoded_key = key.encode().expect("Unable to encode key");
    let decoded_key = Key::decode(&encoded_key).expect("Unable to decode key");
    assert_eq!(decoded_key, key);

    let encoded_value = value.encode().expect("Unable to encode value");
    let decoded_value = Value::decode(&encoded_value).expect("Unable to decode value");
    assert_eq!(decoded_value, value);
}
