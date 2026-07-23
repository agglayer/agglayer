use std::{
    collections::BTreeMap,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::{Arc, Mutex},
};

use agglayer_config::Multiplier;
use agglayer_storage::{error::Error, tests::mocks::MockStateStore};
use agglayer_types::{
    CertificateId, ClientError, ClientErrorType, ContractCallOutcome, Digest,
    SettlementAttemptResult, B256, U256,
};
use alloy::{
    consensus::{Signed, TxEip1559},
    network::EthereumWallet,
    node_bindings::Anvil,
    primitives::{Signature, TxKind, U64},
    providers::{mock::Asserter, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
    transports::TransportErrorKind,
};
use rstest::rstest;

use super::*;
use crate::utils::build_provider;

fn test_signer() -> PrivateKeySigner {
    PrivateKeySigner::from_slice(&[0x11; 32]).expect("valid test signing key")
}

fn mk_provider() -> impl Provider + WalletProvider + 'static {
    ProviderBuilder::new()
        .wallet(EthereumWallet::from(test_signer()))
        .connect_http(
            "http://127.0.0.1:0"
                .parse()
                .expect("test provider URL should parse"),
        )
}

fn mk_job_id(seed: u128) -> SettlementJobId {
    SettlementJobId::from(ulid::Ulid::from(seed))
}

fn mk_control() -> TaskControl {
    let (_handle, control) = TaskControlHandle::new(&CancellationToken::new());
    control
}

fn mk_job() -> SettlementJob {
    SettlementJob {
        contract_address: agglayer_types::Address::from([1; 20]),
        calldata: vec![2, 3].into(),
        eth_value: U256::from(0),
        gas_limit: 100_000,
    }
}

fn mk_tx_hash(seed: u8) -> SettlementTxHash {
    SettlementTxHash::new(Digest::from([seed; 32]))
}

fn mk_tx(hash_seed: u8) -> TxEnvelope {
    TxEnvelope::Eip1559(Signed::new_unchecked(
        TxEip1559 {
            chain_id: 1,
            nonce: 2,
            gas_limit: 100_000,
            max_fee_per_gas: 100,
            max_priority_fee_per_gas: 10,
            to: TxKind::Call(Address::from([6; 20])),
            value: U256::from(7_u64),
            input: vec![8].into(),
            access_list: Default::default(),
        },
        Signature::test_signature(),
        B256::from([hash_seed; 32]),
    ))
}

fn mk_contract_call_result(seed: u8, outcome: ContractCallOutcome) -> ContractCallResult {
    ContractCallResult {
        outcome,
        metadata: vec![seed, seed.wrapping_add(1)].into(),
        block_hash: B256::from([seed.wrapping_add(2); 32]),
        block_number: seed as u64,
        tx_hash: mk_tx_hash(seed.wrapping_add(3)),
    }
}

fn mk_job_result(seed: u8, outcome: ContractCallOutcome) -> SettlementJobResult {
    SettlementJobResult {
        wallet: Address::from([seed; 20]).into(),
        nonce: Nonce(seed as u64),
        attempt_number: SettlementAttemptNumber(seed as u64),
        contract_call_result: mk_contract_call_result(seed, outcome),
    }
}

fn mk_active_attempt(
    wallet: Address,
    nonce: Nonce,
    hash: SettlementTxHash,
    result: Option<SettlementAttemptResult>,
) -> ActiveSettlementAttempt {
    ActiveSettlementAttempt {
        attempt: SettlementAttempt {
            sender_wallet: wallet.into(),
            nonce,
            hash,
            submission_time: SystemTime::UNIX_EPOCH,
            max_fee_per_gas: 0,
            max_priority_fee_per_gas: 0,
        },
        result,
    }
}

fn mk_stored_attempt(seed: u8, sender_wallet: Address, nonce: Nonce) -> SettlementAttempt {
    SettlementAttempt {
        sender_wallet: sender_wallet.into(),
        nonce,
        hash: mk_tx_hash(seed),
        submission_time: SystemTime::UNIX_EPOCH + Duration::from_secs(seed.into()),
        max_fee_per_gas: 0,
        max_priority_fee_per_gas: 0,
    }
}

fn mk_client_error(seed: u8) -> SettlementAttemptResult {
    SettlementAttemptResult::ClientError(ClientError {
        kind: ClientErrorType::Unknown,
        message: format!("client error {seed}"),
    })
}

fn mk_task(
    store: Arc<MockStateStore>,
    attempts: ActiveSettlementAttempts,
) -> SettlementTask<impl Provider + WalletProvider + 'static, MockStateStore> {
    mk_task_with_id(SettlementJobId::from(1u128), store, attempts)
}

fn mk_task_with_id(
    job_id: SettlementJobId,
    store: Arc<MockStateStore>,
    attempts: ActiveSettlementAttempts,
) -> SettlementTask<impl Provider + WalletProvider + 'static, MockStateStore> {
    mk_task_with_id_and_provider(job_id, mk_provider(), store, attempts)
}

fn mk_task_with_provider<L1Provider: Provider + WalletProvider + 'static>(
    provider: L1Provider,
    store: Arc<MockStateStore>,
    attempts: ActiveSettlementAttempts,
) -> SettlementTask<L1Provider, MockStateStore> {
    mk_task_with_id_and_provider(SettlementJobId::from(1u128), provider, store, attempts)
}

fn mk_task_with_id_and_provider<L1Provider: Provider + WalletProvider + 'static>(
    job_id: SettlementJobId,
    provider: L1Provider,
    store: Arc<MockStateStore>,
    attempts: ActiveSettlementAttempts,
) -> SettlementTask<L1Provider, MockStateStore> {
    SettlementTask {
        id: job_id,
        job: mk_job(),
        tx_config: Arc::new(SettlementTransactionConfig::default()),
        provider: Arc::new(provider),
        store,
        wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
        control: mk_control(),
        attempts,
    }
}

fn mk_task_with_tx_config<P: Provider + WalletProvider + 'static>(
    provider: P,
    tx_config: SettlementTransactionConfig,
) -> SettlementTask<P, MockStateStore> {
    SettlementTask {
        id: mk_job_id(1),
        job: mk_job(),
        tx_config: Arc::new(tx_config),
        provider: Arc::new(provider),
        store: Arc::new(MockStateStore::new()),
        wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
        control: mk_control(),
        attempts: BTreeMap::new(),
    }
}

mod nonce_lock;

#[test]
fn next_attempt_number_starts_at_zero_and_increments_past_max() {
    let store = Arc::new(MockStateStore::new());

    let empty = mk_task(store.clone(), BTreeMap::new());
    assert_eq!(empty.next_attempt_number(), SettlementAttemptNumber(0));

    let wallet = Address::from([1; 20]);
    let nonce = Nonce(7);
    let attempts = BTreeMap::from([(
        (wallet, nonce),
        BTreeMap::from([
            (
                SettlementAttemptNumber(2),
                mk_active_attempt(wallet, nonce, mk_tx_hash(1), None),
            ),
            (
                SettlementAttemptNumber(5),
                mk_active_attempt(wallet, nonce, mk_tx_hash(2), None),
            ),
        ]),
    )]);
    let task = mk_task(store, attempts);
    assert_eq!(task.next_attempt_number(), SettlementAttemptNumber(6));
}

fn mk_mock_provider_with_pending_nonce(nonce: u64) -> impl Provider + WalletProvider + 'static {
    let asserter = Asserter::new();
    asserter.push_success(&U64::from(nonce));
    ProviderBuilder::new()
        .wallet(EthereumWallet::from(test_signer()))
        .connect_mocked_client(asserter)
}

// `create` now resolves the gas limit via `eth_estimateGas`; this answers that
// one call so the durable-write path under test runs.
fn mk_mock_provider_with_gas_estimate(gas: u64) -> impl Provider + WalletProvider + 'static {
    let asserter = Asserter::new();
    asserter.push_success(&U64::from(gas));
    ProviderBuilder::new()
        .wallet(EthereumWallet::from(test_signer()))
        .connect_mocked_client(asserter)
}

async fn load_job_from_store<L1Provider: Provider + WalletProvider + 'static>(
    _provider: L1Provider,
    store: &MockStateStore,
    job_id: SettlementJobId,
) -> eyre::Result<(SettlementJob, Option<SettlementJobResult>)> {
    SettlementTask::<L1Provider, MockStateStore>::load_settlement_job_from_db(store, job_id).await
}

#[tokio::test]
async fn save_settlement_job_to_db_inserts_job() {
    let mut store = MockStateStore::new();
    let job_id = mk_job_id(1);
    let job = mk_job();
    let expected_job = job.clone();

    store
        .expect_insert_settlement_job()
        .once()
        .withf(move |recorded_job_id, recorded_job| {
            recorded_job_id == &job_id && recorded_job == &expected_job
        })
        .return_once(|_, _| Ok(()));

    let task = mk_task(Arc::new(store), BTreeMap::new());

    task.save_settlement_job_to_db()
        .await
        .expect("settlement job should be saved");
}

#[tokio::test]
async fn create_generates_settlement_job_id() {
    let mut store = MockStateStore::new();
    let job = mk_job();
    // `create` resolves the gas limit via estimateGas (mock returns 200_000).
    let mut expected_job = job.clone();
    expected_job.gas_limit = 200_000;
    let recorded_job_id = Arc::new(Mutex::new(None));
    let recorded_job_id_for_store = recorded_job_id.clone();

    store
        .expect_insert_settlement_job()
        .once()
        .withf(move |_, recorded_job| recorded_job == &expected_job)
        .return_once(move |recorded_job_id, _| {
            *recorded_job_id_for_store.lock().unwrap() = Some(*recorded_job_id);
            Ok(())
        });

    let (job_id, task) = SettlementTask::create(
        None,
        job,
        Arc::new(SettlementTransactionConfig::default()),
        Arc::new(mk_mock_provider_with_gas_estimate(200_000)),
        Arc::new(store),
        Arc::new(WalletNonceLocks::default()),
        mk_control(),
    )
    .await
    .expect("settlement task should be created");

    assert_eq!(task.id, job_id);
    assert_eq!(*recorded_job_id.lock().unwrap(), Some(job_id));
}

