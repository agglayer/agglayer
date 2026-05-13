use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use agglayer_types::{
    Address, Digest, Nonce, SettlementAttempt, SettlementJob, SettlementTxHash, U256,
};
use ulid::Ulid;

use crate::{
    backup::BackupClient,
    columns::{
        settlement_attempt_per_wallet::SettlementAttemptPerWalletColumn,
        settlement_attempt_results::SettlementAttemptResultsColumn,
        settlement_attempts::SettlementAttemptsColumn,
        settlement_job_results::SettlementJobResultsColumn, settlement_jobs::SettlementJobsColumn,
    },
    error::Error,
    stores::{state::StateStore, SettlementReader as _, SettlementWriter as _},
    tests::TempDBDir,
    types::{
        generated::agglayer::storage::v0,
        settlement::{
            attempt::Key as SettlementAttemptKey,
            attempt_per_wallet::{
                Key as SettlementAttemptPerWalletKey, Value as SettlementAttemptPerWalletValue,
            },
        },
    },
};

fn mk_ulid(seed: u128) -> Ulid {
    Ulid::from(seed)
}

fn mk_settlement_job(seed: u8) -> SettlementJob {
    SettlementJob {
        contract_address: Address::from([seed; 20]),
        calldata: vec![seed, seed.wrapping_add(1)].into(),
        eth_value: U256::from_be_bytes([seed; 32]),
        gas_limit: u128::from_be_bytes([seed; 16]),
    }
}

fn mk_settlement_attempt(seed: u64) -> SettlementAttempt {
    SettlementAttempt {
        sender_wallet: Address::from([(seed as u8).wrapping_add(1); 20]),
        nonce: Nonce(seed),
        hash: SettlementTxHash::new(Digest::from([(seed as u8).wrapping_add(4); 32])),
        submission_time: SystemTime::UNIX_EPOCH + Duration::from_secs(seed),
    }
}

fn setup_store() -> (TempDBDir, Arc<crate::storage::DB>, StateStore) {
    let tmp = TempDBDir::new();
    let db = Arc::new(StateStore::init_db(tmp.path.as_path()).expect("Unable to init db"));
    let store = StateStore::new(db.clone(), BackupClient::noop());
    (tmp, db, store)
}

#[test]
fn insert_settlement_job_succeeds_once() {
    let (_tmp, _db, store) = setup_store();
    assert!(store
        .insert_settlement_job(&mk_ulid(1), &mk_settlement_job(1))
        .is_ok());
}

#[test]
fn insert_settlement_job_duplicate_fails() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_ulid(2);
    let first = mk_settlement_job(2);
    let second = mk_settlement_job(3);
    store
        .insert_settlement_job(&job_id, &first)
        .expect("first insert must succeed");
    let res = store.insert_settlement_job(&job_id, &second);
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));
    assert_eq!(
        db.get::<SettlementJobsColumn>(&job_id)
            .expect("Unable to read stored value"),
        Some((&first).into())
    );
}

#[test]
fn insert_settlement_attempt_succeeds_once() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_ulid(3);
    store
        .insert_settlement_job(&job_id, &mk_settlement_job(3))
        .expect("job insert must succeed");
    assert!(store
        .insert_settlement_attempt(&job_id, 1, &mk_settlement_attempt(1))
        .is_ok());
}

#[test]
fn insert_settlement_attempt_duplicate_fails() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_ulid(4);
    let first = mk_settlement_attempt(1);
    let second = mk_settlement_attempt(2);
    store
        .insert_settlement_job(&job_id, &mk_settlement_job(4))
        .expect("job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 1, &first)
        .expect("first insert must succeed");
    let res = store.insert_settlement_attempt(&job_id, 1, &second);
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));
    assert_eq!(
        db.get::<SettlementAttemptsColumn>(&SettlementAttemptKey {
            settlement_job_id: job_id,
            attempt_sequence_number: 1,
        })
        .expect("Unable to read stored value"),
        Some((&first).into())
    );
}

#[test]
fn insert_settlement_attempt_without_job_fails() {
    let (_tmp, _db, store) = setup_store();
    let res = store.insert_settlement_attempt(&mk_ulid(404), 1, &mk_settlement_attempt(1));
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));
}

