use std::sync::Arc;

use ulid::Ulid;

use crate::{
    backup::BackupClient,
    columns::{
        settlement_attempt_results::SettlementAttemptResultsColumn,
        settlement_attempts::SettlementAttemptsColumn, settlement_jobs::SettlementJobsColumn,
    },
    error::Error,
    stores::{state::StateStore, SettlementReader as _, SettlementWriter as _},
    tests::TempDBDir,
    types::{
        generated::agglayer::storage::v0::{
            tx_result, Address, BlockHash, BlockNumber, Calldata, ContractCallMetadata,
            ContractCallOutcome, ContractCallResult, EthValue, Nonce, SettlementAttempt,
            SettlementJob, TxHash, TxResult, Uint128, Uint256,
        },
        settlement::attempt::Key as SettlementAttemptKey,
    },
};

fn mk_ulid(seed: u128) -> Ulid {
    Ulid::from(seed)
}

fn mk_settlement_job(seed: u8) -> SettlementJob {
    SettlementJob {
        contract_address: Some(Address {
            address: vec![seed; 20].into(),
        }),
        calldata: Some(Calldata {
            data: vec![seed, seed.wrapping_add(1)].into(),
        }),
        eth_value: Some(EthValue {
            value: Some(Uint256 {
                value: vec![seed; 32].into(),
            }),
        }),
        gas_limit: Some(Uint128 {
            value: vec![seed; 16].into(),
        }),
        max_fee_per_gas_ceiling: Some(Uint128 {
            value: vec![seed.wrapping_add(1); 16].into(),
        }),
        max_fee_per_gas_floor: Some(Uint128 {
            value: vec![seed.wrapping_add(2); 16].into(),
        }),
        max_fee_per_gas_increase_percents: 10,
        max_priority_fee_per_gas_ceiling: Some(Uint128 {
            value: vec![seed.wrapping_add(3); 16].into(),
        }),
        max_priority_fee_per_gas_floor: Some(Uint128 {
            value: vec![seed.wrapping_add(4); 16].into(),
        }),
        max_priority_fee_per_gas_increase_percents: 20,
    }
}

fn mk_settlement_attempt(seed: u64) -> SettlementAttempt {
    SettlementAttempt {
        sender_wallet: Some(Address {
            address: vec![(seed as u8).wrapping_add(1); 20].into(),
        }),
        nonce: Some(Nonce { nonce: seed }),
        max_fee_per_gas: Some(Uint128 {
            value: vec![(seed as u8).wrapping_add(2); 16].into(),
        }),
        max_priority_fee_per_gas: Some(Uint128 {
            value: vec![(seed as u8).wrapping_add(3); 16].into(),
        }),
        tx_hash: Some(TxHash {
            hash: vec![(seed as u8).wrapping_add(4); 32].into(),
        }),
        submission_time: None,
    }
}

fn mk_tx_result_success(seed: u8) -> TxResult {
    TxResult {
        tx_result: Some(tx_result::TxResult::ContractCallResult(
            ContractCallResult {
                outcome: ContractCallOutcome::Success as i32,
                metadata: Some(ContractCallMetadata {
                    metadata: vec![seed, seed.wrapping_add(1)].into(),
                }),
                block_hash: Some(BlockHash {
                    hash: vec![seed; 32].into(),
                }),
                block_number: Some(BlockNumber {
                    number: seed as u64 + 100,
                }),
                tx_hash: Some(TxHash {
                    hash: vec![seed.wrapping_add(2); 32].into(),
                }),
            },
        )),
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
        Some(first)
    );
}

#[test]
fn insert_settlement_attempt_succeeds_once() {
    let (_tmp, _db, store) = setup_store();
    assert!(store
        .insert_settlement_attempt(&mk_ulid(3), 1, &mk_settlement_attempt(1))
        .is_ok());
}

#[test]
fn insert_settlement_attempt_duplicate_fails() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_ulid(4);
    let first = mk_settlement_attempt(1);
    let second = mk_settlement_attempt(2);
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
        Some(first)
    );
}

#[test]
fn insert_settlement_attempt_result_succeeds_once() {
    let (_tmp, _db, store) = setup_store();
    assert!(store
        .insert_settlement_attempt_result(&mk_ulid(5), 1, &mk_tx_result_success(1))
        .is_ok());
}

#[test]
fn insert_settlement_attempt_result_duplicate_fails() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_ulid(6);
    let first = mk_tx_result_success(1);
    let second = mk_tx_result_success(2);
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
        Some(first)
    );
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
fn get_settlement_attempt_returns_none_when_missing() {
    let (_tmp, _db, store) = setup_store();
    assert_eq!(
        store
            .get_settlement_attempt(&mk_ulid(12), 1)
            .expect("read must succeed"),
        None
    );
}

#[test]
fn get_settlement_attempt_returns_value_after_insert() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_ulid(13);
    let attempt = mk_settlement_attempt(9);
    store
        .insert_settlement_attempt(&job_id, 9, &attempt)
        .expect("insert must succeed");
    assert_eq!(
        store
            .get_settlement_attempt(&job_id, 9)
            .expect("read must succeed"),
        Some(attempt)
    );
}

