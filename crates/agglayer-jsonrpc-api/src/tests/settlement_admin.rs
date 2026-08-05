use std::time::SystemTime;

use agglayer_storage::stores::{SettlementReader, SettlementWriter};
use agglayer_types::{
    Address, ClientError as SettlementClientError, ContractCallOutcome, ContractCallResult, Digest,
    Nonce, RpcErrorCode, SettlementAttempt, SettlementAttemptNumber, SettlementAttemptResult,
    SettlementJob, SettlementJobId, SettlementJobResult, SettlementTxHash, B256, U256,
};
use alloy::{
    network::EthereumWallet,
    providers::{mock::Asserter, ProviderBuilder},
    signers::local::PrivateKeySigner,
};
use jsonrpsee::{
    core::{client::ClientT, ClientError},
    rpc_params,
};

use crate::testutils::TestContext;

fn normalize_report_locations(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::String(message) => {
            const LOCATION_PREFIX: &str = "\n\nLocation:\n    ";

            let Some((chain, location)) = message.rsplit_once(LOCATION_PREFIX) else {
                return;
            };
            let Some((path_and_line, column)) = location.rsplit_once(':') else {
                return;
            };
            let Some((path, line)) = path_and_line.rsplit_once(':') else {
                return;
            };
            if line.parse::<u32>().is_err() || column.parse::<u32>().is_err() {
                return;
            }

            *message = format!("{chain}{LOCATION_PREFIX}{path}:<line>:<column>");
        }
        serde_json::Value::Array(values) => {
            for value in values {
                normalize_report_locations(value);
            }
        }
        serde_json::Value::Object(fields) => {
            for value in fields.values_mut() {
                normalize_report_locations(value);
            }
        }
        _ => {}
    }
}

fn settlement_job() -> SettlementJob {
    SettlementJob {
        contract_address: Address::from([0x12; 20]),
        calldata: vec![0x34, 0x56].into(),
        eth_value: U256::from(0),
        gas_limit: 100_000,
    }
}

fn settlement_result() -> SettlementJobResult {
    SettlementJobResult {
        wallet: Address::from([0x78; 20]),
        nonce: Nonce(1),
        attempt_number: SettlementAttemptNumber(1),
        contract_call_result: ContractCallResult {
            outcome: ContractCallOutcome::Success,
            metadata: vec![0x9a].into(),
            block_hash: B256::from([0xbc; 32]),
            block_number: 1,
            tx_hash: SettlementTxHash::new(Digest::from([0xde; 32])),
        },
    }
}

fn settlement_attempt() -> SettlementAttempt {
    SettlementAttempt {
        sender_wallet: Address::from([0x21; 20]),
        nonce: Nonce(0),
        hash: SettlementTxHash::new(Digest::from([0x43; 32])),
        submission_time: SystemTime::UNIX_EPOCH,
        max_fee_per_gas: 10,
        max_priority_fee_per_gas: 1,
    }
}

fn l1_transaction(seed: u8) -> alloy::rpc::types::Transaction {
    let transaction =
        alloy::consensus::TxEnvelope::Eip1559(alloy::consensus::Signed::new_unhashed(
            alloy::consensus::TxEip1559 {
                chain_id: 1,
                nonce: seed as u64,
                gas_limit: 21_000,
                max_fee_per_gas: seed as u128 + 30,
                max_priority_fee_per_gas: seed as u128 + 3,
                ..Default::default()
            },
            alloy::primitives::Signature::new(U256::from(1), U256::from(2), false),
        ));

    alloy::rpc::types::Transaction {
        inner: alloy::consensus::transaction::Recovered::new_unchecked(
            transaction,
            Address::from([seed; 20]).into(),
        ),
        block_hash: None,
        block_number: None,
        transaction_index: None,
        effective_gas_price: Some(seed as u128 + 30),
    }
}

fn insert_attempt_params(seed: u8) -> serde_json::Value {
    let transaction = l1_transaction(seed);
    serde_json::json!({
        "txHash": SettlementTxHash::from(
            alloy::network::TransactionResponse::tx_hash(&transaction)
        ),
        "senderWallet": Address::from([seed; 20]),
        "nonce": seed as u64,
        "submissionTimeUnixSecs": seed as u64 + 1_700_000_000,
        "maxFeePerGas": seed as u128 + 30,
        "maxPriorityFeePerGas": seed as u128 + 3,
    })
}