#[test]
fn insert_settlement_attempt_result_succeeds_once() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_ulid(5);
    store
        .insert_settlement_job(&job_id, &mk_settlement_job(5))
        .expect("job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 1, &mk_settlement_attempt(1))
        .expect("attempt insert must succeed");
    assert!(store
        .insert_settlement_attempt_result(
            &job_id,
            1,
            &v0::SettlementAttemptResult::contract_call_success_for_test(1)
                .try_into()
                .expect("test tx result helper should be decodable"),
        )
        .is_ok());
}

#[test]
fn insert_settlement_attempt_result_duplicate_fails() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_ulid(6);
    let first = v0::SettlementAttemptResult::contract_call_success_for_test(1)
        .try_into()
        .expect("test tx result helper should be decodable");
    let second = v0::SettlementAttemptResult::contract_call_success_for_test(2)
        .try_into()
        .expect("test tx result helper should be decodable");
    store
        .insert_settlement_job(&job_id, &mk_settlement_job(6))
        .expect("job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 1, &mk_settlement_attempt(1))
        .expect("attempt insert must succeed");
    store
        .insert_settlement_attempt_result(&job_id, 1, &first)
        .expect("first insert must succeed");
    let res = store.insert_settlement_attempt_result(&job_id, 1, &second);
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));
    assert_eq!(
        db.get::<SettlementAttemptResultsColumn>(&SettlementAttemptKey {
            settlement_job_id: job_id,
            attempt_sequence_number: 1,
        })
        .expect("Unable to read stored value"),
        Some((&first).into())
    );
}

#[test]
fn insert_settlement_attempt_result_without_attempt_fails() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_ulid(405);
    store
        .insert_settlement_job(&job_id, &mk_settlement_job(42))
        .expect("job insert must succeed");

    let res = store.insert_settlement_attempt_result(
        &job_id,
        1,
        &v0::SettlementAttemptResult::contract_call_success_for_test(1)
            .try_into()
            .expect("test tx result helper should be decodable"),
    );
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));
}

#[test]
fn insert_settlement_attempt_indexes_by_wallet_and_nonce() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_ulid(406);
    let seq = 3;
    let attempt = mk_settlement_attempt(seq);
    let wallet_bytes = attempt.sender_wallet.into_array();
    let nonce = attempt.nonce.0;

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(42))
        .expect("job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, seq, &attempt)
        .expect("attempt insert must succeed");

    let index_key = SettlementAttemptPerWalletKey {
        address: wallet_bytes,
        nonce,
        settlement_job_id: job_id,
        attempt_sequence_number: seq,
    };

    assert!(matches!(
        db.get::<SettlementAttemptPerWalletColumn>(&index_key)
            .expect("index read must succeed"),
        Some(SettlementAttemptPerWalletValue)
    ));
}

#[test]
fn get_settlement_job_returns_none_when_missing() {
    let (_tmp, _db, store) = setup_store();
    assert_eq!(
        store
            .get_settlement_job(&mk_ulid(10))
            .expect("read must succeed"),
        None
    );
}

#[test]
fn get_settlement_job_returns_value_after_insert() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_ulid(11);
    let job = mk_settlement_job(11);
    store
        .insert_settlement_job(&job_id, &job)
        .expect("insert must succeed");
    assert_eq!(
        store
            .get_settlement_job(&job_id)
            .expect("read must succeed"),
        Some(job)
    );
}

#[test]
fn get_settlement_job_result_returns_none_when_missing() {
    let (_tmp, _db, store) = setup_store();
    assert_eq!(
        store
            .get_settlement_job_result(&mk_ulid(12))
            .expect("read must succeed"),
        None
    );
}

#[test]
fn insert_settlement_job_result_without_job_fails() {
    let (_tmp, _db, store) = setup_store();
    let res = store.insert_settlement_job_result(
        &mk_ulid(13),
        &v0::SettlementJobResult::contract_call_success_for_test(13)
            .try_into()
            .expect("test tx result helper should be decodable"),
    );
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));
}

#[test]
fn get_settlement_job_result_returns_value_after_insert() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_ulid(14);
    let result = v0::SettlementJobResult::contract_call_success_for_test(14)
        .try_into()
        .expect("test tx result helper should be decodable");

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(14))
        .expect("job insert must succeed");
    store
        .insert_settlement_job_result(&job_id, &result)
        .expect("result insert must succeed");

    assert_eq!(
        store
            .get_settlement_job_result(&job_id)
            .expect("read must succeed"),
        Some(result)
    );
}

