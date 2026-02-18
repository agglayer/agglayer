use ulid::Ulid;

use crate::{
    schema::Codec as _,
    types::{
        generated::agglayer::storage::v0::{
            tx_result, BlockHash, BlockNumber, ContractCallMetadata, ContractCallOutcome,
            ContractCallResult, TxHash, TxResult,
        },
        settlement::{
            attempt,
            attempt_result::{Key, Value},
        },
    },
};

#[test]
fn settlement_attempt_result_roundtrip_codec() {
    let key = attempt::Key {
        settlement_job_id: Ulid::from(7u128),
        attempt_sequence_number: 3,
    };
    let value = mk_tx_result_success();

    let encoded_key = key.encode().expect("Unable to encode key");
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

fn mk_tx_result_success() -> TxResult {
    TxResult {
        tx_result: Some(tx_result::TxResult::ContractCallResult(
            ContractCallResult {
                outcome: ContractCallOutcome::Success as i32,
                metadata: Some(ContractCallMetadata {
                    metadata: vec![0xab, 0xcd].into(),
                }),
                block_hash: Some(BlockHash {
                    hash: vec![0x44; 32].into(),
                }),
                block_number: Some(BlockNumber { number: 123 }),
                tx_hash: Some(TxHash {
                    hash: vec![0x55; 32].into(),
                }),
            },
        )),
    }
}