async fn context_with_settlement_transactions(
    transactions: impl IntoIterator<Item = Option<alloy::rpc::types::Transaction>>,
) -> TestContext {
    let asserter = Asserter::new();
    for transaction in transactions {
        asserter.push_success(&transaction);
    }
    let settlement_provider = ProviderBuilder::new()
        .wallet(EthereumWallet::from(
            PrivateKeySigner::from_slice(&[0x11; 32]).expect("valid test signing key"),
        ))
        .connect_mocked_client(asserter);
    TestContext::new_with_settlement_provider(
        TestContext::get_default_config(),
        settlement_provider,
    )
    .await
}

fn error_payload(error: ClientError, expected: RpcErrorCode) -> String {
    let error = match error {
        ClientError::Call(error) => error,
        error => panic!("expected JSON-RPC call error, got {error}"),
    };
    assert_eq!(error.code(), expected.code());

    let data: Option<serde_json::Value> = error
        .data()
        .map(|data| serde_json::from_str(data.get()).unwrap());
    let mut payload = serde_json::json!({
        "code": error.code(),
        "message": error.message(),
        "data": data,
    });
    normalize_report_locations(&mut payload);
    serde_json::to_string_pretty(&payload).unwrap()
}

fn call_error(result: Result<(), ClientError>, expected: RpcErrorCode) -> String {
    error_payload(
        result.expect_err("expected JSON-RPC request to fail"),
        expected,
    )
}

fn mutation_error(
    result: Result<serde_json::Value, ClientError>,
    expected: RpcErrorCode,
) -> String {
    error_payload(
        result.expect_err("expected JSON-RPC mutation to fail"),
        expected,
    )
}

#[test_log::test(tokio::test)]
async fn admin_abort_settlement_task_errors_are_classified() {
    let context = TestContext::new_with_config(TestContext::get_default_config()).await;

    let unknown_job_id = SettlementJobId::from(1_u128);
    let error = call_error(
        context
            .admin_client
            .request("admin_abortSettlementTask", rpc_params![unknown_job_id])
            .await,
        RpcErrorCode::NotFound,
    );
    insta::assert_snapshot!("admin_abort_settlement_task__unknown_job", error);

    let pending_job_id = SettlementJobId::from(2_u128);
    context
        .state_store
        .insert_settlement_job(&pending_job_id, &settlement_job())
        .unwrap();
    let error = call_error(
        context
            .admin_client
            .request("admin_abortSettlementTask", rpc_params![pending_job_id])
            .await,
        RpcErrorCode::NoLiveTask,
    );
    insta::assert_snapshot!("admin_abort_settlement_task__no_live_task", error);

    let completed_job_id = SettlementJobId::from(3_u128);
    context
        .state_store
        .insert_settlement_job(&completed_job_id, &settlement_job())
        .unwrap();
    context
        .state_store
        .insert_settlement_job_result(&completed_job_id, &settlement_result())
        .unwrap();
    let error = call_error(
        context
            .admin_client
            .request("admin_abortSettlementTask", rpc_params![completed_job_id])
            .await,
        RpcErrorCode::AlreadyCompleted,
    );
    insta::assert_snapshot!("admin_abort_settlement_task__completed_job", error);
}

#[test_log::test(tokio::test)]
async fn admin_reload_settlement_task_errors_are_classified() {
    let context = TestContext::new_with_config(TestContext::get_default_config()).await;

    let unknown_job_id = SettlementJobId::from(4_u128);
    let error = call_error(
        context
            .admin_client
            .request("admin_reloadSettlementTask", rpc_params![unknown_job_id])
            .await,
        RpcErrorCode::NotFound,
    );
    insta::assert_snapshot!("admin_reload_settlement_task__unknown_job", error);

    let completed_job_id = SettlementJobId::from(6_u128);
    context
        .state_store
        .insert_settlement_job(&completed_job_id, &settlement_job())
        .unwrap();
    context
        .state_store
        .insert_settlement_job_result(&completed_job_id, &settlement_result())
        .unwrap();
    let error = call_error(
        context
            .admin_client
            .request("admin_reloadSettlementTask", rpc_params![completed_job_id])
            .await,
        RpcErrorCode::AlreadyCompleted,
    );
    insta::assert_snapshot!("admin_reload_settlement_task__completed_job", error);
}

