use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use agglayer_types::{
    Address, CertificateId, ClientError, ClientErrorType, Digest, Nonce, SettlementAttempt,
    SettlementAttemptResult, SettlementJob, SettlementJobId, SettlementTxHash, U256,
};

use crate::{
    backup::BackupClient,
    columns::{
        settlement_attempt_per_wallet::SettlementAttemptPerWalletColumn,
        settlement_attempt_results::SettlementAttemptResultsColumn,
        settlement_attempts::SettlementAttemptsColumn,
        settlement_job_id_per_certificate_id::SettlementJobIdPerCertificateIdColumn,
        settlement_job_results::SettlementJobResultsColumn, settlement_jobs::SettlementJobsColumn,
        SETTLEMENT_ATTEMPTS_CF,
    },
    error::Error,
    stores::{
        state::StateStore, SettlementReader as _, SettlementWriter as _, StateReader as _,
        StateWriter as _,
    },
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

fn mk_job_id(seed: u128) -> SettlementJobId {
    SettlementJobId::from(seed)
}

fn mk_certificate_id(seed: u8) -> CertificateId {
    CertificateId::new(Digest::from([seed; 32]))
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
        max_fee_per_gas: 30_000_000_000 + seed as u128,
        max_priority_fee_per_gas: 1_000_000_000 + seed as u128,
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
        .insert_settlement_job(&mk_job_id(1), &mk_settlement_job(1))
        .is_ok());
}

#[test]
fn insert_settlement_job_duplicate_fails() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_job_id(2);
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
fn list_settlement_job_ids_returns_all_jobs_in_key_order() {
    let (_tmp, _db, store) = setup_store();
    let first_job_id = mk_job_id(1);
    let second_job_id = mk_job_id(2);
    let third_job_id = mk_job_id(3);

    store
        .insert_settlement_job(&third_job_id, &mk_settlement_job(3))
        .expect("third job insert must succeed");
    store
        .insert_settlement_job(&first_job_id, &mk_settlement_job(1))
        .expect("first job insert must succeed");
    store
        .insert_settlement_job(&second_job_id, &mk_settlement_job(2))
        .expect("second job insert must succeed");

    assert_eq!(
        store
            .list_settlement_job_ids()
            .expect("job id scan must succeed"),
        vec![first_job_id, second_job_id, third_job_id]
    );
}

#[test]
fn get_certificate_settlement_job_id_returns_none_when_missing() {
    let (_tmp, _db, store) = setup_store();

    assert_eq!(
        store
            .get_certificate_settlement_job_id(&mk_certificate_id(1))
            .expect("read must succeed"),
        None
    );
}

#[test]
fn insert_certificate_settlement_job_id_allows_missing_job() {
    let (_tmp, db, store) = setup_store();
    let certificate_id = mk_certificate_id(2);
    let job_id = mk_job_id(200);

    store
        .insert_certificate_settlement_job_id(&certificate_id, &job_id)
        .expect("mapping insert must not require an existing job");

    assert_eq!(
        db.get::<SettlementJobIdPerCertificateIdColumn>(&certificate_id)
            .expect("Unable to read stored value"),
        Some(job_id)
    );

    assert_eq!(
        db.get::<SettlementJobsColumn>(&job_id)
            .expect("Unable to read stored value"),
        None
    );
}

#[test]
fn insert_certificate_settlement_job_id_returns_value_after_insert() {
    let (_tmp, _db, store) = setup_store();
    let certificate_id = mk_certificate_id(3);
    let job_id = mk_job_id(300);

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(3))
        .expect("job insert must succeed");
    store
        .insert_certificate_settlement_job_id(&certificate_id, &job_id)
        .expect("mapping insert must succeed");

    assert_eq!(
        store
            .get_certificate_settlement_job_id(&certificate_id)
            .expect("read must succeed"),
        Some(job_id)
    );
}

