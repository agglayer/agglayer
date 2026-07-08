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
    },
    error::Error,
    stores::{
        state::StateStore, EditEvenIfCompleted, SettlementReader as _, SettlementWriter as _,
        StateReader as _, StateWriter as _,
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

fn abandoned_by_admin_result() -> SettlementAttemptResult {
    SettlementAttemptResult::ClientError(ClientError::abandoned_by_admin("operator says no"))
}

fn stored_attempt_result(
    db: &crate::storage::DB,
    job_id: SettlementJobId,
    attempt_sequence_number: u64,
) -> Option<v0::SettlementAttemptResult> {
    db.get::<SettlementAttemptResultsColumn>(&SettlementAttemptKey {
        settlement_job_id: job_id,
        attempt_sequence_number,
    })
    .expect("attempt result read must succeed")
}

#[test]
fn admin_insert_settlement_attempt_assigns_next_sequence_number() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_job_id(50);
    let first = mk_settlement_attempt(1);
    let second = mk_settlement_attempt(2);

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(50))
        .expect("job insert must succeed");

    assert_eq!(
        store
            .admin_insert_settlement_attempt(&job_id, &first, EditEvenIfCompleted::No)
            .expect("first admin insert must succeed"),
        0
    );

    // A gap left by the regular writer does not confuse the assignment: the
    // next sequence number is one past the highest existing one.
    store
        .insert_settlement_attempt(&job_id, 7, &mk_settlement_attempt(3))
        .expect("regular insert must succeed");

    assert_eq!(
        store
            .admin_insert_settlement_attempt(&job_id, &second, EditEvenIfCompleted::No)
            .expect("second admin insert must succeed"),
        8
    );

    assert_eq!(
        store
            .list_settlement_attempts(&job_id)
            .expect("attempt list must succeed")
            .into_iter()
            .map(|(sequence_number, _)| sequence_number)
            .collect::<Vec<_>>(),
        vec![0, 7, 8]
    );

    // The per-wallet index is maintained like for regular inserts.
    let index_key = SettlementAttemptPerWalletKey {
        address: second.sender_wallet.into_array(),
        nonce: second.nonce.0,
        settlement_job_id: job_id,
        attempt_sequence_number: 8,
    };
    assert!(matches!(
        db.get::<SettlementAttemptPerWalletColumn>(&index_key)
            .expect("index read must succeed"),
        Some(SettlementAttemptPerWalletValue)
    ));
}

#[test]
fn admin_insert_settlement_attempt_without_job_fails() {
    let (_tmp, _db, store) = setup_store();
    let res = store.admin_insert_settlement_attempt(
        &mk_job_id(51),
        &mk_settlement_attempt(1),
        EditEvenIfCompleted::No,
    );
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));
}

#[test]
fn admin_insert_settlement_attempt_on_completed_job_requires_force() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_job_id(52);

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(52))
        .expect("job insert must succeed");
    store
        .insert_settlement_job_result(
            &job_id,
            &v0::SettlementJobResult::contract_call_success_for_test(52)
                .try_into()
                .expect("test tx result helper should be decodable"),
        )
        .expect("job result insert must succeed");

    let res = store.admin_insert_settlement_attempt(
        &job_id,
        &mk_settlement_attempt(1),
        EditEvenIfCompleted::No,
    );
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));

    assert_eq!(
        store
            .admin_insert_settlement_attempt(
                &job_id,
                &mk_settlement_attempt(1),
                EditEvenIfCompleted::Yes,
            )
            .expect("forced insert must succeed on a completed job"),
        0
    );
}

#[test]
fn admin_override_settlement_attempt_result_writes_missing_result() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_job_id(53);
    let abandoned = abandoned_by_admin_result();

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(53))
        .expect("job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 1, &mk_settlement_attempt(1))
        .expect("attempt insert must succeed");

    store
        .admin_override_settlement_attempt_result(&job_id, 1, &abandoned, EditEvenIfCompleted::No)
        .expect("override must succeed");

    assert_eq!(
        stored_attempt_result(&db, job_id, 1),
        Some((&abandoned).into())
    );
}

#[test]
fn admin_override_settlement_attempt_result_replaces_conflicting_result() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_job_id(54);
    let contract_call: SettlementAttemptResult =
        v0::SettlementAttemptResult::contract_call_success_for_test(54)
            .try_into()
            .expect("test tx result helper should be decodable");
    let abandoned = abandoned_by_admin_result();

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(54))
        .expect("job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 1, &mk_settlement_attempt(1))
        .expect("attempt insert must succeed");
    store
        .record_settlement_attempt_result(&job_id, 1, &contract_call)
        .expect("result insert must succeed");

    // The regular writer refuses this transition...
    let res = store.record_settlement_attempt_result(&job_id, 1, &abandoned);
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));

    // ...while the admin override bypasses `can_be_replaced_by`.
    store
        .admin_override_settlement_attempt_result(&job_id, 1, &abandoned, EditEvenIfCompleted::No)
        .expect("override must succeed");

    assert_eq!(
        stored_attempt_result(&db, job_id, 1),
        Some((&abandoned).into())
    );
}