#[test]
fn get_settlement_attempt_result_returns_none_when_missing() {
    let (_tmp, _db, store) = setup_store();
    assert_eq!(
        store
            .get_settlement_attempt_result(&mk_ulid(14), 1)
            .expect("read must succeed"),
        None
    );
}

#[test]
fn get_settlement_attempt_result_returns_value_after_insert() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_ulid(15);
    let result = mk_tx_result_success(15);
    store
        .insert_settlement_attempt_result(&job_id, 2, &result)
        .expect("insert must succeed");
    assert_eq!(
        store
            .get_settlement_attempt_result(&job_id, 2)
            .expect("read must succeed"),
        Some(result)
    );
}

#[test]
fn latest_attempt_sequence_none_when_no_attempts() {
    let (_tmp, _db, store) = setup_store();
    assert_eq!(
        store
            .get_latest_settlement_attempt_sequence_number(&mk_ulid(16))
            .expect("read must succeed"),
        None
    );
}

#[test]
fn latest_attempt_sequence_returns_max_for_single_job() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_ulid(17);
    for seq in [1u64, 3, 10] {
        store
            .insert_settlement_attempt(&job_id, seq, &mk_settlement_attempt(seq))
            .expect("insert must succeed");
    }
    assert_eq!(
        store
            .get_latest_settlement_attempt_sequence_number(&job_id)
            .expect("read must succeed"),
        Some(10)
    );
}

#[test]
fn latest_attempt_sequence_ignores_other_jobs() {
    let (_tmp, _db, store) = setup_store();
    let job_a = mk_ulid(18);
    let job_b = mk_ulid(19);

    store
        .insert_settlement_attempt(&job_a, 2, &mk_settlement_attempt(2))
        .expect("insert must succeed");
    store
        .insert_settlement_attempt(&job_b, 7, &mk_settlement_attempt(7))
        .expect("insert must succeed");
    store
        .insert_settlement_attempt(&job_a, 9, &mk_settlement_attempt(9))
        .expect("insert must succeed");

    assert_eq!(
        store
            .get_latest_settlement_attempt_sequence_number(&job_a)
            .expect("read must succeed"),
        Some(9)
    );
    assert_eq!(
        store
            .get_latest_settlement_attempt_sequence_number(&job_b)
            .expect("read must succeed"),
        Some(7)
    );
}

#[test]
fn latest_attempt_sequence_with_single_attempt_returns_that_sequence() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_ulid(20);
    store
        .insert_settlement_attempt(&job_id, 42, &mk_settlement_attempt(42))
        .expect("insert must succeed");
    assert_eq!(
        store
            .get_latest_settlement_attempt_sequence_number(&job_id)
            .expect("read must succeed"),
        Some(42)
    );
}

#[test]
fn job_attempt_result_can_be_read_back_together() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_ulid(21);
    let job = mk_settlement_job(21);
    let attempt = mk_settlement_attempt(5);
    let result = mk_tx_result_success(21);

    store
        .insert_settlement_job(&job_id, &job)
        .expect("insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 5, &attempt)
        .expect("insert must succeed");
    store
        .insert_settlement_attempt_result(&job_id, 5, &result)
        .expect("insert must succeed");

    assert_eq!(store.get_settlement_job(&job_id).unwrap(), Some(job));
    assert_eq!(
        store.get_settlement_attempt(&job_id, 5).unwrap(),
        Some(attempt)
    );
    assert_eq!(
        store.get_settlement_attempt_result(&job_id, 5).unwrap(),
        Some(result)
    );
}

#[test]
fn result_absent_does_not_imply_attempt_absent() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_ulid(22);
    let attempt = mk_settlement_attempt(1);
    store
        .insert_settlement_attempt(&job_id, 1, &attempt)
        .expect("insert must succeed");

    assert_eq!(
        store.get_settlement_attempt(&job_id, 1).unwrap(),
        Some(attempt)
    );
    assert_eq!(
        store.get_settlement_attempt_result(&job_id, 1).unwrap(),
        None
    );
}

#[test]
fn duplicate_insert_preserves_original_value() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_ulid(23);
    let first = mk_tx_result_success(1);
    let second = mk_tx_result_success(2);

    store
        .insert_settlement_attempt_result(&job_id, 9, &first)
        .expect("first insert must succeed");
    let res = store.insert_settlement_attempt_result(&job_id, 9, &second);
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));

    assert_eq!(
        store
            .get_settlement_attempt_result(&job_id, 9)
            .expect("read must succeed"),
        Some(first)
    );
}

#[test]
fn latest_attempt_sequence_handles_large_sequence_numbers() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_ulid(24);
    let high = u64::MAX - 1;

    store
        .insert_settlement_attempt(&job_id, high, &mk_settlement_attempt(high))
        .expect("insert must succeed");

    assert_eq!(
        store
            .get_latest_settlement_attempt_sequence_number(&job_id)
            .expect("read must succeed"),
        Some(high)
    );
}