#[test]
fn insert_certificate_settlement_job_id_duplicate_fails() {
    let (_tmp, db, store) = setup_store();
    let certificate_id = mk_certificate_id(4);
    let first_job_id = mk_job_id(400);
    let second_job_id = mk_job_id(401);

    store
        .insert_settlement_job(&first_job_id, &mk_settlement_job(4))
        .expect("first job insert must succeed");
    store
        .insert_settlement_job(&second_job_id, &mk_settlement_job(5))
        .expect("second job insert must succeed");
    store
        .insert_certificate_settlement_job_id(&certificate_id, &first_job_id)
        .expect("first mapping insert must succeed");

    let res = store.insert_certificate_settlement_job_id(&certificate_id, &second_job_id);

    assert!(matches!(res, Err(Error::UnprocessedAction(_))));
    assert_eq!(
        db.get::<SettlementJobIdPerCertificateIdColumn>(&certificate_id)
            .expect("Unable to read stored value"),
        Some(first_job_id)
    );
}

#[test]
fn insert_settlement_attempt_succeeds_once() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_job_id(3);
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
    let job_id = mk_job_id(4);
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
    let res = store.insert_settlement_attempt(&mk_job_id(404), 1, &mk_settlement_attempt(1));
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));
}

#[test]
fn record_settlement_attempt_result_succeeds_once() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_job_id(5);
    store
        .insert_settlement_job(&job_id, &mk_settlement_job(5))
        .expect("job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 1, &mk_settlement_attempt(1))
        .expect("attempt insert must succeed");
    assert!(store
        .record_settlement_attempt_result(
            &job_id,
            1,
            &v0::SettlementAttemptResult::contract_call_success_for_test(1)
                .try_into()
                .expect("test tx result helper should be decodable"),
        )
        .is_ok());
}

#[test]
fn record_settlement_attempt_result_idempotent_record_succeeds() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_job_id(6);
    let first = v0::SettlementAttemptResult::contract_call_success_for_test(1)
        .try_into()
        .expect("test tx result helper should be decodable");
    store
        .insert_settlement_job(&job_id, &mk_settlement_job(6))
        .expect("job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 1, &mk_settlement_attempt(1))
        .expect("attempt insert must succeed");
    store
        .record_settlement_attempt_result(&job_id, 1, &first)
        .expect("first insert must succeed");
    store
        .record_settlement_attempt_result(&job_id, 1, &first)
        .expect("idempotent record must succeed");
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
fn record_settlement_attempt_result_without_attempt_fails() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_job_id(405);
    store
        .insert_settlement_job(&job_id, &mk_settlement_job(42))
        .expect("job insert must succeed");

    let res = store.record_settlement_attempt_result(
        &job_id,
        1,
        &v0::SettlementAttemptResult::contract_call_success_for_test(1)
            .try_into()
            .expect("test tx result helper should be decodable"),
    );
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));
}

#[test]
fn record_settlement_attempt_result_replaces_client_error_with_contract_call() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_job_id(24);
    let attempt = mk_settlement_attempt(1);
    let client_error = SettlementAttemptResult::ClientError(ClientError {
        kind: ClientErrorType::Unknown,
        message: "submit failed".to_string(),
    });
    let contract_call_result = v0::SettlementAttemptResult::contract_call_success_for_test(24)
        .try_into()
        .expect("test tx result helper should be decodable");

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(24))
        .expect("job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 1, &attempt)
        .expect("attempt insert must succeed");
    store
        .record_settlement_attempt_result(&job_id, 1, &client_error)
        .expect("client error insert must succeed");
    store
        .record_settlement_attempt_result(&job_id, 1, &contract_call_result)
        .expect("contract call should replace client error");

    assert_eq!(
        db.get::<SettlementAttemptResultsColumn>(&SettlementAttemptKey {
            settlement_job_id: job_id,
            attempt_sequence_number: 1,
        })
        .expect("attempt result read must succeed"),
        Some((&contract_call_result).into())
    );
}

#[test]
fn record_settlement_attempt_result_rejects_conflicting_contract_calls() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_job_id(25);
    let attempt = mk_settlement_attempt(1);
    let first = v0::SettlementAttemptResult::contract_call_success_for_test(25)
        .try_into()
        .expect("test tx result helper should be decodable");
    let second = v0::SettlementAttemptResult::contract_call_success_for_test(26)
        .try_into()
        .expect("test tx result helper should be decodable");

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(25))
        .expect("job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 1, &attempt)
        .expect("attempt insert must succeed");
    store
        .record_settlement_attempt_result(&job_id, 1, &first)
        .expect("first result insert must succeed");

    let res = store.record_settlement_attempt_result(&job_id, 1, &second);
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));

    assert_eq!(
        db.get::<SettlementAttemptResultsColumn>(&SettlementAttemptKey {
            settlement_job_id: job_id,
            attempt_sequence_number: 1,
        })
        .expect("attempt result read must succeed"),
        Some((&first).into())
    );
}