#[tokio::test]
async fn create_records_certificate_link_before_settlement_job() {
    let mut store = MockStateStore::new();
    let certificate_id = CertificateId::new(Digest::from([7; 32]));
    let job = mk_job();
    // `create` resolves the gas limit via estimateGas (mock returns 200_000).
    let mut expected_job = job.clone();
    expected_job.gas_limit = 200_000;
    let recorded_job_id = Arc::new(Mutex::new(None));
    let ordering = Arc::new(Mutex::new(Vec::new()));

    store
        .expect_insert_certificate_settlement_job_id()
        .once()
        .withf(move |recorded_certificate_id, _| recorded_certificate_id == &certificate_id)
        .return_once({
            let ordering = ordering.clone();
            let recorded_job_id = recorded_job_id.clone();
            move |_, settlement_job_id| {
                ordering.lock().unwrap().push("write_link");
                *recorded_job_id.lock().unwrap() = Some(*settlement_job_id);
                Ok(())
            }
        });

    store
        .expect_insert_settlement_job()
        .once()
        .withf(move |_, recorded_job| recorded_job == &expected_job)
        .return_once({
            let ordering = ordering.clone();
            let recorded_job_id = recorded_job_id.clone();
            move |settlement_job_id, _| {
                ordering.lock().unwrap().push("write_job");
                assert_eq!(*recorded_job_id.lock().unwrap(), Some(*settlement_job_id));
                Ok(())
            }
        });

    let (job_id, task) = SettlementTask::create(
        Some(certificate_id),
        job,
        Arc::new(SettlementTransactionConfig::default()),
        Arc::new(mk_mock_provider_with_gas_estimate(200_000)),
        Arc::new(store),
        Arc::new(WalletNonceLocks::default()),
        mk_control(),
    )
    .await
    .expect("settlement task should be created");

    assert_eq!(task.id, job_id);
    assert_eq!(*recorded_job_id.lock().unwrap(), Some(job_id));
    assert_eq!(
        ordering.lock().unwrap().as_slice(),
        ["write_link", "write_job"]
    );
}

#[tokio::test]
async fn create_fails_when_certificate_link_already_exists() {
    let mut store = MockStateStore::new();
    let certificate_id = CertificateId::new(Digest::from([8; 32]));
    let job = mk_job();

    store
        .expect_insert_certificate_settlement_job_id()
        .once()
        .withf(move |recorded_certificate_id, _| recorded_certificate_id == &certificate_id)
        .return_once(|_, _| {
            Err(Error::Unexpected(
                "Certificate already has a settlement job id".to_string(),
            ))
        });

    store.expect_insert_settlement_job().never();

    let result = SettlementTask::create(
        Some(certificate_id),
        job,
        Arc::new(SettlementTransactionConfig::default()),
        Arc::new(mk_mock_provider_with_gas_estimate(200_000)),
        Arc::new(store),
        Arc::new(WalletNonceLocks::default()),
        mk_control(),
    )
    .await;

    let error = result
        .err()
        .expect("duplicate certificate link should fail");
    assert!(
        error
            .to_string()
            .contains("Failed to write settlement job id"),
        "{error:?}"
    );
}

#[tokio::test]
async fn save_settlement_job_to_db_reports_storage_error() {
    let mut store = MockStateStore::new();
    store
        .expect_insert_settlement_job()
        .once()
        .return_once(|_, _| Err(Error::Unexpected("injected storage failure".to_string())));

    let task = mk_task(Arc::new(store), BTreeMap::new());

    let error = task
        .save_settlement_job_to_db()
        .await
        .expect_err("storage errors should be surfaced");

    assert!(error.to_string().contains("Failed to write settlement job"));
}

#[tokio::test]
async fn load_settlement_job_from_db_returns_pending_job() {
    let mut store = MockStateStore::new();
    let job_id = mk_job_id(2);
    let job = mk_job();
    let expected_job = job.clone();

    store
        .expect_get_settlement_job()
        .once()
        .withf(move |recorded_job_id| recorded_job_id == &job_id)
        .return_once(move |_| Ok(Some(job)));
    store
        .expect_get_settlement_job_result()
        .once()
        .withf(move |recorded_job_id| recorded_job_id == &job_id)
        .return_once(|_| Ok(None));

    let (loaded_job, loaded_result) = load_job_from_store(mk_provider(), &store, job_id)
        .await
        .expect("settlement job should load");

    assert_eq!(loaded_job, expected_job);
    assert!(loaded_result.is_none());
}

#[tokio::test]
async fn load_settlement_job_from_db_returns_completed_job() {
    let mut store = MockStateStore::new();
    let job_id = mk_job_id(3);
    let job = mk_job();
    let job_result = mk_job_result(4, ContractCallOutcome::Success);
    let expected_job = job.clone();
    let expected_job_result = job_result.clone();

    store
        .expect_get_settlement_job()
        .once()
        .return_once(move |_| Ok(Some(job)));
    store
        .expect_get_settlement_job_result()
        .once()
        .withf(move |recorded_job_id| recorded_job_id == &job_id)
        .return_once(move |_| Ok(Some(job_result)));

    let (loaded_job, loaded_result) = load_job_from_store(mk_provider(), &store, job_id)
        .await
        .expect("settlement job should load");

    assert_eq!(loaded_job, expected_job);
    assert_eq!(loaded_result, Some(expected_job_result));
}

#[tokio::test]
async fn load_settlement_job_from_db_reports_missing_job() {
    let mut store = MockStateStore::new();
    let job_id = mk_job_id(4);

    store
        .expect_get_settlement_job()
        .once()
        .withf(move |recorded_job_id| recorded_job_id == &job_id)
        .return_once(|_| Ok(None));
    store.expect_get_settlement_job_result().never();

    let error = load_job_from_store(mk_provider(), &store, job_id)
        .await
        .expect_err("missing settlement job should be reported");

    assert!(error.to_string().contains("No settlement job found for id"));
}

#[tokio::test]
async fn load_returns_completed_settlement_job() {
    let mut store = MockStateStore::new();
    let job_id = mk_job_id(5);
    let job = mk_job();
    let job_result = mk_job_result(6, ContractCallOutcome::Success);
    let expected_job = job.clone();
    let expected_job_result = job_result.clone();

    store
        .expect_get_settlement_job()
        .once()
        .withf(move |recorded_job_id| recorded_job_id == &job_id)
        .return_once(move |_| Ok(Some(job)));
    store
        .expect_get_settlement_job_result()
        .once()
        .withf(move |recorded_job_id| recorded_job_id == &job_id)
        .return_once(move |_| Ok(Some(job_result)));
    store.expect_list_settlement_attempts().never();
    store.expect_list_settlement_attempt_results().never();

    let loaded = SettlementTask::load(
        job_id,
        Arc::new(SettlementTransactionConfig::default()),
        Arc::new(mk_provider()),
        Arc::new(store),
        Arc::new(WalletNonceLocks::default()),
        mk_control(),
    )
    .await
    .expect("completed settlement job should load");

    match loaded {
        StoredSettlementJob::Completed(loaded_job, loaded_result) => {
            assert_eq!(loaded_job, expected_job);
            assert_eq!(loaded_result, expected_job_result);
        }
        StoredSettlementJob::Pending(_) => {
            panic!("completed settlement job should not reload as pending")
        }
    }
}

#[tokio::test]
async fn assign_next_nonce_for_wallet_uses_l1_pending_nonce_when_queue_is_empty() {
    let wallet = Address::from([9; 20]);
    let expected_wallet: agglayer_types::Address = wallet.into();
    let mut store = MockStateStore::new();
    store
        .expect_max_settlement_nonce_for_wallet()
        .once()
        .withf(move |recorded_wallet| *recorded_wallet == expected_wallet)
        .return_once(|_| Ok(None));

    let task = mk_task_with_provider(
        mk_mock_provider_with_pending_nonce(5),
        Arc::new(store),
        BTreeMap::new(),
    );

    let nonce = task
        .assign_next_nonce_for_wallet(wallet)
        .await
        .expect("nonce assignment should succeed");

    assert_eq!(nonce, Nonce(5));
}

#[tokio::test]
async fn assign_next_nonce_for_wallet_uses_next_local_nonce_when_queue_is_ahead_of_l1() {
    let wallet = Address::from([10; 20]);
    let expected_wallet: agglayer_types::Address = wallet.into();
    let mut store = MockStateStore::new();
    store
        .expect_max_settlement_nonce_for_wallet()
        .once()
        .withf(move |recorded_wallet| *recorded_wallet == expected_wallet)
        .return_once(|_| Ok(Some(Nonce(6))));

    let task = mk_task_with_provider(
        mk_mock_provider_with_pending_nonce(5),
        Arc::new(store),
        BTreeMap::new(),
    );

    let nonce = task
        .assign_next_nonce_for_wallet(wallet)
        .await
        .expect("nonce assignment should succeed");

    assert_eq!(nonce, Nonce(7));
}

#[tokio::test]
async fn assign_next_nonce_for_wallet_uses_l1_pending_nonce_when_l1_is_ahead_of_queue() {
    let wallet = Address::from([11; 20]);
    let expected_wallet: agglayer_types::Address = wallet.into();
    let mut store = MockStateStore::new();
    store
        .expect_max_settlement_nonce_for_wallet()
        .once()
        .withf(move |recorded_wallet| *recorded_wallet == expected_wallet)
        .return_once(|_| Ok(Some(Nonce(6))));

    let task = mk_task_with_provider(
        mk_mock_provider_with_pending_nonce(9),
        Arc::new(store),
        BTreeMap::new(),
    );

    let nonce = task
        .assign_next_nonce_for_wallet(wallet)
        .await
        .expect("nonce assignment should succeed");

    assert_eq!(nonce, Nonce(9));
}

