use std::time::{Duration, SystemTime};

use agglayer_types::{
    Address, ClientError, ClientErrorType, ContractCallOutcome, ContractCallResult, Digest, Nonce,
    SettlementAttempt, SettlementAttemptResult, SettlementTxHash, B256,
};

use super::*;

fn attempt(seed: u64) -> SettlementAttempt {
    SettlementAttempt {
        sender_wallet: Address::from([seed as u8; 20]),
        nonce: Nonce(seed),
        hash: SettlementTxHash::new(Digest::from([seed as u8; 32])),
        submission_time: SystemTime::UNIX_EPOCH + Duration::from_secs(seed),
        max_fee_per_gas: 30_000_000_000,
        max_priority_fee_per_gas: 1_000_000_000,
    }
}

fn client_error_result(message: &str) -> SettlementAttemptResult {
    SettlementAttemptResult::ClientError(ClientError {
        kind: ClientErrorType::Unknown,
        message: message.to_string(),
    })
}

fn contract_call_result(outcome: ContractCallOutcome) -> SettlementAttemptResult {
    SettlementAttemptResult::ContractCall(ContractCallResult {
        outcome,
        metadata: vec![].into(),
        block_hash: B256::from([9u8; 32]),
        block_number: 9,
        tx_hash: SettlementTxHash::new(Digest::from([9u8; 32])),
    })
}

#[test]
fn last_error_is_none_without_results() {
    assert_eq!(render_last_error(&[]), None);
}

#[test]
fn last_error_renders_the_latest_client_error() {
    let results = vec![
        (0, client_error_result("older error")),
        (1, client_error_result("newer error")),
    ];
    let rendered = render_last_error(&results).expect("latest failure must render");
    assert!(rendered.contains("newer error"), "got: {rendered}");
}

#[test]
fn last_error_is_none_when_latest_result_is_a_success() {
    let results = vec![
        (0, client_error_result("older error")),
        (1, contract_call_result(ContractCallOutcome::Success)),
    ];
    assert_eq!(render_last_error(&results), None);
}

#[test]
fn last_error_renders_the_latest_revert() {
    let results = vec![(0, contract_call_result(ContractCallOutcome::Revert))];
    let rendered = render_last_error(&results).expect("revert must render");
    assert!(rendered.contains("Reverted"), "got: {rendered}");
}

#[test]
fn job_summary_serializes_camel_case() {
    let summary = SettlementJobSummary {
        job_id: agglayer_types::SettlementJobId::from(1u128),
        certificate_id: None,
        status: SettlementJobStatus::Pending,
        has_live_task: true,
        attempt_count: 1,
        latest_attempt: Some(SettlementAttemptSummary::from((0u64, &attempt(0)))),
        last_error: None,
    };
    let json = serde_json::to_value(&summary).expect("summary must serialize");
    assert!(json.get("hasLiveTask").is_some());
    assert!(json.get("attemptCount").is_some());
    assert_eq!(json["status"], "pending");
    assert!(json["latestAttempt"].get("senderWallet").is_some());
}