#[test_log::test(tokio::test)]
async fn admin_insert_settlement_attempt_round_trips_over_http() {
    let context =
        context_with_settlement_transactions([Some(l1_transaction(1)), Some(l1_transaction(2))])
            .await;
    let job_id = SettlementJobId::from(10_u128);

    context
        .state_store
        .insert_settlement_job(&job_id, &settlement_job())
        .unwrap();

    // All identity fields are explicit, but the service still queries L1 and
    // verifies that sender and nonce match the authoritative transaction.
    let first_response: serde_json::Value = context
        .admin_client
        .request(
            "admin_insertSettlementAttempt",
            rpc_params![job_id, insert_attempt_params(1)],
        )
        .await
        .unwrap();
    assert_eq!(
        first_response,
        serde_json::json!({"attemptNumber": 0, "liveTask": "absent"})
    );

    let second_response: serde_json::Value = context
        .admin_client
        .request(
            "admin_insertSettlementAttempt",
            rpc_params![job_id, insert_attempt_params(2)],
        )
        .await
        .unwrap();
    assert_eq!(
        second_response,
        serde_json::json!({"attemptNumber": 1, "liveTask": "absent"})
    );
}

#[test_log::test(tokio::test)]
async fn admin_insert_settlement_attempt_rejects_invalid_attempt_params() {
    let context = TestContext::new_with_config(TestContext::get_default_config()).await;
    let job_id = SettlementJobId::from(11_u128);
    let missing_tx_hash = serde_json::json!({
        "senderWallet": Address::from([0x11; 20]),
        "nonce": 1,
        "submissionTimeUnixSecs": 1_700_000_001_u64,
        "maxFeePerGas": 31,
        "maxPriorityFeePerGas": 4,
    });
    let mut unknown_field = insert_attempt_params(3);
    unknown_field
        .as_object_mut()
        .expect("attempt params should be an object")
        .insert("unexpected".to_owned(), serde_json::Value::Bool(true));

    for attempt in [missing_tx_hash, unknown_field] {
        let error = context
            .admin_client
            .request::<serde_json::Value, _>(
                "admin_insertSettlementAttempt",
                rpc_params![job_id, attempt],
            )
            .await
            .expect_err("invalid attempt parameters should be rejected");

        match error {
            ClientError::Call(error) => {
                assert_eq!(error.code(), jsonrpsee::types::error::INVALID_PARAMS_CODE);
            }
            error => panic!("expected JSON-RPC call error, got {error}"),
        }
    }
}

#[test_log::test(tokio::test)]
async fn admin_insert_settlement_attempt_errors_are_classified() {
    let context = context_with_settlement_transactions([None, None]).await;

    let unknown_job_id = SettlementJobId::from(12_u128);
    let unknown_error = mutation_error(
        context
            .admin_client
            .request::<serde_json::Value, _>(
                "admin_insertSettlementAttempt",
                rpc_params![unknown_job_id, insert_attempt_params(4)],
            )
            .await,
        RpcErrorCode::NotFound,
    );
    insta::assert_snapshot!(
        "admin_insert_settlement_attempt__unknown_job",
        unknown_error
    );

    let completed_job_id = SettlementJobId::from(13_u128);
    context
        .state_store
        .insert_settlement_job(&completed_job_id, &settlement_job())
        .unwrap();
    context
        .state_store
        .insert_settlement_job_result(&completed_job_id, &settlement_result())
        .unwrap();
    let completed_error = mutation_error(
        context
            .admin_client
            .request::<serde_json::Value, _>(
                "admin_insertSettlementAttempt",
                rpc_params![completed_job_id, insert_attempt_params(5)],
            )
            .await,
        RpcErrorCode::AlreadyCompleted,
    );
    insta::assert_snapshot!(
        "admin_insert_settlement_attempt__completed_job",
        completed_error
    );
}