#[tokio::test]
async fn save_attempt_to_db_records_attempt_in_storage_and_memory() {
    let mut store = MockStateStore::new();
    let job_id = mk_job_id(1);
    let wallet = Address::from([2; 20]);
    let expected_wallet: agglayer_types::Address = wallet.into();
    let nonce = Nonce(7);
    let attempt_number = SettlementAttemptNumber(3);
    let tx = mk_tx(4);
    let tx_hash = SettlementTxHash::from(Digest::from(*tx.tx_hash()));
    let earliest_submission_time = SystemTime::now();

    store
        .expect_insert_settlement_attempt()
        .once()
        .withf(
            move |recorded_job_id, recorded_attempt_number, recorded_attempt| {
                recorded_job_id == &job_id
                    && *recorded_attempt_number == attempt_number.0
                    && recorded_attempt.sender_wallet == expected_wallet
                    && recorded_attempt.nonce == nonce
                    && recorded_attempt.hash == tx_hash
                    && recorded_attempt.submission_time >= earliest_submission_time
                    && recorded_attempt.max_fee_per_gas == 100
                    && recorded_attempt.max_priority_fee_per_gas == 10
            },
        )
        .return_once(|_, _, _| Ok(()));

    let mut task = mk_task(Arc::new(store), BTreeMap::new());

    task.save_attempt_to_db(wallet, nonce, attempt_number, &tx);

    let active_attempt = task
        .attempts
        .get(&(wallet, nonce))
        .and_then(|attempts_for_nonce| attempts_for_nonce.get(&attempt_number))
        .expect("attempt should be tracked in memory");

    assert_eq!(active_attempt.attempt.sender_wallet, wallet.into());
    assert_eq!(active_attempt.attempt.nonce, nonce);
    assert_eq!(active_attempt.attempt.hash, tx_hash);
    assert!(active_attempt.result.is_none());
    assert_eq!(active_attempt.attempt.max_fee_per_gas, 100);
    assert_eq!(active_attempt.attempt.max_priority_fee_per_gas, 10);
}

#[tokio::test]
async fn save_attempt_to_db_does_not_track_attempt_when_storage_write_fails() {
    let mut store = MockStateStore::new();
    store
        .expect_insert_settlement_attempt()
        .once()
        .return_once(|_, _, _| Err(Error::Unexpected("injected storage failure".to_string())));

    let wallet = Address::from([2; 20]);
    let nonce = Nonce(7);
    let tx = mk_tx(4);
    let mut task = mk_task(Arc::new(store), BTreeMap::new());

    let result = catch_unwind(AssertUnwindSafe(|| {
        task.save_attempt_to_db(wallet, nonce, SettlementAttemptNumber(3), &tx);
    }));

    assert!(result.is_err());
    assert!(task.attempts.is_empty());
}

#[tokio::test]
async fn save_attempt_to_db_rejects_attempt_number_already_tracked_for_other_nonce() {
    let store = MockStateStore::new();
    let existing_wallet = Address::from([1; 20]);
    let existing_nonce = Nonce(7);
    let new_wallet = Address::from([2; 20]);
    let new_nonce = Nonce(8);
    let attempt_number = SettlementAttemptNumber(3);
    let existing_hash = mk_tx_hash(9);

    let attempts = BTreeMap::from([(
        (existing_wallet, existing_nonce),
        BTreeMap::from([(
            attempt_number,
            mk_active_attempt(existing_wallet, existing_nonce, existing_hash, None),
        )]),
    )]);
    let mut task = mk_task(Arc::new(store), attempts);
    let tx = mk_tx(4);

    let result = catch_unwind(AssertUnwindSafe(|| {
        task.save_attempt_to_db(new_wallet, new_nonce, attempt_number, &tx);
    }));

    assert!(result.is_err());
    assert_eq!(task.attempts.len(), 1);
    assert_eq!(
        task.attempts[&(existing_wallet, existing_nonce)][&attempt_number]
            .attempt
            .hash,
        existing_hash
    );
    assert!(!task.attempts.contains_key(&(new_wallet, new_nonce)));
}

#[tokio::test]
async fn record_attempt_result_keeps_revert_over_conflicting_write() {
    // Regression (#1607): a stored revert must survive a later "settled elsewhere"
    // write instead of panicking (which used to repeat on every restart).
    let wallet = Address::from([2; 20]);
    let nonce = Nonce(7);
    let attempt_number = SettlementAttemptNumber(3);
    let revert = SettlementAttemptResult::ContractCall(mk_contract_call_result(
        1,
        ContractCallOutcome::Revert,
    ));

    let attempts = BTreeMap::from([(
        (wallet, nonce),
        BTreeMap::from([(
            attempt_number,
            mk_active_attempt(wallet, nonce, mk_tx_hash(1), Some(revert.clone())),
        )]),
    )]);

    // The conflicting write is dropped, so the store is never called.
    let mut task = mk_task(Arc::new(MockStateStore::new()), attempts);

    task.record_attempt_result_to_db(
        attempt_number,
        SettlementAttemptResult::ClientError(ClientError::settlement_succeeded_elsewhere(
            mk_tx_hash(9),
        )),
    );

    assert_eq!(
        task.attempts[&(wallet, nonce)][&attempt_number].result,
        Some(revert)
    );
}

#[tokio::test]
async fn is_wallet_privkey_known_true_for_configured_wallet() {
    let task = mk_task(Arc::new(MockStateStore::new()), BTreeMap::new());
    assert!(task.is_wallet_privkey_known(test_signer().address()));
}

#[tokio::test]
async fn is_wallet_privkey_known_false_for_unknown_wallet() {
    let task = mk_task(Arc::new(MockStateStore::new()), BTreeMap::new());
    assert!(!task.is_wallet_privkey_known(Address::repeat_byte(0xAB)));
}

#[tokio::test]
async fn write_job_result_records_success_and_marks_other_attempts() {
    let wallet = Address::from([1; 20]);
    let other_wallet = Address::from([2; 20]);
    let nonce = Nonce(7);
    let other_nonce = Nonce(8);
    let attempt_number = SettlementAttemptNumber(1);
    let sibling_attempt_number = SettlementAttemptNumber(2);
    let other_attempt_number = SettlementAttemptNumber(3);
    let tx_result = mk_contract_call_result(10, ContractCallOutcome::Success);
    let expected_wallet: agglayer_types::Address = wallet.into();
    let expected_tx_result = tx_result.clone();

    let mut attempts = BTreeMap::new();
    attempts.insert(
        (wallet, nonce),
        BTreeMap::from([
            (
                attempt_number,
                mk_active_attempt(wallet, nonce, tx_result.tx_hash, None),
            ),
            (
                sibling_attempt_number,
                mk_active_attempt(wallet, nonce, mk_tx_hash(20), None),
            ),
        ]),
    );
    attempts.insert(
        (other_wallet, other_nonce),
        BTreeMap::from([(
            other_attempt_number,
            mk_active_attempt(other_wallet, other_nonce, mk_tx_hash(30), None),
        )]),
    );

    let mut store = MockStateStore::new();
    store
        .expect_record_settlement_attempt_result()
        .times(3)
        .returning(|_, _, _| Ok(()));
    store
        .expect_insert_settlement_job_result()
        .once()
        .withf(move |_, result| {
            result.wallet == expected_wallet
                && result.nonce == nonce
                && result.attempt_number == attempt_number
                && result.contract_call_result == expected_tx_result
        })
        .returning(|_, _| Ok(()));

    let mut task = mk_task(Arc::new(store), attempts);

    let job_result = task
        .write_job_result_to_db(wallet, nonce, attempt_number, tx_result.clone())
        .await;

    assert_eq!(job_result.contract_call_result, tx_result);
    assert!(matches!(
        task.attempts[&(wallet, nonce)][&attempt_number]
            .result
            .as_ref(),
        Some(SettlementAttemptResult::ContractCall(_))
    ));
    assert!(matches!(
        task.attempts[&(wallet, nonce)][&sibling_attempt_number]
            .result
            .as_ref(),
        Some(SettlementAttemptResult::ClientError(ClientError {
            kind: ClientErrorType::NonceAlreadyUsed,
            ..
        }))
    ));
    assert!(matches!(
        task.attempts[&(other_wallet, other_nonce)][&other_attempt_number]
            .result
            .as_ref(),
        Some(SettlementAttemptResult::ClientError(ClientError {
            kind: ClientErrorType::SettlementSucceededElsewhere,
            ..
        }))
    ));
}

#[tokio::test]
async fn write_nonce_revert_replaces_previous_client_error_for_finalized_attempt() {
    let wallet = Address::from([3; 20]);
    let nonce = Nonce(9);
    let attempt_number = SettlementAttemptNumber(1);
    let sibling_attempt_number = SettlementAttemptNumber(2);
    let tx_result = mk_contract_call_result(40, ContractCallOutcome::Revert);

    let attempts = BTreeMap::from([(
        (wallet, nonce),
        BTreeMap::from([
            (
                attempt_number,
                mk_active_attempt(
                    wallet,
                    nonce,
                    tx_result.tx_hash,
                    Some(SettlementAttemptResult::ClientError(ClientError {
                        kind: ClientErrorType::Unknown,
                        message: "submission failed".to_string(),
                    })),
                ),
            ),
            (
                sibling_attempt_number,
                mk_active_attempt(wallet, nonce, mk_tx_hash(50), None),
            ),
        ]),
    )]);

    let mut store = MockStateStore::new();
    store
        .expect_record_settlement_attempt_result()
        .times(2)
        .returning(|_, _, _| Ok(()));

    let mut task = mk_task(Arc::new(store), attempts);

    task.write_nonce_revert_to_db(wallet, nonce, attempt_number, tx_result.clone())
        .await;

    assert_eq!(
        task.attempts[&(wallet, nonce)][&attempt_number]
            .result
            .as_ref(),
        Some(&SettlementAttemptResult::ContractCall(tx_result))
    );
    assert!(matches!(
        task.attempts[&(wallet, nonce)][&sibling_attempt_number]
            .result
            .as_ref(),
        Some(SettlementAttemptResult::ClientError(ClientError {
            kind: ClientErrorType::NonceAlreadyUsed,
            ..
        }))
    ));
}

