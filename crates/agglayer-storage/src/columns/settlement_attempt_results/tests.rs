use agglayer_types::SettlementJobId;

use crate::{
    schema::Codec as _,
    types::{
        generated::agglayer::storage::v0::SettlementAttemptResult,
        settlement::{
            attempt,
            attempt_result::{Key, Value},
        },
    },
};

#[test]
fn settlement_attempt_result_roundtrip_codec() {
    let key = attempt::Key {
        settlement_job_id: SettlementJobId::from(7u128),
        attempt_sequence_number: 3,
    };
    let value = SettlementAttemptResult::contract_call_success_for_test(23);

    let encoded_key = key.encode().expect("Unable to encode key");
    let expected_key = [
        key.settlement_job_id.to_be_bytes().as_slice(),
        &3_u64.to_be_bytes(),
    ]
    .concat();
    assert_eq!(encoded_key.len(), Key::LEN);
    assert_eq!(encoded_key, expected_key);

    let decoded_key = Key::decode(&encoded_key).expect("Unable to decode key");
    assert_eq!(decoded_key.settlement_job_id, key.settlement_job_id);
    assert_eq!(
        decoded_key.attempt_sequence_number,
        key.attempt_sequence_number
    );

    let encoded_value = value.encode().expect("Unable to encode value");
    let decoded_value = Value::decode(&encoded_value).expect("Unable to decode value");
    assert_eq!(decoded_value, value);
}