#[test_log::test(tokio::test)]
async fn admin_insert_settlement_attempt_identity_mismatch_is_invalid_params() {
    let context = context_with_settlement_transactions([Some(l1_transaction(6))]).await;
    let job_id = SettlementJobId::from(14_u128);
    context
        .state_store
        .insert_settlement_job(&job_id, &settlement_job())
        .unwrap();
    let mut attempt = insert_attempt_params(6);
    attempt
        .as_object_mut()
        .expect("attempt params should be an object")
        .insert(
            "senderWallet".to_owned(),
            serde_json::json!(Address::from([0xff; 20])),
        );

    let error = context
        .admin_client
        .request::<serde_json::Value, _>(
            "admin_insertSettlementAttempt",
            rpc_params![job_id, attempt],
        )
        .await
        .expect_err("an identity mismatch should be rejected");

    match error {
        ClientError::Call(error) => {
            assert_eq!(error.code(), RpcErrorCode::InvalidParams.code());
        }
        error => panic!("expected JSON-RPC call error, got {error}"),
    }
}

#[test_log::test(tokio::test)]
async fn admin_attempt_result_mutations_round_trip_over_http() {
    let context = TestContext::new_with_config(TestContext::get_default_config()).await;
    let job_id = SettlementJobId::from(7_u128);
    let reason = "the wallet nonce was burned";

    context
        .state_store
        .insert_settlement_job(&job_id, &settlement_job())
        .unwrap();
    context
        .state_store
        .insert_settlement_attempt(&job_id, 0, &settlement_attempt())
        .unwrap();

    let mark_response: serde_json::Value = context
        .admin_client
        .request(
            "admin_markSettlementAttemptDefinitelyFailed",
            rpc_params![job_id, 0, reason],
        )
        .await
        .unwrap();
    assert_eq!(
        mark_response,
        serde_json::json!({"attemptNumber": 0, "liveTask": "absent"})
    );
    assert_eq!(
        context
            .state_store
            .list_settlement_attempt_results(&job_id)
            .unwrap(),
        vec![(
            0,
            SettlementAttemptResult::ClientError(SettlementClientError::abandoned_by_admin(reason))
        )]
    );

    let remove_response: serde_json::Value = context
        .admin_client
        .request(
            "admin_removeSettlementAttemptResult",
            rpc_params![job_id, 0],
        )
        .await
        .unwrap();
    assert_eq!(
        remove_response,
        serde_json::json!({"attemptNumber": 0, "liveTask": "absent"})
    );
    assert!(context
        .state_store
        .list_settlement_attempt_results(&job_id)
        .unwrap()
        .is_empty());

    context
        .state_store
        .insert_settlement_job_result(&job_id, &settlement_result())
        .unwrap();

    let error = mutation_error(
        context
            .admin_client
            .request::<serde_json::Value, _>(
                "admin_markSettlementAttemptDefinitelyFailed",
                rpc_params![job_id, 0, "cannot edit completed job"],
            )
            .await,
        RpcErrorCode::AlreadyCompleted,
    );
    insta::assert_snapshot!(
        "admin_mark_settlement_attempt_definitely_failed__completed_job",
        error
    );

    let forced_response: serde_json::Value = context
        .admin_client
        .request(
            "admin_markSettlementAttemptDefinitelyFailed",
            rpc_params![job_id, 0, "operator override", "force=true"],
        )
        .await
        .unwrap();
    assert_eq!(
        forced_response,
        serde_json::json!({"attemptNumber": 0, "liveTask": "absent"})
    );
    assert_eq!(
        context
            .state_store
            .list_settlement_attempt_results(&job_id)
            .unwrap(),
        vec![(
            0,
            SettlementAttemptResult::ClientError(SettlementClientError::abandoned_by_admin(
                "operator override"
            ))
        )]
    );
}

