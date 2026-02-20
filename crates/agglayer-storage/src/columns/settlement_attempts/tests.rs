use rocksdb::{Direction, ReadOptions};
use ulid::Ulid;

use super::SettlementAttemptsColumn;
use crate::{
    schema::Codec as _,
    stores::state::StateStore,
    tests::TempDBDir,
    types::{
        generated::agglayer::storage::v0::{Address, Nonce, SettlementAttempt, TxHash, Uint128},
        settlement::attempt::{Key, Value},
    },
};

#[test]
fn settlement_attempt_roundtrip_codec() {
    let key = Key {
        settlement_job_id: Ulid::from(24u128),
        attempt_sequence_number: 2,
    };
    let value = mk_settlement_attempt(2);

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

#[test]
fn settlement_attempt_key_ordering_is_stable_for_same_job() {
    let tmp = TempDBDir::new();
    let db = StateStore::init_db(tmp.path.as_path()).expect("Unable to init db");

    let settlement_job_id = Ulid::from(99u128);
    for seq in [1u64, 2, 3, 4, 5] {
        let key = Key {
            settlement_job_id,
            attempt_sequence_number: seq,
        };
        db.put::<SettlementAttemptsColumn>(&key, &mk_settlement_attempt(seq))
            .expect("Unable to insert settlement attempt");
    }

    let sequences: Vec<u64> = db
        .iter_with_direction::<SettlementAttemptsColumn>(ReadOptions::default(), Direction::Reverse)
        .expect("Unable to iterate settlement attempts")
        .filter_map(|entry| match entry {
            Ok((key, _)) if key.settlement_job_id == settlement_job_id => {
                Some(key.attempt_sequence_number)
            }
            _ => None,
        })
        .collect();

    assert_eq!(sequences, vec![5, 4, 3, 2, 1]);
}

fn mk_settlement_attempt(seed: u64) -> SettlementAttempt {
    SettlementAttempt {
        sender_wallet: Some(Address {
            address: vec![0x22; 20].into(),
        }),
        nonce: Some(Nonce { nonce: seed }),
        max_fee_per_gas: Some(Uint128 {
            value: vec![0x07; 16].into(),
        }),
        max_priority_fee_per_gas: Some(Uint128 {
            value: vec![0x08; 16].into(),
        }),
        tx_hash: Some(TxHash {
            hash: vec![0x66; 32].into(),
        }),
        submission_time: None,
    }
}