fn mk_rpc_block(number: u64, hash: B256) -> alloy::rpc::types::Block {
    let mut block: alloy::rpc::types::Block = Default::default();
    block.header.hash = hash;
    block.header.inner.number = number;
    block
}

fn mk_rpc_receipt(
    tx_hash: SettlementTxHash,
    block_hash: B256,
    block_number: u64,
) -> alloy::rpc::types::TransactionReceipt {
    alloy::rpc::types::TransactionReceipt {
        inner: alloy::consensus::ReceiptEnvelope::Eip1559(alloy::consensus::ReceiptWithBloom {
            receipt: alloy::consensus::Receipt {
                status: true.into(),
                cumulative_gas_used: 0,
                logs: vec![],
            },
            logs_bloom: Default::default(),
        }),
        transaction_hash: tx_hash.into(),
        transaction_index: Some(0),
        block_hash: Some(block_hash),
        block_number: Some(block_number),
        gas_used: 0,
        effective_gas_price: 0,
        blob_gas_used: None,
        blob_gas_price: None,
        from: Address::from([9; 20]),
        to: None,
        contract_address: None,
    }
}

/// Attempts of a job whose completion was interrupted right after the
/// winning attempt's result write: the winner carries `stored_result`,
/// the same-nonce sibling and the other-wallet attempt are unresolved.
fn mk_interrupted_completion_attempts(
    wallet: Address,
    nonce: Nonce,
    other_wallet: Address,
    other_nonce: Nonce,
    stored_result: &ContractCallResult,
) -> ActiveSettlementAttempts {
    let mut attempts = BTreeMap::new();
    attempts.insert(
        (wallet, nonce),
        BTreeMap::from([
            (
                SettlementAttemptNumber(1),
                mk_active_attempt(
                    wallet,
                    nonce,
                    stored_result.tx_hash,
                    Some(SettlementAttemptResult::ContractCall(stored_result.clone())),
                ),
            ),
            (
                SettlementAttemptNumber(2),
                mk_active_attempt(wallet, nonce, mk_tx_hash(70), None),
            ),
        ]),
    );
    attempts.insert(
        (other_wallet, other_nonce),
        BTreeMap::from([(
            SettlementAttemptNumber(3),
            mk_active_attempt(other_wallet, other_nonce, mk_tx_hash(80), None),
        )]),
    );
    attempts
}

fn mk_rpc_transaction(
    tx: TxEnvelope,
    from: Address,
    block_number: u64,
) -> alloy::rpc::types::Transaction {
    alloy::rpc::types::Transaction {
        inner: alloy::consensus::transaction::Recovered::new_unchecked(tx, from),
        block_hash: Some(B256::from([2; 32])),
        block_number: Some(block_number),
        transaction_index: Some(0),
        effective_gas_price: Some(0),
    }
}

#[tokio::test]
async fn run_finishes_interrupted_completion_before_other_nonces() {
    // The other wallet sorts before the winner: without the processing
    // order it would be handled first and consume the mocked responses.
    let wallet = Address::from([4; 20]);
    let other_wallet = Address::from([3; 20]);
    let nonce = Nonce(11);
    let other_nonce = Nonce(12);
    let attempt_number = SettlementAttemptNumber(1);
    let block_hash = B256::from([7; 32]);
    let block_number = 10;
    let stored_result = ContractCallResult {
        outcome: ContractCallOutcome::Success,
        metadata: Default::default(),
        block_hash,
        block_number,
        tx_hash: mk_tx_hash(60),
    };
    let expected_wallet: agglayer_types::Address = wallet.into();
    let expected_tx_result = stored_result.clone();

    let attempts = mk_interrupted_completion_attempts(
        wallet,
        nonce,
        other_wallet,
        other_nonce,
        &stored_result,
    );

    // The loop replays its normal success checks on the winning nonce:
    // the mined transaction for the nonce, its receipt, then the
    // settlement check (safe head, receipt again, canonical block).
    let asserter = Asserter::new();
    asserter.push_success(&mk_rpc_transaction(mk_tx(60), wallet, block_number));
    asserter.push_success(&mk_rpc_receipt(
        stored_result.tx_hash,
        block_hash,
        block_number,
    ));
    asserter.push_success(&mk_rpc_block(1_000, B256::from([1; 32])));
    asserter.push_success(&mk_rpc_receipt(
        stored_result.tx_hash,
        block_hash,
        block_number,
    ));
    asserter.push_success(&mk_rpc_block(block_number, block_hash));
    let provider = ProviderBuilder::new()
        .wallet(EthereumWallet::from(test_signer()))
        .connect_mocked_client(asserter);

    let mut store = MockStateStore::new();
    // Only the two unresolved attempts get a store write; the winner's
    // identical re-record no-ops in memory.
    store
        .expect_record_settlement_attempt_result()
        .times(2)
        .returning(|_, _, _| Ok(()));
    store
        .expect_insert_settlement_job_result()
        .once()
        .withf(move |_, result| {
            result.wallet == expected_wallet
                && result.nonce == nonce
                && result.attempt_number == attempt_number
                && result.contract_call_result == expected_tx_result
        })
        .returning(|_, _| Ok(()));

    let cancellation_token = CancellationToken::new();
    let (_control_handle, control) = TaskControlHandle::new(&cancellation_token);
    let mut task = SettlementTask {
        id: mk_job_id(1),
        job: mk_job(),
        tx_config: Arc::new(SettlementTransactionConfig::default()),
        provider: Arc::new(provider),
        store: Arc::new(store),
        wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
        control,
        attempts,
    };

    let run_result = tokio::time::timeout(Duration::from_secs(30), task.run())
        .await
        .expect("the interrupted completion must finish without further L1 events");

    let SettlementTaskRunResult::Completed(job_result) = run_result else {
        panic!("expected the run to complete the job");
    };
    assert_eq!(job_result.contract_call_result, stored_result);
    assert!(matches!(
        task.attempts[&(wallet, nonce)][&SettlementAttemptNumber(2)]
            .result
            .as_ref(),
        Some(SettlementAttemptResult::ClientError(ClientError {
            kind: ClientErrorType::NonceAlreadyUsed,
            ..
        }))
    ));
    assert!(matches!(
        task.attempts[&(other_wallet, other_nonce)][&SettlementAttemptNumber(3)]
            .result
            .as_ref(),
        Some(SettlementAttemptResult::ClientError(ClientError {
            kind: ClientErrorType::SettlementSucceededElsewhere,
            ..
        }))
    ));
}

#[test]
fn nonces_in_processing_order_puts_recorded_success_first() {
    let first_wallet = Address::from([3; 20]);
    let winner_wallet = Address::from([4; 20]);
    let success = mk_contract_call_result(60, ContractCallOutcome::Success);
    let revert = mk_contract_call_result(90, ContractCallOutcome::Revert);

    let attempts = BTreeMap::from([
        (
            (first_wallet, Nonce(12)),
            BTreeMap::from([(
                SettlementAttemptNumber(3),
                mk_active_attempt(first_wallet, Nonce(12), mk_tx_hash(80), None),
            )]),
        ),
        (
            (winner_wallet, Nonce(11)),
            BTreeMap::from([
                (
                    SettlementAttemptNumber(1),
                    mk_active_attempt(
                        winner_wallet,
                        Nonce(11),
                        success.tx_hash,
                        Some(SettlementAttemptResult::ContractCall(success)),
                    ),
                ),
                (
                    SettlementAttemptNumber(2),
                    mk_active_attempt(winner_wallet, Nonce(11), mk_tx_hash(70), None),
                ),
            ]),
        ),
    ]);
    let task = mk_task(Arc::new(MockStateStore::new()), attempts);
    assert_eq!(
        task.nonces_in_processing_order(),
        vec![(winner_wallet, Nonce(11)), (first_wallet, Nonce(12))]
    );

    // Without a recorded success (a revert or client error is not one),
    // the natural nonce order is kept.
    let attempts = BTreeMap::from([
        (
            (first_wallet, Nonce(12)),
            BTreeMap::from([(
                SettlementAttemptNumber(3),
                mk_active_attempt(first_wallet, Nonce(12), mk_tx_hash(80), None),
            )]),
        ),
        (
            (winner_wallet, Nonce(11)),
            BTreeMap::from([(
                SettlementAttemptNumber(1),
                mk_active_attempt(
                    winner_wallet,
                    Nonce(11),
                    revert.tx_hash,
                    Some(SettlementAttemptResult::ContractCall(revert)),
                ),
            )]),
        ),
    ]);
    let task = mk_task(Arc::new(MockStateStore::new()), attempts);
    assert_eq!(
        task.nonces_in_processing_order(),
        vec![(first_wallet, Nonce(12)), (winner_wallet, Nonce(11))]
    );
}

#[test]
fn required_settlement_head_number_is_inclusive_of_receipt_block() {
    // Confirmations count the receipt block itself, and saturate rather than
    // overflow.
    for (receipt_block, confirmations, required_head) in [
        (10, 0, 10),
        (10, 1, 10),
        (10, 12, 21),
        (10, usize::MAX, u64::MAX),
    ] {
        assert_eq!(
            required_settlement_head_number(receipt_block, confirmations),
            required_head
        );
    }
}