#[test]
fn admin_override_settlement_attempt_result_without_attempt_fails() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_job_id(55);

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(55))
        .expect("job insert must succeed");

    let res = store.admin_override_settlement_attempt_result(
        &job_id,
        1,
        &abandoned_by_admin_result(),
        EditEvenIfCompleted::No,
    );
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));
}

#[test]
fn admin_override_settlement_attempt_result_on_completed_job_requires_force() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_job_id(56);
    let contract_call: SettlementAttemptResult =
        v0::SettlementAttemptResult::contract_call_success_for_test(56)
            .try_into()
            .expect("test tx result helper should be decodable");

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(56))
        .expect("job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 1, &mk_settlement_attempt(1))
        .expect("attempt insert must succeed");
    store
        .record_settlement_attempt_result(&job_id, 1, &contract_call)
        .expect("result insert must succeed");
    store
        .insert_settlement_job_result(
            &job_id,
            &v0::SettlementJobResult::contract_call_success_for_test(56)
                .try_into()
                .expect("test tx result helper should be decodable"),
        )
        .expect("job result insert must succeed");

    let res = store.admin_override_settlement_attempt_result(
        &job_id,
        1,
        &abandoned_by_admin_result(),
        EditEvenIfCompleted::No,
    );
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));
    assert_eq!(
        stored_attempt_result(&db, job_id, 1),
        Some((&contract_call).into())
    );

    store
        .admin_override_settlement_attempt_result(
            &job_id,
            1,
            &abandoned_by_admin_result(),
            EditEvenIfCompleted::Yes,
        )
        .expect("forced override must succeed on a completed job");
    assert_eq!(
        stored_attempt_result(&db, job_id, 1),
        Some((&abandoned_by_admin_result()).into())
    );
}

#[test]
fn record_settlement_attempt_result_keeps_admin_abandoned_over_client_notes() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_job_id(60);
    let abandoned = abandoned_by_admin_result();
    let nonce_used = SettlementAttemptResult::ClientError(ClientError {
        kind: ClientErrorType::NonceAlreadyUsed,
        message: "nonce used elsewhere".to_string(),
    });
    let contract_call: SettlementAttemptResult =
        v0::SettlementAttemptResult::contract_call_success_for_test(60)
            .try_into()
            .expect("test tx result helper should be decodable");

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(60))
        .expect("job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 1, &mk_settlement_attempt(1))
        .expect("attempt insert must succeed");
    store
        .admin_override_settlement_attempt_result(&job_id, 1, &abandoned, EditEvenIfCompleted::No)
        .expect("override must succeed");

    // A client-side note from a task that missed the override is dropped
    // without an error...
    store
        .record_settlement_attempt_result(&job_id, 1, &nonce_used)
        .expect("client note over admin abandon must report success");
    assert_eq!(
        stored_attempt_result(&db, job_id, 1),
        Some((&abandoned).into())
    );

    // ...while real on-chain evidence still replaces the assertion.
    store
        .record_settlement_attempt_result(&job_id, 1, &contract_call)
        .expect("on-chain evidence must replace admin abandon");
    assert_eq!(
        stored_attempt_result(&db, job_id, 1),
        Some((&contract_call).into())
    );
}

#[test]
fn admin_remove_settlement_attempt_result_hands_attempt_back() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_job_id(57);
    let abandoned = abandoned_by_admin_result();

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(57))
        .expect("job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 1, &mk_settlement_attempt(1))
        .expect("attempt insert must succeed");
    store
        .record_settlement_attempt_result(
            &job_id,
            1,
            &SettlementAttemptResult::ClientError(ClientError {
                kind: ClientErrorType::Unknown,
                message: "submit failed".to_string(),
            }),
        )
        .expect("result insert must succeed");

    store
        .admin_remove_settlement_attempt_result(&job_id, 1, EditEvenIfCompleted::No)
        .expect("removal must succeed");

    assert_eq!(stored_attempt_result(&db, job_id, 1), None);

    // The attempt itself is untouched and can receive a fresh result through
    // the regular writer.
    assert_eq!(
        store
            .list_settlement_attempts(&job_id)
            .expect("attempt list must succeed")
            .len(),
        1
    );
    store
        .record_settlement_attempt_result(&job_id, 1, &abandoned)
        .expect("recording a fresh result must succeed");
}

#[test]
fn admin_remove_settlement_attempt_result_without_result_fails() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_job_id(58);

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(58))
        .expect("job insert must succeed");

    // Missing attempt.
    let res = store.admin_remove_settlement_attempt_result(&job_id, 1, EditEvenIfCompleted::No);
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));

    // Attempt present but no recorded result.
    store
        .insert_settlement_attempt(&job_id, 1, &mk_settlement_attempt(1))
        .expect("attempt insert must succeed");
    let res = store.admin_remove_settlement_attempt_result(&job_id, 1, EditEvenIfCompleted::No);
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));
}

