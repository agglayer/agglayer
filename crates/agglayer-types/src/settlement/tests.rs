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
    assert!(!client_error(ClientErrorType::AbandonedByAdmin).is_resolved_elsewhere());
}

#[test]
fn abandoned_by_admin_yields_only_to_on_chain_evidence() {
    let abandoned = client_error(ClientErrorType::AbandonedByAdmin);
    let contract_call = SettlementAttemptResult::ContractCall(ContractCallResult {
        outcome: ContractCallOutcome::Success,
        metadata: Bytes::new(),
        block_hash: B256::ZERO,
        block_number: 0,
        tx_hash: SettlementTxHash::from(crate::Digest::from([0; 32])),
    });

    // Real on-chain evidence supersedes an admin abandon assertion.
    assert!(abandoned.can_be_replaced_by(&contract_call));

    // Resolved-elsewhere notes and the admin assertion itself do not
    // overwrite an existing abandon through the normal upgrade path.
    for kind in [
        ClientErrorType::Unknown,
        ClientErrorType::NonceAlreadyUsed,
        ClientErrorType::SettlementSucceededElsewhere,
        ClientErrorType::AbandonedByAdmin,
    ] {
        assert!(!abandoned.can_be_replaced_by(&client_error(kind)));
    }

    // The normal upgrade path never writes an admin abandon over another
    // client error; only the admin override (which bypasses this check)
    // records it.
    assert!(!client_error(ClientErrorType::Unknown)
        .can_be_replaced_by(&client_error(ClientErrorType::AbandonedByAdmin)));
}