#[test]
fn insert_settlement_attempt_indexes_by_wallet_and_nonce() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_job_id(406);
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
fn max_settlement_nonce_for_wallet_returns_highest_indexed_nonce() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_job_id(407);
    let other_job_id = mk_job_id(408);
    let wallet = Address::from([10; 20]);
    let max_wallet = Address::from([0xff; 20]);
    let other_wallet = Address::from([11; 20]);
    let mut lower_nonce_attempt = mk_settlement_attempt(1);
    let mut higher_nonce_attempt = mk_settlement_attempt(2);
    let mut other_wallet_attempt = mk_settlement_attempt(3);
    let mut max_wallet_lower_nonce_attempt = mk_settlement_attempt(4);
    let mut max_wallet_higher_nonce_attempt = mk_settlement_attempt(5);

    lower_nonce_attempt.sender_wallet = wallet;
    lower_nonce_attempt.nonce = Nonce(12);
    higher_nonce_attempt.sender_wallet = wallet;
    higher_nonce_attempt.nonce = Nonce(14);
    other_wallet_attempt.sender_wallet = other_wallet;
    other_wallet_attempt.nonce = Nonce(99);
    max_wallet_lower_nonce_attempt.sender_wallet = max_wallet;
    max_wallet_lower_nonce_attempt.nonce = Nonce(7);
    max_wallet_higher_nonce_attempt.sender_wallet = max_wallet;
    max_wallet_higher_nonce_attempt.nonce = Nonce(8);

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(42))
        .expect("first job insert must succeed");
    store
        .insert_settlement_job(&other_job_id, &mk_settlement_job(43))
        .expect("second job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 1, &lower_nonce_attempt)
        .expect("lower nonce attempt insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 2, &higher_nonce_attempt)
        .expect("higher nonce attempt insert must succeed");
    store
        .insert_settlement_attempt(&other_job_id, 1, &other_wallet_attempt)
        .expect("other-wallet attempt insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 3, &max_wallet_lower_nonce_attempt)
        .expect("max-wallet lower nonce attempt insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 4, &max_wallet_higher_nonce_attempt)
        .expect("max-wallet higher nonce attempt insert must succeed");

    assert_eq!(
        store
            .max_settlement_nonce_for_wallet(wallet)
            .expect("max nonce lookup must succeed"),
        Some(Nonce(14))
    );
    assert_eq!(
        store
            .max_settlement_nonce_for_wallet(max_wallet)
            .expect("max wallet lookup must succeed"),
        Some(Nonce(8))
    );
    assert_eq!(
        store
            .max_settlement_nonce_for_wallet(Address::from([12; 20]))
            .expect("missing wallet lookup must succeed"),
        None
    );
}

#[test]
fn get_settlement_job_returns_none_when_missing() {
    let (_tmp, _db, store) = setup_store();
    assert_eq!(
        store
            .get_settlement_job(&mk_job_id(10))
            .expect("read must succeed"),
        None
    );
}

#[test]
fn get_settlement_job_returns_value_after_insert() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_job_id(11);
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
            .get_settlement_job_result(&mk_job_id(12))
            .expect("read must succeed"),
        None
    );
}

#[test]
fn insert_settlement_job_result_without_job_fails() {
    let (_tmp, _db, store) = setup_store();
    let res = store.insert_settlement_job_result(
        &mk_job_id(13),
        &v0::SettlementJobResult::contract_call_success_for_test(13)
            .try_into()
            .expect("test tx result helper should be decodable"),
    );
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));
}

#[test]
fn get_settlement_job_result_returns_value_after_insert() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_job_id(14);
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
fn list_settlement_attempts_returns_empty_vec_for_missing_job() {
    let (_tmp, _db, store) = setup_store();

    assert!(store
        .list_settlement_attempts(&mk_job_id(16))
        .expect("read must succeed")
        .is_empty());
}