#[test]
fn admin_remove_settlement_attempt_result_on_completed_job_requires_force() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_job_id(59);
    let contract_call: SettlementAttemptResult =
        v0::SettlementAttemptResult::contract_call_success_for_test(59)
            .try_into()
            .expect("test tx result helper should be decodable");

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(59))
        .expect("job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 1, &mk_settlement_attempt(1))
        .expect("attempt insert must succeed");
    store
        .record_settlement_attempt_result(&job_id, 1, &contract_call)
        .expect("result insert must succeed");
    store
        .insert_settlement_job_result(
            &job_id,
            &v0::SettlementJobResult::contract_call_success_for_test(59)
                .try_into()
                .expect("test tx result helper should be decodable"),
        )
        .expect("job result insert must succeed");

    let res = store.admin_remove_settlement_attempt_result(&job_id, 1, EditEvenIfCompleted::No);
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));
    assert_eq!(
        stored_attempt_result(&db, job_id, 1),
        Some((&contract_call).into())
    );

    store
        .admin_remove_settlement_attempt_result(&job_id, 1, EditEvenIfCompleted::Yes)
        .expect("forced removal must succeed on a completed job");
    assert_eq!(stored_attempt_result(&db, job_id, 1), None);
}

#[test]
fn admin_force_remove_settlement_job_result_uncompletes_job() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_job_id(61);
    let attempt = mk_settlement_attempt(1);
    let attempt_result: SettlementAttemptResult =
        v0::SettlementAttemptResult::contract_call_success_for_test(61)
            .try_into()
            .expect("test tx result helper should be decodable");

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(61))
        .expect("job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 1, &attempt)
        .expect("attempt insert must succeed");
    store
        .record_settlement_attempt_result(&job_id, 1, &attempt_result)
        .expect("attempt result insert must succeed");
    store
        .insert_settlement_job_result(
            &job_id,
            &v0::SettlementJobResult::contract_call_success_for_test(61)
                .try_into()
                .expect("test tx result helper should be decodable"),
        )
        .expect("job result insert must succeed");

    store
        .admin_force_remove_settlement_job_result(&job_id)
        .expect("force removal must succeed");

    // The job is pending again; its attempts and their results are untouched.
    assert_eq!(
        store
            .get_settlement_job_result(&job_id)
            .expect("job result read must succeed"),
        None
    );
    assert!(store
        .get_settlement_job(&job_id)
        .expect("job read must succeed")
        .is_some());
    assert_eq!(
        stored_attempt_result(&db, job_id, 1),
        Some((&attempt_result).into())
    );

    // Admin attempt edits are accepted again now that the job is pending.
    store
        .admin_remove_settlement_attempt_result(&job_id, 1, EditEvenIfCompleted::No)
        .expect("attempt result removal must succeed after job un-completion");
}

/// The intended un-completion flow: correct the attempt results with forced
/// edits while the job still has its terminal result, then remove the result.
/// The other order would let the respawned task re-derive the job result from
/// the uncorrected attempts.
#[test]
fn forced_attempt_edits_prepare_job_result_removal() {
    let (_tmp, db, store) = setup_store();
    let job_id = mk_job_id(63);
    let contract_call: SettlementAttemptResult =
        v0::SettlementAttemptResult::contract_call_success_for_test(63)
            .try_into()
            .expect("test tx result helper should be decodable");
    let abandoned = abandoned_by_admin_result();

    store
        .insert_settlement_job(&job_id, &mk_settlement_job(63))
        .expect("job insert must succeed");
    store
        .insert_settlement_attempt(&job_id, 1, &mk_settlement_attempt(1))
        .expect("attempt insert must succeed");
    store
        .record_settlement_attempt_result(&job_id, 1, &contract_call)
        .expect("result insert must succeed");
    store
        .insert_settlement_job_result(
            &job_id,
            &v0::SettlementJobResult::contract_call_success_for_test(63)
                .try_into()
                .expect("test tx result helper should be decodable"),
        )
        .expect("job result insert must succeed");

    // Correct the wrong attempt result first, while the job is still
    // completed...
    store
        .admin_override_settlement_attempt_result(&job_id, 1, &abandoned, EditEvenIfCompleted::Yes)
        .expect("forced override must succeed on the completed job");

    // ...then un-complete the job.
    store
        .admin_force_remove_settlement_job_result(&job_id)
        .expect("force removal must succeed");

    assert_eq!(
        store
            .get_settlement_job_result(&job_id)
            .expect("job result read must succeed"),
        None
    );
    assert_eq!(
        stored_attempt_result(&db, job_id, 1),
        Some((&abandoned).into())
    );
}

#[test]
fn admin_force_remove_settlement_job_result_without_result_fails() {
    let (_tmp, _db, store) = setup_store();
    let job_id = mk_job_id(62);

    // Missing job.
    let res = store.admin_force_remove_settlement_job_result(&job_id);
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));

    // Job present but still pending.
    store
        .insert_settlement_job(&job_id, &mk_settlement_job(62))
        .expect("job insert must succeed");
    let res = store.admin_force_remove_settlement_job_result(&job_id);
    assert!(matches!(res, Err(Error::UnprocessedAction(_))));
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
