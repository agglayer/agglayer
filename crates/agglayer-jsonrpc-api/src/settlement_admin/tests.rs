use std::time::{Duration, SystemTime};

use agglayer_types::{
    Address, ClientError, ClientErrorType, ContractCallOutcome, ContractCallResult, Digest, Nonce,
    SettlementAttempt, SettlementAttemptResult, SettlementJobId, SettlementTxHash, B256, U256,
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

fn client_error_result(kind: ClientErrorType, message: &str) -> SettlementAttemptResult {
    SettlementAttemptResult::ClientError(ClientError {
        kind,
        message: message.to_string(),
    })
}

fn unknown_client_error_result(message: &str) -> SettlementAttemptResult {
    client_error_result(ClientErrorType::Unknown, message)
}

fn contract_call_result(outcome: ContractCallOutcome) -> SettlementAttemptResult {
    SettlementAttemptResult::ContractCall(ContractCallResult {
        outcome,
        metadata: vec![].into(),
        block_hash: B256::from([9_u8; 32]),
        block_number: 9,
        tx_hash: SettlementTxHash::new(Digest::from([9_u8; 32])),
    })
}

fn job_detail(attempts: Vec<SettlementAttemptDetail>) -> SettlementJobDetail {
    SettlementJobDetail {
        job_id: SettlementJobId::from(5_u128),
        certificate_id: None,
        status: SettlementJobStatus::Pending,
        has_live_task: false,
        contract_address: Address::ZERO,
        eth_value: U256::ZERO,
        gas_limit: 0,
        calldata: Default::default(),
        attempts,
        job_result: None,
        last_error: Some("newest failure".to_string()),
    }
}

#[test]
fn last_error_is_none_without_results() {
    assert_eq!(render_last_error(&[]), None);
}

#[test]
fn last_error_renders_the_latest_client_error() {
    // Descending on purpose: latest follows the attempt number, not slice
    // order.
    let results = vec![
        (1, unknown_client_error_result("newer error")),
        (0, unknown_client_error_result("older error")),
    ];

    let rendered = render_last_error(&results).expect("latest failure must render");
    assert!(rendered.contains("newer error"), "got: {rendered}");
}

#[test]
fn last_error_is_none_when_latest_result_is_a_success() {
    let results = vec![
        (0, unknown_client_error_result("older error")),
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
fn job_summary_selects_latest_attempt_by_number() {
    // Descending on purpose: selection must follow the attempt number.
    let attempts = vec![
        SettlementAttemptDetail::new(1, &attempt(1), None),
        SettlementAttemptDetail::new(0, &attempt(0), None),
    ];
    let summary = SettlementJobSummary::from(&job_detail(attempts));
    let summary = summary
        .as_readable()
        .expect("readable detail must produce a readable summary");

    assert_eq!(summary.status, SettlementJobStatus::Pending);
    assert_eq!(summary.attempt_count, 2);
    assert_eq!(
        summary
            .latest_attempt
            .as_ref()
            .expect("latest attempt must be set")
            .attempt_number,
        1
    );
    assert!(summary
        .last_error
        .as_ref()
        .expect("last error must be set")
        .contains("newest failure"));
}

#[test]
fn attempt_result_dto_serializes_internally_tagged_shapes() {
    let client_error = SettlementAttemptResultDto::from(&unknown_client_error_result("boom"));
    let json = serde_json::to_value(&client_error).expect("client error must serialize");
    assert_eq!(json["type"], "clientError");
    assert_eq!(json["kind"], "unknown");
    assert_eq!(json["message"], "boom");

    let call = SettlementAttemptResultDto::from(&contract_call_result(ContractCallOutcome::Revert));
    let json = serde_json::to_value(&call).expect("contract call must serialize");
    assert_eq!(json["type"], "contractCall");
    assert_eq!(json["outcome"], "revert");
    assert!(json.get("txHash").is_some());
    assert!(json.get("blockNumber").is_some());
    assert!(json.get("blockHash").is_some());
}

#[test]
fn client_error_kinds_serialize_as_stable_camel_case_tags() {
    for (kind, expected) in [
        (ClientErrorType::Unknown, "unknown"),
        (ClientErrorType::NonceAlreadyUsed, "nonceAlreadyUsed"),
        (
            ClientErrorType::SettlementSucceededElsewhere,
            "settlementSucceededElsewhere",
        ),
        (ClientErrorType::AbandonedByAdmin, "abandonedByAdmin"),
    ] {
        let result = client_error_result(kind, "message");
        let dto = SettlementAttemptResultDto::from(&result);
        let json = serde_json::to_value(dto).expect("client error must serialize");

        assert_eq!(json["kind"], expected);
    }
}

#[test]
fn job_summary_serializes_camel_case() {
    let summary = SettlementJobSummary::Readable(ReadableSettlementJobSummary {
        job_id: SettlementJobId::from(1_u128),
        certificate_id: None,
        status: SettlementJobStatus::Pending,
        has_live_task: true,
        attempt_count: 1,
        latest_attempt: Some(SettlementAttemptSummary::from(
            &SettlementAttemptDetail::new(0, &attempt(0), None),
        )),
        last_error: None,
    });

    let json = serde_json::to_value(summary).expect("summary must serialize");
    assert!(json.get("hasLiveTask").is_some());
    assert!(json.get("attemptCount").is_some());
    assert_eq!(json["status"], "pending");
    assert!(json["latestAttempt"].get("senderWallet").is_some());
}

#[test]
fn unreadable_job_summary_serializes_the_exact_error() {
    let summary = SettlementJobSummary::unreadable(
        SettlementJobId::from(1_u128),
        "Failed to read settlement job: invalid protobuf".to_string(),
    );

    let json = serde_json::to_value(summary).expect("summary must serialize");
    assert_eq!(
        json,
        serde_json::json!({
            "jobId": "00000000000000000000000001",
            "status": "unreadable",
            "error": "Failed to read settlement job: invalid protobuf",
        })
    );
}
