use std::time::SystemTime;

use agglayer_storage::stores::{SettlementReader, SettlementWriter};
use agglayer_types::{
    Address, ClientError as SettlementClientError, ContractCallOutcome, ContractCallResult, Digest,
    Nonce, RpcErrorCode, SettlementAttempt, SettlementAttemptNumber, SettlementAttemptResult,
    SettlementJob, SettlementJobId, SettlementJobResult, SettlementTxHash, B256, U256,
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

    let pending_job_id = SettlementJobId::from(5_u128);
    context
        .state_store
        .insert_settlement_job(&pending_job_id, &settlement_job())
        .unwrap();
    let error = call_error(
        context
            .admin_client
            .request("admin_reloadSettlementTask", rpc_params![pending_job_id])
            .await,
        RpcErrorCode::NoLiveTask,
    );
    insta::assert_snapshot!("admin_reload_settlement_task__no_live_task", error);

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