#[test]
fn hydrate_settlement_attempts_groups_attempts_and_results() {
    let job_id = SettlementJobId::from(1u128);
    let wallet = Address::repeat_byte(2);
    let other_wallet = Address::repeat_byte(3);
    let nonce = Nonce(7);
    let other_nonce = Nonce(8);
    let pending_attempt = mk_stored_attempt(1, wallet, nonce);
    let completed_attempt = mk_stored_attempt(2, wallet, nonce);
    let other_attempt = mk_stored_attempt(3, other_wallet, other_nonce);
    let completed_result = mk_client_error(4);

    let hydrated_attempts = hydrate_settlement_attempts(
        vec![
            (1, pending_attempt.clone()),
            (2, completed_attempt.clone()),
            (3, other_attempt.clone()),
        ],
        vec![(2, completed_result.clone())],
        job_id,
    )
    .expect("stored attempts should hydrate");

    let attempts_for_nonce = hydrated_attempts
        .get(&(wallet, nonce))
        .expect("wallet nonce should be loaded");
    assert_eq!(attempts_for_nonce.len(), 2);
    let loaded_pending = attempts_for_nonce
        .get(&SettlementAttemptNumber(1))
        .expect("pending attempt should be loaded");
    assert_eq!(loaded_pending.attempt, pending_attempt);
    assert_eq!(loaded_pending.result, None);
    let loaded_completed = attempts_for_nonce
        .get(&SettlementAttemptNumber(2))
        .expect("completed attempt should be loaded");
    assert_eq!(loaded_completed.attempt, completed_attempt);
    assert_eq!(loaded_completed.result.as_ref(), Some(&completed_result));

    let attempts_for_other_nonce = hydrated_attempts
        .get(&(other_wallet, other_nonce))
        .expect("other wallet nonce should be loaded");
    let loaded_other = attempts_for_other_nonce
        .get(&SettlementAttemptNumber(3))
        .expect("other attempt should be loaded");
    assert_eq!(loaded_other.attempt, other_attempt);
    assert_eq!(loaded_other.result, None);
}

#[test]
fn load_settlement_attempts_from_db_hydrates_attempts_and_results() {
    let job_id = SettlementJobId::from(1u128);
    let wallet = Address::repeat_byte(2);
    let nonce = Nonce(7);
    let pending_attempt = mk_stored_attempt(1, wallet, nonce);
    let completed_attempt = mk_stored_attempt(2, wallet, nonce);
    let completed_result = mk_client_error(4);

    let attempts_for_store = vec![(1, pending_attempt.clone()), (2, completed_attempt.clone())];
    let completed_result_for_store = completed_result.clone();
    let mut store = MockStateStore::new();
    let expected_job_id = job_id;
    store
        .expect_list_settlement_attempt_results()
        .once()
        .withf(move |requested_job_id| requested_job_id == &expected_job_id)
        .return_once(move |_| Ok(vec![(2, completed_result_for_store)]));
    let expected_job_id = job_id;
    store
        .expect_list_settlement_attempts()
        .once()
        .withf(move |requested_job_id| requested_job_id == &expected_job_id)
        .return_once(move |_| Ok(attempts_for_store));

    let mut task = mk_task_with_id(job_id, Arc::new(store), BTreeMap::new());

    task.load_settlement_attempts_from_db()
        .expect("stored attempts should hydrate");

    let attempts_for_nonce = task
        .attempts
        .get(&(wallet, nonce))
        .expect("wallet nonce should be loaded");
    assert_eq!(attempts_for_nonce.len(), 2);
    let loaded_pending = attempts_for_nonce
        .get(&SettlementAttemptNumber(1))
        .expect("pending attempt should be loaded");
    assert_eq!(loaded_pending.attempt, pending_attempt);
    assert_eq!(loaded_pending.result, None);
    let loaded_completed = attempts_for_nonce
        .get(&SettlementAttemptNumber(2))
        .expect("completed attempt should be loaded");
    assert_eq!(loaded_completed.attempt, completed_attempt);
    assert_eq!(loaded_completed.result.as_ref(), Some(&completed_result));
}

#[test]
fn hydrate_settlement_attempts_rejects_result_without_attempt() {
    let error = hydrate_settlement_attempts(
        std::iter::empty::<(u64, SettlementAttempt)>(),
        vec![(7, mk_client_error(5))],
        SettlementJobId::from(2u128),
    )
    .err()
    .expect("orphaned attempt result should fail hydration");

    assert!(error
        .to_string()
        .contains("without a recorded settlement attempt"));
}

#[tokio::test]
async fn wait_for_any_nonce_on_l1_returns_when_a_pending_nonce_is_included() {
    let anvil = Anvil::new().spawn();
    let sender = anvil.addresses()[0];
    let provider = build_provider(&anvil);
    provider
        .send_transaction(TransactionRequest::default().to(anvil.addresses()[1]))
        .await
        .expect("send transaction")
        .get_receipt()
        .await
        .expect("get receipt");

    // The zero address sorts first and is never mined, so the wait must skip it
    // and still resolve on the mined (sender, Nonce(0)).
    let pending = BTreeSet::from([(Address::ZERO, Nonce(0)), (sender, Nonce(0))]);
    let task = mk_task_with_tx_config(provider, SettlementTransactionConfig::default());

    tokio::time::timeout(
        Duration::from_secs(5),
        task.wait_for_any_nonce_on_l1(&pending),
    )
    .await
    .expect("wait should return once a pending nonce is included");
}

#[tokio::test]
async fn wait_for_any_nonce_on_l1_keeps_waiting_when_no_pending_nonce_is_included() {
    let anvil = Anvil::new().spawn();
    let sender = anvil.addresses()[0];
    let provider = build_provider(&anvil);

    // No matching tx is mined. A large non-inclusion interval keeps the first
    // poll's backoff longer than the timeout. `start_paused` is unusable here:
    // Anvil uses real I/O and the paused clock would auto-advance the sleep.
    let mut tx_config = SettlementTransactionConfig::default();
    tx_config.retry_on_not_included_on_l1.initial_interval = Duration::from_secs(3600);

    let pending = BTreeSet::from([(sender, Nonce(5))]);
    let task = mk_task_with_tx_config(provider, tx_config);

    assert!(tokio::time::timeout(
        Duration::from_millis(300),
        task.wait_for_any_nonce_on_l1(&pending)
    )
    .await
    .is_err());
}

#[tokio::test]
async fn current_result_once_returns_result_for_mined_tx() {
    let anvil = Anvil::new().spawn();
    let sender = anvil.addresses()[0];
    let provider = build_provider(&anvil);
    let receipt = provider
        .send_transaction(TransactionRequest::default().to(anvil.addresses()[1]))
        .await
        .expect("send transaction")
        .get_receipt()
        .await
        .expect("get receipt");
    let tx_hash = SettlementTxHash::from(receipt.transaction_hash);

    let task = mk_task_with_tx_config(provider, SettlementTransactionConfig::default());
    let result = task
        .current_result_once(sender, Nonce(0), tx_hash)
        .await
        .expect("query should succeed")
        .expect("mined tx should have a result");

    assert_eq!(result.outcome, ContractCallOutcome::Success);
    assert_eq!(result.tx_hash, tx_hash);
    assert_eq!(Some(result.block_number), receipt.block_number);
    assert!(result.metadata.is_empty());
}

#[tokio::test]
async fn current_result_once_reports_none_when_nonce_no_longer_maps() {
    // A receipt is missing and the nonce no longer resolves to this tx, so it
    // was reorged out (here: never mined). The lag branch -- receipt missing
    // but the nonce still maps -- needs a node that is inconsistent between
    // `eth_getTransactionReceipt` and sender+nonce lookup, which anvil is not.
    let anvil = Anvil::new().arg("--no-mining").spawn();
    let sender = anvil.addresses()[0];
    let provider = build_provider(&anvil);
    let pending = provider
        .send_transaction(TransactionRequest::default().to(anvil.addresses()[1]))
        .await
        .expect("send transaction");
    let tx_hash = SettlementTxHash::from(*pending.tx_hash());

    let task = mk_task_with_tx_config(provider, SettlementTransactionConfig::default());
    let result = task
        .current_result_once(sender, Nonce(0), tx_hash)
        .await
        .expect("query should succeed");

    assert_eq!(result, None);
}

#[test]
fn submission_outcome_reports_success() {
    assert!(submission_outcome(Ok(())).is_ok());
}

#[test]
fn submission_outcome_treats_cancellation_as_cancelled() {
    // A shutdown mid-retry must surface as cancellation so the caller leaves
    // the already-saved attempt pending and stops the runner, rather than
    // recording a client error or silently continuing as success.
    assert!(matches!(
        submission_outcome(Err(RetryCallbackError::Cancelled)),
        Err(SubmitAttemptError::Cancelled)
    ));
}

#[test]
fn submission_outcome_reports_transport_error_as_failed() {
    let error = RetryCallbackError::Error(TransportErrorKind::custom_str("boom"));
    assert!(matches!(
        submission_outcome(Err(error)),
        Err(SubmitAttemptError::Failed(_))
    ));
}

#[tokio::test]
async fn submit_attempt_to_l1_broadcasts_signed_envelope() {
    let anvil = Anvil::new().spawn();
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let provider = ProviderBuilder::new()
        .wallet(EthereumWallet::from(signer))
        .connect_http(anvil.endpoint_url());

    // Build and sign a transaction envelope through the provider's fillers so
    // it carries a valid nonce, gas, and chain id, then hand it off as the
    // settlement attempt to submit.
    let tx_request = TransactionRequest::default()
        .to(anvil.addresses()[1])
        .value(alloy::primitives::U256::from(1));
    let envelope = provider
        .fill(tx_request)
        .await
        .expect("filling the settlement transaction should succeed")
        .try_into_envelope()
        .expect("a wallet-filled transaction should be a signed envelope");
    let expected_tx_hash: TxHash = *envelope.tx_hash();

    let task = SettlementTask {
        id: mk_job_id(1),
        job: mk_job(),
        tx_config: Arc::new(SettlementTransactionConfig::default()),
        provider: Arc::new(provider),
        store: Arc::new(MockStateStore::new()),
        wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
        control: mk_control(),
        attempts: BTreeMap::new(),
    };

    task.submit_attempt_to_l1(envelope)
        .await
        .expect("submitting the settlement attempt should succeed");

    // The helper does not wait for inclusion, but the node must have accepted
    // the broadcast, so it should know the transaction by hash.
    let broadcast_tx = task
        .provider
        .get_transaction_by_hash(expected_tx_hash)
        .await
        .expect("querying the broadcast transaction should succeed");
    assert!(
        broadcast_tx.is_some(),
        "the node should know the broadcast settlement transaction"
    );
}

