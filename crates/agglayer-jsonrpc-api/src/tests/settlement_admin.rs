//! Tests for the settlement task admin RPC methods.

use agglayer_storage::stores::SettlementWriter;
use agglayer_types::{
    ContractCallOutcome, ContractCallResult, Digest, Nonce, SettlementAttemptNumber, SettlementJob,
    SettlementJobId, SettlementJobResult, SettlementTxHash, B256, U256,
};
use jsonrpsee::{core::client::ClientT, rpc_params};

use crate::testutils::TestContext;

fn mk_job_id(seed: u128) -> SettlementJobId {
    SettlementJobId::from(seed)
}

fn mk_job(seed: u8) -> SettlementJob {
    SettlementJob {
        contract_address: agglayer_types::Address::from([seed; 20]),
        calldata: vec![seed, seed.wrapping_add(1)].into(),
        eth_value: U256::from(seed),
        gas_limit: seed as u128 + 100_000,
    }
}

/// Seed one pending settlement job directly in storage. The settlement
/// service does not know about it until reload-and-restart spawns its
/// task.
fn seed_pending_job(context: &TestContext, seed: u8) -> SettlementJobId {
    let job_id = mk_job_id(seed as u128);
    context
        .state_store
        .insert_settlement_job(&job_id, &mk_job(seed))
        .expect("job insert must succeed");
    job_id
}

/// Seed a settlement job with a stored terminal result: a completed job
/// with no live task, as after a settled transaction and a node restart.
fn seed_completed_job(context: &TestContext, seed: u8) -> SettlementJobId {
    let job_id = seed_pending_job(context, seed);
    let result = SettlementJobResult {
        wallet: agglayer_types::Address::from([seed; 20]),
        nonce: Nonce(seed as u64),
        attempt_number: SettlementAttemptNumber(seed as u64),
        contract_call_result: ContractCallResult {
            outcome: ContractCallOutcome::Success,
            metadata: vec![].into(),
            block_hash: B256::from([seed; 32]),
            block_number: seed as u64,
            tx_hash: SettlementTxHash::new(Digest::from([seed; 32])),
        },
    };
    context
        .state_store
        .insert_settlement_job_result(&job_id, &result)
        .expect("job result insert must succeed");
    job_id
}

async fn wait_until_task_gone(context: &TestContext, job_id: SettlementJobId) {
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        while context.settlement_service.has_live_task(job_id).await {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("aborted task should deregister");
}

fn assert_error_code(error: jsonrpsee::core::client::Error, code: i32) {
    match error {
        jsonrpsee::core::client::Error::Call(call_error) => {
            assert_eq!(call_error.code(), code)
        }
        other => panic!("expected a call error, got {other:?}"),
    }
}

#[test_log::test(tokio::test)]
async fn abort_and_reload_settlement_task_round_trip() {
    // Determinism assumption: on the current_thread test runtime the
    // abort's control-map read and cancel() have no await point between
    // them, so the run-loop's reload-arm handle swap cannot interleave
    // and the abort always lands on the task's current control handle.
    // Mirrors the determinism comments in the settlement-service tests.
    let context = TestContext::new_with_config(TestContext::get_default_config()).await;
    let job_id = seed_pending_job(&context, 1);

    // Dead-task path: reload-and-restart spawns the task from storage.
    let () = context
        .admin_client
        .request("admin_reloadAndRestartSettlementTask", rpc_params![job_id])
        .await
        .expect("reload of a pending job must respawn its task");
    assert!(context.settlement_service.has_live_task(job_id).await);

    // Live-task path: a second reload is accepted by the running task.
    let () = context
        .admin_client
        .request("admin_reloadAndRestartSettlementTask", rpc_params![job_id])
        .await
        .expect("reload of a live task must be accepted");

    // Abort stops it.
    let () = context
        .admin_client
        .request("admin_abortSettlementTask", rpc_params![job_id])
        .await
        .expect("abort of a live task must succeed");
    wait_until_task_gone(&context, job_id).await;

    // A second abort reports the dead task.
    let error = context
        .admin_client
        .request::<(), _>("admin_abortSettlementTask", rpc_params![job_id])
        .await
        .expect_err("abort without a live task must fail");
    assert_error_code(error, crate::error::code::SETTLEMENT_ADMIN);

    // And reload-and-restart revives it: the full unstick cycle.
    let () = context
        .admin_client
        .request("admin_reloadAndRestartSettlementTask", rpc_params![job_id])
        .await
        .expect("reload after abort must respawn the task");
    assert!(context.settlement_service.has_live_task(job_id).await);
}

#[test_log::test(tokio::test)]
async fn settlement_task_controls_report_completed_job() {
    let context = TestContext::new_with_config(TestContext::get_default_config()).await;
    let job_id = seed_completed_job(&context, 42);

    for method in [
        "admin_abortSettlementTask",
        "admin_reloadAndRestartSettlementTask",
    ] {
        let error = context
            .admin_client
            .request::<(), _>(method, rpc_params![job_id])
            .await
            .expect_err("completed job must fail");
        assert_error_code(error, crate::error::code::SETTLEMENT_ADMIN);
    }
}

#[test_log::test(tokio::test)]
async fn settlement_task_controls_report_unknown_job() {
    let context = TestContext::new_with_config(TestContext::get_default_config()).await;
    let job_id = mk_job_id(99);

    for method in [
        "admin_abortSettlementTask",
        "admin_reloadAndRestartSettlementTask",
    ] {
        let error = context
            .admin_client
            .request::<(), _>(method, rpc_params![job_id])
            .await
            .expect_err("unknown job must fail");
        assert_error_code(error, crate::error::code::RESOURCE_NOT_FOUND);
    }
}
