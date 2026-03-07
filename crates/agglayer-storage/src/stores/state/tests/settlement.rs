use std::sync::Arc;

use ulid::Ulid;

use crate::{
    backup::BackupClient,
    columns::{
        settlement_attempt_per_wallet::SettlementAttemptPerWalletColumn,
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
        Some(first)
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
        .insert_settlement_attempt_result(&job_id, 1, &mk_tx_result_success(1))
        .is_ok());
}

#[test]
fn insert_settlement_attempt_result_duplicate_fails() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_ulid(6);
    let first = mk_tx_result_success(1);
    let second = mk_tx_result_success(2);
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
        Some(first)
    );
}

#[test]
fn insert_settlement_attempt_result_without_attempt_fails() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_ulid(405);
    store
        .insert_settlement_job(&job_id, &mk_settlement_job(42))
        .expect("job insert must succeed");

    let res = store.insert_settlement_attempt_result(&job_id, 1, &mk_tx_result_success(1));
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));
}

#[test]
fn insert_settlement_attempt_indexes_by_wallet_and_nonce() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_ulid(406);
    let seq = 3;
    let attempt = mk_settlement_attempt(seq);
    let wallet_bytes: [u8; 20] = attempt
        .sender_wallet
        .as_ref()
        .expect("sender_wallet must be set")
        .address
        .as_ref()
        .try_into()
        .expect("sender wallet should be 20 bytes");
    let nonce = attempt.nonce.as_ref().expect("nonce must be set").nonce;

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
        .insert_settlement_job(&job_id, &mk_settlement_job(13))
        .expect("job insert must succeed");
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
        .insert_settlement_job(&job_id, &mk_settlement_job(15))
        .expect("job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 2, &mk_settlement_attempt(2))
        .expect("attempt insert must succeed");
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
        .insert_settlement_job(&job_id, &mk_settlement_job(22))
        .expect("job insert must succeed");
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
        store
            .get_settlement_attempt_result(&job_id, 9)
            .expect("read must succeed"),
        Some(first)
    );
}

#[test]
fn list_settlement_attempts_returns_empty_when_no_attempts_exist() {
    let (_tmp, _db, store) = setup_store();

    assert_eq!(
        store
            .list_settlement_attempts(&mk_ulid(999))
            .expect("list must succeed"),
        Vec::<(u64, SettlementAttempt)>::new()
    );
}

#[test]
fn list_settlement_attempts_returns_requested_job_attempts_in_sequence_order() {
    let (_tmp, _db, store) = setup_store();
    let job_a = mk_ulid(1_000);
    let job_b = mk_ulid(2_000);

    let attempt_a_1 = mk_settlement_attempt(1);
    let attempt_a_2 = mk_settlement_attempt(2);
    let attempt_b_1 = mk_settlement_attempt(3);

    store
        .insert_settlement_job(&job_a, &mk_settlement_job(10))
        .expect("job A insert must succeed");
    store
        .insert_settlement_job(&job_b, &mk_settlement_job(20))
        .expect("job B insert must succeed");

    store
        .insert_settlement_attempt(&job_a, 2, &attempt_a_2)
        .expect("attempt insert must succeed");
    store
        .insert_settlement_attempt(&job_b, 1, &attempt_b_1)
        .expect("attempt insert must succeed");
    store
        .insert_settlement_attempt(&job_a, 1, &attempt_a_1)
        .expect("attempt insert must succeed");

    let listed = store
        .list_settlement_attempts(&job_a)
        .expect("list must succeed");

    assert_eq!(
        listed,
        vec![(1, attempt_a_1.clone()), (2, attempt_a_2.clone())]
    );
}

#[test]
fn list_settlement_attempt_results_returns_empty_when_no_results_exist() {
    let (_tmp, _db, store) = setup_store();

    assert_eq!(
        store
            .list_settlement_attempt_results(&mk_ulid(1_001))
            .expect("list must succeed"),
        Vec::<(u64, TxResult)>::new()
    );
}

#[test]
fn list_settlement_attempt_results_returns_requested_job_results_in_sequence_order() {
    let (_tmp, _db, store) = setup_store();
    let job_a = mk_ulid(3_000);
    let job_b = mk_ulid(4_000);

    let result_a_1 = mk_tx_result_success(1);
    let result_a_2 = mk_tx_result_success(2);
    let result_b_1 = mk_tx_result_success(3);

    store
        .insert_settlement_job(&job_a, &mk_settlement_job(30))
        .expect("job A insert must succeed");
    store
        .insert_settlement_job(&job_b, &mk_settlement_job(40))
        .expect("job B insert must succeed");

    store
        .insert_settlement_attempt(&job_a, 2, &mk_settlement_attempt(2))
        .expect("attempt insert must succeed");
    store
        .insert_settlement_attempt(&job_b, 1, &mk_settlement_attempt(3))
        .expect("attempt insert must succeed");
    store
        .insert_settlement_attempt(&job_a, 1, &mk_settlement_attempt(1))
        .expect("attempt insert must succeed");

    store
        .insert_settlement_attempt_result(&job_a, 2, &result_a_2)
        .expect("result insert must succeed");
    store
        .insert_settlement_attempt_result(&job_b, 1, &result_b_1)
        .expect("result insert must succeed");
    store
        .insert_settlement_attempt_result(&job_a, 1, &result_a_1)
        .expect("result insert must succeed");

    let listed = store
        .list_settlement_attempt_results(&job_a)
        .expect("list must succeed");

    assert_eq!(listed, vec![(1, result_a_1), (2, result_a_2)]);
}