#[tokio::test]
async fn submit_attempt_to_l1_skips_broadcast_when_already_cancelled() {
    let anvil = Anvil::new().spawn();
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let provider = ProviderBuilder::new()
        .wallet(EthereumWallet::from(signer))
        .connect_http(anvil.endpoint_url());

    let tx_request = TransactionRequest::default()
        .to(anvil.addresses()[1])
        .value(alloy::primitives::U256::from(1));
    let envelope = provider
        .fill(tx_request)
        .await
        .expect("filling the settlement transaction should succeed")
        .try_into_envelope()
        .expect("a wallet-filled transaction should be a signed envelope");
    let expected_tx_hash: TxHash = *envelope.tx_hash();

    let task = SettlementTask {
        id: mk_job_id(1),
        job: mk_job(),
        tx_config: Arc::new(SettlementTransactionConfig::default()),
        provider: Arc::new(provider),
        store: Arc::new(MockStateStore::new()),
        wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
        control: mk_control(),
        attempts: BTreeMap::new(),
    };

    // Request shutdown before submitting: the retry helper only observes the
    // token while backing off, so without an up-front guard the first
    // broadcast would still go out.
    task.control.cancellation_token.cancel();

    let result = task.submit_attempt_to_l1(envelope).await;
    assert!(matches!(result, Err(SubmitAttemptError::Cancelled)));

    // The transaction must never have been broadcast.
    let broadcast_tx = task
        .provider
        .get_transaction_by_hash(expected_tx_hash)
        .await
        .expect("querying the transaction should succeed");
    assert!(
        broadcast_tx.is_none(),
        "a cancelled submission must not broadcast the transaction"
    );
}

/// Tracks the given per-attempt results under one `(wallet, nonce)`, every
/// attempt stamped at `UNIX_EPOCH` so deadlines read as seconds past epoch.
fn attempts_with_results(
    wallet: Address,
    nonce: Nonce,
    results: impl IntoIterator<Item = Option<SettlementAttemptResult>>,
) -> ActiveSettlementAttempts {
    let for_nonce: BTreeMap<_, _> = results
        .into_iter()
        .enumerate()
        .map(|(i, result)| {
            let attempt = mk_active_attempt(wallet, nonce, mk_tx_hash(i as u8), result);
            (SettlementAttemptNumber(i as u64), attempt)
        })
        .collect();
    let mut attempts = ActiveSettlementAttempts::new();
    attempts.insert((wallet, nonce), for_nonce);
    attempts
}

// Per-attempt results → expected seconds after the last submission. The policy
// comes from the *last* attempt's result, then backs off
// `initial * multiplier^(attempts - 1)`, capped at max_interval.
// Defaults: transient 10s/x1.5/120s, non-inclusion 60s/x2/600s.
#[rstest]
#[case::pending(vec![None], 60)]
#[case::rpc_error(vec![Some(mk_client_error(1))], 10)]
#[case::backoff(vec![None, None, None], 240)]
#[case::capped(vec![None; 20], 600)]
#[case::last_attempt_wins(vec![Some(mk_client_error(1)), None], 120)]
fn deadline_is_last_submission_plus_policy_backoff(
    #[case] results: Vec<Option<SettlementAttemptResult>>,
    #[case] expected_secs: u64,
) {
    let wallet = Address::from([9; 20]);
    let nonce = Nonce(0);
    let task = mk_task(
        Arc::new(MockStateStore::new()),
        attempts_with_results(wallet, nonce, results),
    );
    assert_eq!(
        task.next_attempt_deadline_for_nonce(wallet, nonce),
        SystemTime::UNIX_EPOCH + Duration::from_secs(expected_secs),
    );
}

#[test]
fn deadline_without_attempts_is_due_now() {
    let task = mk_task(
        Arc::new(MockStateStore::new()),
        ActiveSettlementAttempts::new(),
    );

    // Nothing tracked for the nonce → due immediately, never in the future.
    let deadline = task.next_attempt_deadline_for_nonce(Address::from([9; 20]), Nonce(0));
    assert!(deadline <= SystemTime::now());
}

#[test]
fn resolve_base_gas_params_applies_multiplier_floor_and_ceiling() {
    let config = SettlementTransactionConfig {
        max_fee_per_gas_multiplier_factor: Multiplier::from_u64_per_1000(1000),
        max_fee_per_gas_floor: 1_000_000_000,    // 1 gwei
        max_fee_per_gas_ceiling: 50_000_000_000, // 50 gwei
        max_priority_fee_per_gas_multiplier_factor: Multiplier::from_u64_per_1000(1000),
        max_priority_fee_per_gas_floor: 2_000_000_000, // 2 gwei
        max_priority_fee_per_gas_ceiling: 50_000_000_000, // 50 gwei
        ..Default::default()
    };

    let mut task = mk_task(Arc::new(MockStateStore::new()), BTreeMap::new());
    task.tx_config = Arc::new(config);
    task.job = SettlementJob {
        gas_limit: 100_000,
        ..mk_job()
    };

    // Estimate above the fee ceiling and below the priority floor.
    let estimate = Eip1559Estimation {
        max_fee_per_gas: 80_000_000_000,       // 80 gwei -> clamps to 50 gwei
        max_priority_fee_per_gas: 100_000_000, // 0.1 gwei -> raised to 2 gwei floor
    };

    let gas = task.resolve_base_gas_params(&estimate);

    // gas_limit passes through the job's gas_limit unchanged.
    assert_eq!(gas.gas_limit, 100_000);
    assert_eq!(gas.max_fee_per_gas, 50_000_000_000);
    assert_eq!(gas.max_priority_fee_per_gas, 2_000_000_000);
}

#[test]
fn resolve_base_gas_params_scales_fees_by_multiplier() {
    // Multipliers scale an estimate that lands strictly inside [floor, ceiling],
    // so this exercises the multiply path (not just clamping).
    let config = SettlementTransactionConfig {
        max_fee_per_gas_multiplier_factor: Multiplier::from_u64_per_1000(1500), // 1.5x
        max_fee_per_gas_floor: 1_000_000_000,                                   // 1 gwei
        max_fee_per_gas_ceiling: 50_000_000_000,                                // 50 gwei
        max_priority_fee_per_gas_multiplier_factor: Multiplier::from_u64_per_1000(1500), // 1.5x
        max_priority_fee_per_gas_floor: 0,
        max_priority_fee_per_gas_ceiling: 50_000_000_000, // 50 gwei
        ..Default::default()
    };

    let mut task = mk_task(Arc::new(MockStateStore::new()), BTreeMap::new());
    task.tx_config = Arc::new(config);

    let estimate = Eip1559Estimation {
        max_fee_per_gas: 10_000_000_000,         // 10 gwei * 1.5 -> 15 gwei
        max_priority_fee_per_gas: 4_000_000_000, // 4 gwei * 1.5 -> 6 gwei
    };

    let gas = task.resolve_base_gas_params(&estimate);

    assert_eq!(gas.max_fee_per_gas, 15_000_000_000);
    assert_eq!(gas.max_priority_fee_per_gas, 6_000_000_000);
}

#[test]
fn resolve_base_gas_params_caps_priority_fee_at_max_fee() {
    // A priority floor above the max-fee ceiling would otherwise produce an
    // invalid `priority > max_fee`; the resolver must cap priority at max_fee.
    let config = SettlementTransactionConfig {
        max_fee_per_gas_multiplier_factor: Multiplier::from_u64_per_1000(1000),
        max_fee_per_gas_floor: 0,
        max_fee_per_gas_ceiling: 50_000_000_000, // 50 gwei
        max_priority_fee_per_gas_multiplier_factor: Multiplier::from_u64_per_1000(1000),
        max_priority_fee_per_gas_floor: 60_000_000_000, // 60 gwei (above max-fee ceiling)
        max_priority_fee_per_gas_ceiling: 100_000_000_000, // 100 gwei
        ..Default::default()
    };

    let mut task = mk_task(Arc::new(MockStateStore::new()), BTreeMap::new());
    task.tx_config = Arc::new(config);

    let estimate = Eip1559Estimation {
        max_fee_per_gas: 70_000_000_000, // -> clamps to 50 gwei ceiling
        max_priority_fee_per_gas: 1_000_000_000, // -> raised to 60 gwei floor, then capped
    };

    let gas = task.resolve_base_gas_params(&estimate);

    assert_eq!(gas.max_fee_per_gas, 50_000_000_000);
    assert_eq!(gas.max_priority_fee_per_gas, gas.max_fee_per_gas);
}

fn bump_config() -> SettlementTransactionConfig {
    // Wide ceilings so the bump path (not clamping) is exercised.
    SettlementTransactionConfig {
        max_fee_per_gas_multiplier_factor: Multiplier::ONE,
        max_fee_per_gas_floor: 0,
        max_fee_per_gas_ceiling: 1_000_000_000_000,
        max_priority_fee_per_gas_multiplier_factor: Multiplier::ONE,
        max_priority_fee_per_gas_floor: 0,
        max_priority_fee_per_gas_ceiling: 1_000_000_000_000,
        ..Default::default()
    }
}

#[test]
fn bump_gas_params_increases_both_fields_at_least_ten_percent() {
    let mut task = mk_task(Arc::new(MockStateStore::new()), BTreeMap::new());
    task.tx_config = Arc::new(bump_config());

    // Fresh estimate below the previous attempt: the prev * 1.10 path wins.
    let estimate = Eip1559Estimation {
        max_fee_per_gas: 1_000_000_000,
        max_priority_fee_per_gas: 100_000_000,
    };
    let bumped = task
        .bump_gas_params(30_000_000_000, 1_000_000_000, &estimate)
        .expect("bump should succeed below ceiling");

    assert!(bumped.max_fee_per_gas >= 33_000_000_000); // 30 gwei * 1.10
    assert!(bumped.max_priority_fee_per_gas >= 1_100_000_000); // 1 gwei * 1.10
    assert!(bumped.max_priority_fee_per_gas <= bumped.max_fee_per_gas);
    // gas_limit comes from the base resolution (job gas_limit, default 1.0x).
    assert_eq!(bumped.gas_limit, 100_000);
}