#[test]
fn insert_settlement_job_result_duplicate_fails() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_ulid(15);
    let first = v0::SettlementJobResult::contract_call_success_for_test(15)
        .try_into()
        .expect("test tx result helper should be decodable");
    let second = v0::SettlementJobResult::contract_call_success_for_test(16)
        .try_into()
        .expect("test tx result helper should be decodable");

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(15))
        .expect("job insert must succeed");
    store
        .insert_settlement_job_result(&job_id, &first)
        .expect("first insert must succeed");

    let res = store.insert_settlement_job_result(&job_id, &second);
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));

    assert_eq!(
        db.get::<SettlementJobResultsColumn>(&job_id)
            .expect("Unable to read stored value"),
        Some((&first).into())
    );
}

#[test]
fn job_attempt_result_can_be_read_back_together() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_ulid(21);
    let job = mk_settlement_job(21);
    let attempt = mk_settlement_attempt(5);
    let attempt_result = v0::SettlementAttemptResult::contract_call_success_for_test(21)
        .try_into()
        .expect("test tx result helper should be decodable");
    let job_result = v0::SettlementJobResult::contract_call_success_for_test(22)
        .try_into()
        .expect("test tx result helper should be decodable");

    store
        .insert_settlement_job(&job_id, &job)
        .expect("insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 5, &attempt)
        .expect("insert must succeed");
    store
        .insert_settlement_attempt_result(&job_id, 5, &attempt_result)
        .expect("insert must succeed");
    store
        .insert_settlement_job_result(&job_id, &job_result)
        .expect("insert must succeed");

    assert_eq!(store.get_settlement_job(&job_id).unwrap(), Some(job));
    assert_eq!(
        db.get::<SettlementAttemptsColumn>(&SettlementAttemptKey {
            settlement_job_id: job_id,
            attempt_sequence_number: 5,
        })
        .expect("attempt read must succeed"),
        Some((&attempt).into())
    );
    assert_eq!(
        db.get::<SettlementAttemptResultsColumn>(&SettlementAttemptKey {
            settlement_job_id: job_id,
            attempt_sequence_number: 5,
        })
        .expect("attempt result read must succeed"),
        Some((&attempt_result).into())
    );
    assert_eq!(
        store.get_settlement_job_result(&job_id).unwrap(),
        Some(job_result)
    );
}

#[test]
fn result_absent_does_not_imply_attempt_absent() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_ulid(22);
    let attempt = mk_settlement_attempt(1);
    store
        .insert_settlement_job(&job_id, &mk_settlement_job(22))
        .expect("job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 1, &attempt)
        .expect("insert must succeed");

    assert_eq!(
        db.get::<SettlementAttemptsColumn>(&SettlementAttemptKey {
            settlement_job_id: job_id,
            attempt_sequence_number: 1,
        })
        .expect("attempt read must succeed"),
        Some((&attempt).into())
    );
    assert_eq!(
        db.get::<SettlementAttemptResultsColumn>(&SettlementAttemptKey {
            settlement_job_id: job_id,
            attempt_sequence_number: 1,
        })
        .expect("attempt result read must succeed"),
        None
    );
}

#[test]
fn duplicate_insert_preserves_original_value() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_ulid(23);
    let first = v0::SettlementAttemptResult::contract_call_success_for_test(1)
        .try_into()
        .expect("test tx result helper should be decodable");
    let second = v0::SettlementAttemptResult::contract_call_success_for_test(2)
        .try_into()
        .expect("test tx result helper should be decodable");
    store
        .insert_settlement_job(&job_id, &mk_settlement_job(23))
        .expect("job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 9, &mk_settlement_attempt(9))
        .expect("attempt insert must succeed");

    store
        .insert_settlement_attempt_result(&job_id, 9, &first)
        .expect("first insert must succeed");
    let res = store.insert_settlement_attempt_result(&job_id, 9, &second);
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));

    assert_eq!(
        db.get::<SettlementAttemptResultsColumn>(&SettlementAttemptKey {
            settlement_job_id: job_id,
            attempt_sequence_number: 9,
        })
        .expect("attempt result read must succeed"),
        Some((&first).into())
    );
}