#[test_log::test(tokio::test)]
async fn admin_attempt_result_mutation_unknown_jobs_are_not_found() {
    let context = TestContext::new_with_config(TestContext::get_default_config()).await;
    let unknown_job_id = SettlementJobId::from(8_u128);

    let mark_error = mutation_error(
        context
            .admin_client
            .request::<serde_json::Value, _>(
                "admin_markSettlementAttemptDefinitelyFailed",
                rpc_params![unknown_job_id, 0, "unknown job"],
            )
            .await,
        RpcErrorCode::NotFound,
    );
    insta::assert_snapshot!(
        "admin_mark_settlement_attempt_definitely_failed__unknown_job",
        mark_error
    );

    let remove_error = mutation_error(
        context
            .admin_client
            .request::<serde_json::Value, _>(
                "admin_removeSettlementAttemptResult",
                rpc_params![unknown_job_id, 0],
            )
            .await,
        RpcErrorCode::NotFound,
    );
    insta::assert_snapshot!(
        "admin_remove_settlement_attempt_result__unknown_job",
        remove_error
    );
}

#[test_log::test(tokio::test)]
async fn admin_attempt_result_mutation_rejects_unknown_force_literal() {
    let context = TestContext::new_with_config(TestContext::get_default_config()).await;

    let error = context
        .admin_client
        .request::<serde_json::Value, _>(
            "admin_markSettlementAttemptDefinitelyFailed",
            rpc_params![
                SettlementJobId::from(9_u128),
                0,
                "invalid force",
                "force=maybe"
            ],
        )
        .await
        .expect_err("an unknown force literal should be invalid params");

    match error {
        ClientError::Call(error) => {
            assert_eq!(error.code(), jsonrpsee::types::error::INVALID_PARAMS_CODE);
        }
        error => panic!("expected JSON-RPC call error, got {error}"),
    }
}

#[test_log::test(tokio::test)]
async fn admin_force_remove_settlement_job_result_round_trips_over_http() {
    let context = TestContext::new_with_config(TestContext::get_default_config()).await;
    let job_id = SettlementJobId::from(20_u128);
    let job_result = settlement_result();
    let attempt_result =
        SettlementAttemptResult::ContractCall(job_result.contract_call_result.clone());

    context
        .state_store
        .insert_settlement_job(&job_id, &settlement_job())
        .unwrap();
    context
        .state_store
        .insert_settlement_attempt(&job_id, 0, &settlement_attempt())
        .unwrap();
    context
        .state_store
        .record_settlement_attempt_result(&job_id, 0, &attempt_result)
        .unwrap();
    context
        .state_store
        .insert_settlement_job_result(&job_id, &job_result)
        .unwrap();

    let _: () = context
        .admin_client
        .request("admin_forceRemoveSettlementJobResult", rpc_params![job_id])
        .await
        .unwrap();

    assert_eq!(
        context
            .state_store
            .get_settlement_job_result(&job_id)
            .unwrap(),
        None
    );
    assert_eq!(
        context
            .state_store
            .list_settlement_attempt_results(&job_id)
            .unwrap(),
        vec![(0, attempt_result)]
    );
}

#[test_log::test(tokio::test)]
async fn admin_force_remove_settlement_job_result_errors_are_classified() {
    let context = TestContext::new_with_config(TestContext::get_default_config()).await;

    let pending_job_id = SettlementJobId::from(21_u128);
    context
        .state_store
        .insert_settlement_job(&pending_job_id, &settlement_job())
        .unwrap();
    let pending_error = call_error(
        context
            .admin_client
            .request(
                "admin_forceRemoveSettlementJobResult",
                rpc_params![pending_job_id],
            )
            .await,
        RpcErrorCode::NotCompleted,
    );
    insta::assert_snapshot!(
        "admin_force_remove_settlement_job_result__pending_job",
        pending_error
    );

    let unknown_job_id = SettlementJobId::from(22_u128);
    let unknown_error = call_error(
        context
            .admin_client
            .request(
                "admin_forceRemoveSettlementJobResult",
                rpc_params![unknown_job_id],
            )
            .await,
        RpcErrorCode::NotFound,
    );
    insta::assert_snapshot!(
        "admin_force_remove_settlement_job_result__unknown_job",
        unknown_error
    );
}