#[test]
fn bump_gas_params_returns_none_when_max_fee_ceiling_reached() {
    let config = SettlementTransactionConfig {
        max_fee_per_gas_ceiling: 30_000_000_000, // == previous max fee
        max_priority_fee_per_gas_ceiling: 1_000_000_000_000,
        ..bump_config()
    };
    let mut task = mk_task(Arc::new(MockStateStore::new()), BTreeMap::new());
    task.tx_config = Arc::new(config);

    let estimate = Eip1559Estimation {
        max_fee_per_gas: 1_000_000_000,
        max_priority_fee_per_gas: 100_000_000,
    };
    // Previous max fee already sits at the ceiling, so no strict bump exists.
    assert_eq!(
        task.bump_gas_params(30_000_000_000, 1_000_000_000, &estimate),
        None
    );
}

#[test]
fn bump_gas_params_falls_back_to_base_resolution_when_no_previous_fees() {
    // Zero previous fees model the defensive fallback used when no prior
    // attempt is recorded; the bump then degrades to the base resolution of
    // the fresh estimate (required_min = 0, so the fresh value wins).
    let mut task = mk_task(Arc::new(MockStateStore::new()), BTreeMap::new());
    task.tx_config = Arc::new(bump_config());

    let estimate = Eip1559Estimation {
        max_fee_per_gas: 5_000_000_000,
        max_priority_fee_per_gas: 2_000_000_000,
    };
    let bumped = task
        .bump_gas_params(0, 0, &estimate)
        .expect("zero previous degrades to base resolution");

    // bump_config: multiplier 1.0, floor 0, wide ceiling -> base == estimate.
    assert_eq!(bumped.max_fee_per_gas, 5_000_000_000);
    assert_eq!(bumped.max_priority_fee_per_gas, 2_000_000_000);
    assert_eq!(bumped.gas_limit, 100_000);
}

#[test]
fn build_retry_policy_bounds_signer_failures_and_rejects_build_bugs() {
    // `Error::message` produces an opaque `Error::Other`, mirroring how a
    // remote signer backend (e.g. GCP KMS) surfaces a signing failure.
    let signer_failure = || {
        BuildAttemptError::from(TransactionBuilderError::Signer(
            alloy::signers::Error::message("remote signer failure"),
        ))
    };

    // A signer-backend failure is retried up to the bound (to ride out a
    // transient blip), then surfaces as non-recoverable.
    let mut policy = BuildRetryPolicy::new();
    for _ in 0..BuildRetryPolicy::MAX_SIGNER_BUILD_RETRIES {
        assert!(policy.should_retry(&signer_failure()));
    }
    assert!(!policy.should_retry(&signer_failure()));

    // A structural build error is never recoverable by retrying.
    let mut policy = BuildRetryPolicy::new();
    assert!(!policy.should_retry(&BuildAttemptError::from(
        TransactionBuilderError::UnsupportedSignatureType
    )));
}

#[tokio::test]
async fn build_attempt_produces_signed_eip1559_envelope() {
    use alloy::consensus::{transaction::SignerRecoverable as _, Transaction as _};

    let task = mk_task(Arc::new(MockStateStore::new()), BTreeMap::new());
    let wallet_address = test_signer().address();

    let gas = GasParams {
        gas_limit: 100_000,
        max_fee_per_gas: 30_000_000_000,
        max_priority_fee_per_gas: 1_000_000_000,
    };

    let envelope = task
        .build_attempt(wallet_address, Nonce(9), 1337, gas)
        .await
        .expect("attempt should build");

    assert!(matches!(envelope, TxEnvelope::Eip1559(_)));
    assert_eq!(envelope.nonce(), 9);
    assert_eq!(envelope.chain_id(), Some(1337));
    assert_eq!(envelope.gas_limit(), 100_000);
    assert_eq!(envelope.max_fee_per_gas(), 30_000_000_000);
    assert_eq!(envelope.max_priority_fee_per_gas(), Some(1_000_000_000));
    assert_eq!(envelope.value(), mk_job().eth_value);
    assert_eq!(envelope.input().as_ref(), mk_job().calldata.as_ref());
    assert_eq!(envelope.to(), Some(mk_job().contract_address.into_alloy()));
    assert_eq!(envelope.recover_signer().unwrap(), wallet_address);
}

#[tokio::test]
async fn build_next_attempt_with_new_nonce_uses_assigned_nonce_and_default_wallet() {
    use alloy::{
        consensus::Transaction as _, node_bindings::Anvil, providers::ProviderBuilder,
        rpc::types::TransactionRequest,
    };

    let anvil = Anvil::new().spawn();
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet_address = signer.address();
    let provider = ProviderBuilder::new()
        .wallet(EthereumWallet::from(signer))
        .connect_http(anvil.endpoint_url());

    // Bump the sender's nonce to 2 by mining two transactions.
    for _ in 0..2 {
        let tx = TransactionRequest::default()
            .to(anvil.addresses()[1])
            .value(U256::from(1));
        provider
            .send_transaction(tx)
            .await
            .unwrap()
            .get_receipt()
            .await
            .unwrap();
    }

    let expected_wallet: agglayer_types::Address = wallet_address.into();
    let mut store = MockStateStore::new();
    store
        .expect_max_settlement_nonce_for_wallet()
        .once()
        .withf(move |recorded_wallet| *recorded_wallet == expected_wallet)
        .return_once(|_| Ok(Some(Nonce(6))));

    let task = SettlementTask {
        id: mk_job_id(1),
        job: mk_job(),
        tx_config: Arc::new(SettlementTransactionConfig::default()),
        provider: Arc::new(provider),
        store: Arc::new(store),
        wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
        control: mk_control(),
        attempts: BTreeMap::new(),
    };

    let (used_wallet, nonce, attempt_number, envelope) = task
        .build_next_attempt_with_new_nonce()
        .await
        .expect("attempt should build");

    assert_eq!(used_wallet, wallet_address);
    assert_eq!(nonce, Nonce(7));
    assert_eq!(attempt_number, SettlementAttemptNumber(0));
    assert_eq!(envelope.nonce(), 7);
    assert_eq!(envelope.to(), Some(mk_job().contract_address.into_alloy()));
    assert_eq!(envelope.chain_id(), Some(anvil.chain_id()));
    // Fees are within the configured bounds (defaults: floor 0, ceiling 100 gwei).
    assert!(envelope.max_fee_per_gas() <= 100_000_000_000);
}

#[test]
fn polling_signals_skip_the_retry_warning_log() {
    assert!(!WaitForSettlementError::NotIncludedYet.needs_warning_log());
    assert!(!WaitForSettlementError::NotSettledYet.needs_warning_log());
    assert!(
        WaitForSettlementError::Transport(TransportErrorKind::custom_str("connection reset"))
            .needs_warning_log()
    );
}

#[test]
fn bump_fee_enforces_minimum_replacement_bump_with_default_multiplier() {
    // Default multiplier is 1.0; the helper must still bump by >= 10%.
    assert_eq!(bump_fee(100, 0, Multiplier::ONE, 0, u128::MAX), Some(110));
}

#[test]
fn bump_fee_tracks_rising_fresh_estimate() {
    // A higher fresh estimate wins over prev * effective_multiplier.
    assert_eq!(
        bump_fee(100, 200, Multiplier::from_u64_per_1000(1100), 0, u128::MAX),
        Some(200)
    );
}

#[test]
fn bump_fee_returns_none_when_ceiling_forbids_strict_bump() {
    // Ceiling 105 caps below prev * 1.10 = 110, so no valid replacement.
    assert_eq!(bump_fee(100, 0, Multiplier::ONE, 0, 105), None);
}

#[test]
fn bump_fee_applies_floor_and_honours_larger_configured_multiplier() {
    // Floor raises the result above the minimum bump.
    assert_eq!(bump_fee(100, 0, Multiplier::ONE, 500, u128::MAX), Some(500));
    // A configured multiplier larger than the 10% minimum is used as-is.
    assert_eq!(
        bump_fee(100, 0, Multiplier::from_u64_per_1000(2000), 0, u128::MAX),
        Some(200)
    );
}

#[tokio::test]
async fn build_next_attempt_with_nonce_bumps_fees_over_previous_attempt() {
    use alloy::consensus::Transaction as _;

    let anvil = Anvil::new().spawn();
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet_address = signer.address();
    let provider = ProviderBuilder::new()
        .wallet(EthereumWallet::from(signer))
        .connect_http(anvil.endpoint_url());

    // A previous attempt for nonce 4 with known fees to bump from.
    let nonce = Nonce(4);
    let previous = SettlementAttempt {
        sender_wallet: wallet_address.into(),
        nonce,
        hash: mk_tx_hash(1),
        submission_time: SystemTime::UNIX_EPOCH,
        max_fee_per_gas: 30_000_000_000,
        max_priority_fee_per_gas: 1_000_000_000,
    };
    let attempts = BTreeMap::from([(
        (wallet_address, nonce),
        BTreeMap::from([(
            SettlementAttemptNumber(0),
            ActiveSettlementAttempt {
                attempt: previous,
                result: None,
            },
        )]),
    )]);

    let task = SettlementTask {
        id: mk_job_id(1),
        job: mk_job(),
        tx_config: Arc::new(SettlementTransactionConfig::default()),
        provider: Arc::new(provider),
        store: Arc::new(MockStateStore::new()),
        wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
        control: mk_control(),
        attempts,
    };

    let (attempt_number, envelope) = task
        .build_next_attempt_with_nonce(wallet_address, nonce)
        .await
        .expect("build should not fail")
        .expect("bump should produce an attempt below the ceiling");

    assert_eq!(attempt_number, SettlementAttemptNumber(1));
    assert_eq!(envelope.nonce(), 4);
    assert_eq!(envelope.chain_id(), Some(anvil.chain_id()));
    // Strictly bumped by >= 10% over the previous attempt on both fields.
    assert!(envelope.max_fee_per_gas() >= 33_000_000_000);
    assert!(envelope.max_priority_fee_per_gas().unwrap() >= 1_100_000_000);
    assert!(envelope.max_priority_fee_per_gas().unwrap() <= envelope.max_fee_per_gas());
}

