use super::*;

fn client_error(kind: ClientErrorType) -> SettlementAttemptResult {
    SettlementAttemptResult::ClientError(ClientError {
        kind,
        message: String::new(),
    })
}

#[test]
fn is_resolved_elsewhere_matches_used_and_settled_kinds() {
    assert!(client_error(ClientErrorType::NonceAlreadyUsed).is_resolved_elsewhere());
    assert!(client_error(ClientErrorType::SettlementSucceededElsewhere).is_resolved_elsewhere());
    assert!(!client_error(ClientErrorType::Unknown).is_resolved_elsewhere());
}