#[test]
fn list_settlement_attempts_returns_all_attempts_for_job() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_job_id(17);
    let first = mk_settlement_attempt(1);
    let second = mk_settlement_attempt(2);
    let third = mk_settlement_attempt(3);

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(17))
        .expect("job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 1, &first)
        .expect("first attempt insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 2, &second)
        .expect("second attempt insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 3, &third)
        .expect("third attempt insert must succeed");

    assert_eq!(
        store
            .list_settlement_attempts(&job_id)
            .expect("read must succeed"),
        vec![(1, first), (2, second), (3, third)]
    );
}

#[test]
fn list_settlement_attempts_does_not_return_attempts_from_other_jobs() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_job_id(18);
    let other_job_id = mk_job_id(19);
    let first = mk_settlement_attempt(1);
    let second = mk_settlement_attempt(2);

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(18))
        .expect("first job insert must succeed");
    store
        .insert_settlement_job(&other_job_id, &mk_settlement_job(19))
        .expect("second job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 1, &first)
        .expect("first attempt insert must succeed");
    store
        .insert_settlement_attempt(&other_job_id, 1, &mk_settlement_attempt(10))
        .expect("other job attempt insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 2, &second)
        .expect("second attempt insert must succeed");

    assert_eq!(
        store
            .list_settlement_attempts(&job_id)
            .expect("read must succeed"),
        vec![(1, first), (2, second)]
    );
}

#[test]
fn list_settlement_attempt_results_returns_empty_vec_for_missing_job() {
    let (_tmp, _db, store) = setup_store();

    assert!(store
        .list_settlement_attempt_results(&mk_job_id(20))
        .expect("read must succeed")
        .is_empty());
}

#[test]
fn list_settlement_attempt_results_returns_all_results_for_job() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_job_id(24);
    let first_attempt = mk_settlement_attempt(1);
    let second_attempt = mk_settlement_attempt(2);
    let first_result = v0::SettlementAttemptResult::contract_call_success_for_test(1)
        .try_into()
        .expect("test tx result helper should be decodable");
    let second_result = v0::SettlementAttemptResult::contract_call_success_for_test(2)
        .try_into()
        .expect("test tx result helper should be decodable");

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(24))
        .expect("job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 1, &first_attempt)
        .expect("first attempt insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 2, &second_attempt)
        .expect("second attempt insert must succeed");
    store
        .record_settlement_attempt_result(&job_id, 1, &first_result)
        .expect("first result insert must succeed");
    store
        .record_settlement_attempt_result(&job_id, 2, &second_result)
        .expect("second result insert must succeed");

    assert_eq!(
        store
            .list_settlement_attempt_results(&job_id)
            .expect("read must succeed"),
        vec![(1, first_result), (2, second_result)]
    );
}

#[test]
fn list_settlement_attempt_results_does_not_return_results_from_other_jobs() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_job_id(25);
    let other_job_id = mk_job_id(26);
    let first_result = v0::SettlementAttemptResult::contract_call_success_for_test(3)
        .try_into()
        .expect("test tx result helper should be decodable");
    let second_result = v0::SettlementAttemptResult::contract_call_success_for_test(4)
        .try_into()
        .expect("test tx result helper should be decodable");

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(25))
        .expect("first job insert must succeed");
    store
        .insert_settlement_job(&other_job_id, &mk_settlement_job(26))
        .expect("second job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 1, &mk_settlement_attempt(1))
        .expect("first attempt insert must succeed");
    store
        .insert_settlement_attempt(&other_job_id, 1, &mk_settlement_attempt(10))
        .expect("other job attempt insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 2, &mk_settlement_attempt(2))
        .expect("second attempt insert must succeed");
    store
        .record_settlement_attempt_result(&job_id, 1, &first_result)
        .expect("first result insert must succeed");
    store
        .record_settlement_attempt_result(
            &other_job_id,
            1,
            &v0::SettlementAttemptResult::contract_call_success_for_test(10)
                .try_into()
                .expect("test tx result helper should be decodable"),
        )
        .expect("other job result insert must succeed");
    store
        .record_settlement_attempt_result(&job_id, 2, &second_result)
        .expect("second result insert must succeed");

    assert_eq!(
        store
            .list_settlement_attempt_results(&job_id)
            .expect("read must succeed"),
        vec![(1, first_result), (2, second_result)]
    );
}

/// Reopening an existing database goes through a different code path than
/// creating a fresh one (`open_rocksdb_existing` opens column families by
/// name, without the column options carrying the prefix extractor), so
/// listing must not rely on column-family options to stop at the job-id
/// prefix boundary. Every job numbers its attempts from 0, so leaking the
/// next job's entries surfaces as a duplicate attempt 0 during startup
/// recovery.
#[test]
fn list_settlement_attempts_stays_within_job_after_reopen() {
    let tmp = TempDBDir::new();
    let job_id = mk_job_id(27);
    let other_job_id = mk_job_id(28);
    let attempt = mk_settlement_attempt(1);
    let result: SettlementAttemptResult =
        v0::SettlementAttemptResult::contract_call_success_for_test(5)
            .try_into()
            .expect("test tx result helper should be decodable");

    {
        let db = Arc::new(StateStore::init_db(tmp.path.as_path()).expect("Unable to init db"));
        let store = StateStore::new(db, BackupClient::noop());
        store
            .insert_settlement_job(&job_id, &mk_settlement_job(27))
            .expect("first job insert must succeed");
        store
            .insert_settlement_job(&other_job_id, &mk_settlement_job(28))
            .expect("second job insert must succeed");
        store
            .insert_settlement_attempt(&job_id, 0, &attempt)
            .expect("first job attempt insert must succeed");
        store
            .insert_settlement_attempt(&other_job_id, 0, &mk_settlement_attempt(10))
            .expect("other job attempt insert must succeed");
        store
            .record_settlement_attempt_result(&job_id, 0, &result)
            .expect("first job result insert must succeed");
        store
            .record_settlement_attempt_result(
                &other_job_id,
                0,
                &v0::SettlementAttemptResult::contract_call_success_for_test(10)
                    .try_into()
                    .expect("test tx result helper should be decodable"),
            )
            .expect("other job result insert must succeed");
    }

    let db = Arc::new(StateStore::init_db(tmp.path.as_path()).expect("Unable to reopen db"));
    let store = StateStore::new(db, BackupClient::noop());

    assert_eq!(
        store
            .list_settlement_attempts(&job_id)
            .expect("read must succeed"),
        vec![(0, attempt)]
    );
    assert_eq!(
        store
            .list_settlement_attempt_results(&job_id)
            .expect("read must succeed"),
        vec![(0, result)]
    );
}

/// The declared column options — in particular the fixed-prefix extractor —
/// must be applied when reopening an existing database, not only when
/// creating a fresh one. This probes RocksDB's own prefix iterator directly,
/// bypassing the explicit bounds set by `DB::prefix_iterator`: without the
/// extractor it silently degrades to an unbounded scan.
#[test]
fn reopened_database_applies_declared_prefix_extractor() {
    let tmp = TempDBDir::new();
    let job_id = mk_job_id(29);
    let other_job_id = mk_job_id(30);

    {
        let db = Arc::new(StateStore::init_db(tmp.path.as_path()).expect("Unable to init db"));
        let store = StateStore::new(db, BackupClient::noop());
        store
            .insert_settlement_job(&job_id, &mk_settlement_job(29))
            .expect("first job insert must succeed");
        store
            .insert_settlement_job(&other_job_id, &mk_settlement_job(30))
            .expect("second job insert must succeed");
        store
            .insert_settlement_attempt(&job_id, 0, &mk_settlement_attempt(1))
            .expect("first job attempt insert must succeed");
        store
            .insert_settlement_attempt(&other_job_id, 0, &mk_settlement_attempt(10))
            .expect("other job attempt insert must succeed");
    }

    let db = StateStore::init_db(tmp.path.as_path()).expect("Unable to reopen db");
    let cf = db
        .raw_rocksdb()
        .cf_handle(SETTLEMENT_ATTEMPTS_CF)
        .expect("settlement attempts column family must exist");
    let prefix = job_id.to_be_bytes();
    let keys_outside_prefix: Vec<_> = db
        .raw_rocksdb()
        .prefix_iterator_cf(cf, prefix)
        .map(|entry| entry.expect("iteration must succeed").0)
        .filter(|key| !key.starts_with(&prefix))
        .collect();

    assert!(
        keys_outside_prefix.is_empty(),
        "prefix extractor was not applied on reopen; leaked keys: {keys_outside_prefix:?}",
    );
}

#[test]
fn insert_settlement_job_result_duplicate_fails() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_job_id(15);
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
    let job_id = mk_job_id(21);
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
        .record_settlement_attempt_result(&job_id, 5, &attempt_result)
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
    let job_id = mk_job_id(22);
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