#[tokio::test]
async fn build_next_attempt_with_nonce_returns_none_at_ceiling() {
    let anvil = Anvil::new().spawn();
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet_address = signer.address();
    let provider = ProviderBuilder::new()
        .wallet(EthereumWallet::from(signer))
        .connect_http(anvil.endpoint_url());

    let nonce = Nonce(4);
    let previous = SettlementAttempt {
        sender_wallet: wallet_address.into(),
        nonce,
        hash: mk_tx_hash(1),
        submission_time: SystemTime::UNIX_EPOCH,
        max_fee_per_gas: 30_000_000_000,
        max_priority_fee_per_gas: 1_000_000_000,
    };
    let attempts = BTreeMap::from([(
        (wallet_address, nonce),
        BTreeMap::from([(
            SettlementAttemptNumber(0),
            ActiveSettlementAttempt {
                attempt: previous,
                result: None,
            },
        )]),
    )]);

    // The sole attempt is still pending (`result: None` above): a live tx
    // sits in the mempool, and the ceilings are pinned to its fees, so no
    // strict bump is possible and waiting (not re-broadcasting) is correct.
    let config = SettlementTransactionConfig {
        max_fee_per_gas_ceiling: 30_000_000_000,
        max_priority_fee_per_gas_ceiling: 1_000_000_000,
        ..SettlementTransactionConfig::default()
    };

    let task = SettlementTask {
        id: mk_job_id(1),
        job: mk_job(),
        tx_config: Arc::new(config),
        provider: Arc::new(provider),
        store: Arc::new(MockStateStore::new()),
        wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
        control: mk_control(),
        attempts,
    };

    let result = task
        .build_next_attempt_with_nonce(wallet_address, nonce)
        .await
        .expect("build should not fail");
    assert_eq!(result, None);
}

#[tokio::test]
async fn build_next_attempt_with_nonce_rebroadcasts_errored_attempt_at_ceiling() {
    use alloy::consensus::Transaction as _;

    let anvil = Anvil::new().spawn();
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet_address = signer.address();
    let provider = ProviderBuilder::new()
        .wallet(EthereumWallet::from(signer))
        .connect_http(anvil.endpoint_url());

    // The only attempt for this nonce errored on broadcast (no live tx in the
    // mempool) and sits at the fee ceiling. There is nothing to replace, so
    // the task must re-broadcast on the same nonce at freshly-resolved fees
    // rather than stall by bumping past the ceiling.
    let nonce = Nonce(4);
    let previous = SettlementAttempt {
        sender_wallet: wallet_address.into(),
        nonce,
        hash: mk_tx_hash(1),
        submission_time: SystemTime::UNIX_EPOCH,
        max_fee_per_gas: 30_000_000_000,
        max_priority_fee_per_gas: 1_000_000_000,
    };
    let attempts = BTreeMap::from([(
        (wallet_address, nonce),
        BTreeMap::from([(
            SettlementAttemptNumber(0),
            ActiveSettlementAttempt {
                attempt: previous,
                result: Some(mk_client_error(7)),
            },
        )]),
    )]);

    // Ceilings pinned to the previous fees: a strict bump is impossible.
    let config = SettlementTransactionConfig {
        max_fee_per_gas_ceiling: 30_000_000_000,
        max_priority_fee_per_gas_ceiling: 1_000_000_000,
        ..SettlementTransactionConfig::default()
    };

    let task = SettlementTask {
        id: mk_job_id(1),
        job: mk_job(),
        tx_config: Arc::new(config),
        provider: Arc::new(provider),
        store: Arc::new(MockStateStore::new()),
        wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
        control: mk_control(),
        attempts,
    };

    let (attempt_number, envelope) = task
        .build_next_attempt_with_nonce(wallet_address, nonce)
        .await
        .expect("build should not fail")
        .expect("an errored attempt has no live tx to replace, so it must re-broadcast");

    assert_eq!(attempt_number, SettlementAttemptNumber(1));
    assert_eq!(envelope.nonce(), 4);
    assert!(matches!(envelope, TxEnvelope::Eip1559(_)));
    // Re-broadcast at base fees, within the configured ceiling; no strict bump.
    assert!(envelope.max_fee_per_gas() <= 30_000_000_000);
    assert!(envelope.max_priority_fee_per_gas().unwrap() <= envelope.max_fee_per_gas());
}

#[tokio::test]
async fn build_next_attempt_with_nonce_bumps_over_live_tx_ignoring_errored_ceiling_attempt() {
    use alloy::consensus::Transaction as _;

    let anvil = Anvil::new().spawn();
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet_address = signer.address();
    let provider = ProviderBuilder::new()
        .wallet(EthereumWallet::from(signer))
        .connect_http(anvil.endpoint_url());

    let nonce = Nonce(4);
    // Live tx: a pending attempt well below the ceiling.
    let pending = SettlementAttempt {
        sender_wallet: wallet_address.into(),
        nonce,
        hash: mk_tx_hash(1),
        submission_time: SystemTime::UNIX_EPOCH,
        max_fee_per_gas: 10_000_000_000,
        max_priority_fee_per_gas: 1_000_000_000,
    };
    // A newer attempt that errored on broadcast at the ceiling (no live tx).
    let errored = SettlementAttempt {
        sender_wallet: wallet_address.into(),
        nonce,
        hash: mk_tx_hash(2),
        submission_time: SystemTime::UNIX_EPOCH,
        max_fee_per_gas: 30_000_000_000,
        max_priority_fee_per_gas: 1_000_000_000,
    };
    let attempts = BTreeMap::from([(
        (wallet_address, nonce),
        BTreeMap::from([
            (
                SettlementAttemptNumber(0),
                ActiveSettlementAttempt {
                    attempt: pending,
                    result: None,
                },
            ),
            (
                SettlementAttemptNumber(1),
                ActiveSettlementAttempt {
                    attempt: errored,
                    result: Some(mk_client_error(9)),
                },
            ),
        ]),
    )]);

    // Ceiling at the errored attempt's fees: bumping over *it* is impossible,
    // but the live pending tx (10 gwei) can still be out-bid below the ceiling.
    let config = SettlementTransactionConfig {
        max_fee_per_gas_ceiling: 30_000_000_000,
        max_priority_fee_per_gas_ceiling: 30_000_000_000,
        ..SettlementTransactionConfig::default()
    };

    let task = SettlementTask {
        id: mk_job_id(1),
        job: mk_job(),
        tx_config: Arc::new(config),
        provider: Arc::new(provider),
        store: Arc::new(MockStateStore::new()),
        wallet_nonce_locks: Arc::new(WalletNonceLocks::default()),
        control: mk_control(),
        attempts,
    };

    let (attempt_number, envelope) = task
        .build_next_attempt_with_nonce(wallet_address, nonce)
        .await
        .expect("build should not fail")
        .expect("a valid replacement over the live pending tx is possible below the ceiling");

    assert_eq!(attempt_number, SettlementAttemptNumber(2));
    assert_eq!(envelope.nonce(), 4);
    // Bumped >= 10% over the *pending* tx (10 gwei), not the errored 30 gwei one.
    assert!(envelope.max_fee_per_gas() >= 11_000_000_000);
    assert!(envelope.max_fee_per_gas() <= 30_000_000_000);
    assert!(envelope.max_priority_fee_per_gas().unwrap() <= envelope.max_fee_per_gas());
}

#[test]
fn latest_pending_attempt_fees_for_nonce_ignores_errored_and_unknown() {
    let wallet = Address::from([7; 20]);
    let nonce = Nonce(3);
    let errored_only_nonce = Nonce(5);

    // A live pending tx, plus a higher-numbered attempt that errored on
    // broadcast (no live tx) at higher fees.
    let mut pending = mk_active_attempt(wallet, nonce, mk_tx_hash(1), None);
    pending.attempt.max_fee_per_gas = 10_000_000_000;
    pending.attempt.max_priority_fee_per_gas = 1_000_000_000;
    let mut errored_newer =
        mk_active_attempt(wallet, nonce, mk_tx_hash(2), Some(mk_client_error(9)));
    errored_newer.attempt.max_fee_per_gas = 99_000_000_000;
    errored_newer.attempt.max_priority_fee_per_gas = 9_000_000_000;

    // A separate nonce whose only attempt errored.
    let errored_only = mk_active_attempt(
        wallet,
        errored_only_nonce,
        mk_tx_hash(3),
        Some(mk_client_error(2)),
    );

    let attempts = BTreeMap::from([
        (
            (wallet, nonce),
            BTreeMap::from([
                (SettlementAttemptNumber(4), errored_newer),
                (SettlementAttemptNumber(1), pending),
            ]),
        ),
        (
            (wallet, errored_only_nonce),
            BTreeMap::from([(SettlementAttemptNumber(2), errored_only)]),
        ),
    ]);
    let task = mk_task(Arc::new(MockStateStore::new()), attempts);

    // The higher-numbered errored attempt is ignored; the live pending tx wins.
    assert_eq!(
        task.latest_pending_attempt_fees_for_nonce(wallet, nonce),
        Some((10_000_000_000, 1_000_000_000))
    );
    // A nonce whose only attempt errored has no live tx.
    assert_eq!(
        task.latest_pending_attempt_fees_for_nonce(wallet, errored_only_nonce),
        None
    );
    // Unknown nonce -> None.
    assert_eq!(
        task.latest_pending_attempt_fees_for_nonce(wallet, Nonce(999)),
        None
    );
}
