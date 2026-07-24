use agglayer_storage::stores::SettlementWriter;
use agglayer_types::{
    Address, ContractCallOutcome, ContractCallResult, Digest, Nonce, RpcErrorCode,
    SettlementAttemptNumber, SettlementJob, SettlementJobId, SettlementJobResult, SettlementTxHash,
    B256, U256,
};
use jsonrpsee::{
    core::{client::ClientT, ClientError},
    rpc_params,
};

use crate::testutils::TestContext;

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

fn call_error(result: Result<(), ClientError>, expected: RpcErrorCode) -> String {
    let error = match result.unwrap_err() {
        ClientError::Call(error) => error,
        error => panic!("expected JSON-RPC call error, got {error}"),
    };
    assert_eq!(error.code(), expected.code());

    let data: Option<serde_json::Value> = error
        .data()
        .map(|data| serde_json::from_str(data.get()).unwrap());
    let payload = serde_json::json!({
        "code": error.code(),
        "message": error.message(),
        "data": data,
    });
    serde_json::to_string_pretty(&payload).unwrap()
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
