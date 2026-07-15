//! Tests for the settlement task admin RPC methods.

use std::time::{Duration, SystemTime};

use agglayer_storage::stores::{SettlementWriter, StateWriter};
use agglayer_types::{
    CertificateId, ClientError, ClientErrorType, ContractCallOutcome, ContractCallResult, Digest,
    Nonce, SettlementAttempt, SettlementAttemptNumber, SettlementAttemptResult, SettlementJob,
    SettlementJobId, SettlementJobResult, SettlementTxHash, B256, U256,
};
use jsonrpsee::{core::client::ClientT, rpc_params};

use crate::{
    settlement_admin::{SettlementJobDetail, SettlementJobStatus, SettlementJobSummary},
    testutils::TestContext,
};

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

fn mk_attempt(seed: u64) -> SettlementAttempt {
    SettlementAttempt {
        sender_wallet: agglayer_types::Address::from([seed as u8; 20]),
        nonce: Nonce(seed),
        hash: SettlementTxHash::new(Digest::from([seed as u8; 32])),
        submission_time: SystemTime::UNIX_EPOCH + Duration::from_secs(seed),
        max_fee_per_gas: 30_000_000_000,
        max_priority_fee_per_gas: 1_000_000_000,
    }
}

#[test_log::test(tokio::test)]
async fn list_settlement_jobs_returns_seeded_jobs() {
    let context = TestContext::new_with_config(TestContext::get_default_config()).await;

    let empty: Vec<SettlementJobSummary> = context
        .admin_client
        .request("admin_listSettlementJobs", rpc_params![])
        .await
        .expect("empty list must succeed");
    assert!(empty.is_empty());

    // A pending job with one errored attempt, linked to a certificate.
    let pending_id = seed_pending_job(&context, 2);
    let certificate_id = CertificateId::new(Digest::from([2u8; 32]));
    context
        .state_store
        .insert_certificate_settlement_job_id(&certificate_id, &pending_id)
        .expect("link insert must succeed");
    context
        .state_store
        .insert_settlement_attempt(&pending_id, 0, &mk_attempt(2))
        .expect("attempt insert must succeed");
    context
        .state_store
        .record_settlement_attempt_result(
            &pending_id,
            0,
            &SettlementAttemptResult::ClientError(ClientError {
                kind: ClientErrorType::Unknown,
                message: "rpc flake".to_string(),
            }),
        )
        .expect("attempt result insert must succeed");

    // A completed job without attempts.
    let completed_id = seed_completed_job(&context, 3);

    let jobs: Vec<SettlementJobSummary> = context
        .admin_client
        .request("admin_listSettlementJobs", rpc_params![])
        .await
        .expect("list must succeed");
    assert_eq!(jobs.len(), 2);

    let pending = jobs
        .iter()
        .find(|job| job.job_id == pending_id)
        .expect("pending job must be listed");
    assert_eq!(pending.status, SettlementJobStatus::Pending);
    assert_eq!(pending.certificate_id, Some(certificate_id));
    assert!(!pending.has_live_task);
    assert_eq!(pending.attempt_count, 1);
    assert_eq!(
        pending
            .latest_attempt
            .as_ref()
            .expect("latest attempt must be set")
            .attempt_number,
        0
    );
    assert!(pending
        .last_error
        .as_ref()
        .expect("last error must be set")
        .contains("rpc flake"));

    let completed = jobs
        .iter()
        .find(|job| job.job_id == completed_id)
        .expect("completed job must be listed");
    assert_eq!(completed.status, SettlementJobStatus::Completed);
    assert_eq!(completed.certificate_id, None);
    assert_eq!(completed.last_error, None);
}

#[test_log::test(tokio::test)]
async fn get_settlement_job_returns_detail_with_attempts() {
    let context = TestContext::new_with_config(TestContext::get_default_config()).await;
    let job_id = seed_pending_job(&context, 4);
    context
        .state_store
        .insert_settlement_attempt(&job_id, 0, &mk_attempt(4))
        .expect("attempt insert must succeed");

    let detail: SettlementJobDetail = context
        .admin_client
        .request("admin_getSettlementJob", rpc_params![job_id])
        .await
        .expect("get must succeed");
    assert_eq!(detail.job_id, job_id);
    assert_eq!(detail.status, SettlementJobStatus::Pending);
    assert_eq!(detail.attempts.len(), 1);
    assert_eq!(detail.attempts[0].nonce, 4);
    assert!(detail.attempts[0].result.is_none());
    assert!(detail.job_result.is_none());

    // Live-task flag through the respawn path.
    let () = context
        .admin_client
        .request("admin_reloadAndRestartSettlementTask", rpc_params![job_id])
        .await
        .expect("reload must respawn");
    let detail: SettlementJobDetail = context
        .admin_client
        .request("admin_getSettlementJob", rpc_params![job_id])
        .await
        .expect("get must succeed");
    assert!(detail.has_live_task);
}

#[test_log::test(tokio::test)]
async fn get_settlement_job_returns_completed_detail() {
    let context = TestContext::new_with_config(TestContext::get_default_config()).await;
    let job_id = seed_completed_job(&context, 6);

    let detail: SettlementJobDetail = context
        .admin_client
        .request("admin_getSettlementJob", rpc_params![job_id])
        .await
        .expect("get must succeed");
    assert_eq!(detail.status, SettlementJobStatus::Completed);
    let job_result = detail.job_result.expect("job result must be set");
    assert_eq!(job_result.outcome, "success");
    assert_eq!(job_result.nonce, 6);
    assert_eq!(job_result.attempt_number, 6);
    assert_eq!(job_result.block_number, 6);

    // Pin the wire casing of the terminal result.
    let json = serde_json::to_value(&job_result).expect("job result must serialize");
    assert!(json.get("attemptNumber").is_some());
    assert!(json.get("txHash").is_some());
    assert!(json.get("blockNumber").is_some());
}

#[test_log::test(tokio::test)]
async fn get_settlement_job_unknown_id_is_resource_not_found() {
    let context = TestContext::new_with_config(TestContext::get_default_config()).await;
    let error = context
        .admin_client
        .request::<SettlementJobDetail, _>("admin_getSettlementJob", rpc_params![mk_job_id(98)])
        .await
        .expect_err("unknown job must fail");
    assert_error_code(error, crate::error::code::RESOURCE_NOT_FOUND);
}
