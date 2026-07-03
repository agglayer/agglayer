use std::sync::Arc;

use agglayer_config::Config;
use agglayer_storage::tests::mocks::{
    MockDebugStore, MockEpochsStore, MockPendingStore, MockStateStore,
};
use agglayer_types::{
    Address, Certificate, CertificateId, ContractCallOutcome, ContractCallResult, Digest,
    NetworkId, Nonce, SettlementAttemptNumber, SettlementJobId, SettlementJobResult,
    SettlementTxHash, B256,
};
use alloy::providers::{mock::Asserter, ProviderBuilder};
use mockall::predicate::eq;

use crate::CertificateSubmissionError;

const NETWORK_1: NetworkId = NetworkId::new(1);

fn settlement_job_id() -> SettlementJobId {
    SettlementJobId::from(42u128)
}

fn job_result(outcome: ContractCallOutcome) -> SettlementJobResult {
    SettlementJobResult {
        wallet: Address::new([0u8; 20]),
        nonce: Nonce(0),
        attempt_number: SettlementAttemptNumber(0),
        contract_call_result: ContractCallResult {
            outcome,
            metadata: Default::default(),
            block_hash: B256::ZERO,
            block_number: 0,
            tx_hash: SettlementTxHash::new(Digest([9u8; 32])),
        },
    }
}

fn service_with_state(
    state_store: MockStateStore,
) -> crate::AgglayerService<
    impl alloy::providers::Provider,
    MockPendingStore,
    MockStateStore,
    MockDebugStore,
    MockEpochsStore,
> {
    let certificate_sender = tokio::sync::mpsc::channel(1).0;
    let asserter = Asserter::new();
    let l1_rpc_provider = Arc::new(ProviderBuilder::new().connect_mocked_client(asserter));

    crate::AgglayerService::new(
        certificate_sender,
        Arc::new(MockPendingStore::new()),
        Arc::new(state_store),
        Arc::new(MockDebugStore::new()),
        Arc::new(MockEpochsStore::new()),
        Arc::new(Config::default()),
        l1_rpc_provider,
    )
}

fn ids() -> (Certificate, CertificateId, CertificateId) {
    let replacement = Certificate::new_for_test(NETWORK_1, 0.into());
    let replacement_id = replacement.hash();
    let pre_existing_id = CertificateId::new(Digest([1u8; 32]));
    (replacement, pre_existing_id, replacement_id)
}

#[test]
fn replacement_refused_while_settlement_job_is_pending() {
    let (replacement, pre_existing_id, replacement_id) = ids();

    let mut state_store = MockStateStore::new();
    state_store
        .expect_get_certificate_settlement_job_id()
        .with(eq(pre_existing_id))
        .once()
        .returning(|_| Ok(Some(settlement_job_id())));
    state_store
        .expect_get_settlement_job_result()
        .with(eq(settlement_job_id()))
        .once()
        .returning(|_| Ok(None));

    let service = service_with_state(state_store);
    let error = service
        .ensure_no_live_settlement_job(&replacement, pre_existing_id, replacement_id)
        .expect_err("a pending settlement job must block replacement");

    match error {
        CertificateSubmissionError::UnableToReplacePendingCertificate {
            reason,
            stored_certificate_id,
            replacement_certificate_id,
            ..
        } => {
            assert!(reason.contains("still in flight"), "reason: {reason}");
            assert_eq!(stored_certificate_id, pre_existing_id);
            assert_eq!(replacement_certificate_id, replacement_id);
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn replacement_refused_after_settlement_job_success() {
    let (replacement, pre_existing_id, replacement_id) = ids();

    let mut state_store = MockStateStore::new();
    state_store
        .expect_get_certificate_settlement_job_id()
        .with(eq(pre_existing_id))
        .once()
        .returning(|_| Ok(Some(settlement_job_id())));
    state_store
        .expect_get_settlement_job_result()
        .with(eq(settlement_job_id()))
        .once()
        .returning(|_| Ok(Some(job_result(ContractCallOutcome::Success))));

    let service = service_with_state(state_store);
    let error = service
        .ensure_no_live_settlement_job(&replacement, pre_existing_id, replacement_id)
        .expect_err("a succeeded settlement job must block replacement");

    match error {
        CertificateSubmissionError::UnableToReplacePendingCertificate { reason, .. } => {
            assert!(reason.contains("already succeeded"), "reason: {reason}");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn replacement_allowed_after_settlement_job_revert() {
    let (replacement, pre_existing_id, replacement_id) = ids();

    let mut state_store = MockStateStore::new();
    state_store
        .expect_get_certificate_settlement_job_id()
        .with(eq(pre_existing_id))
        .once()
        .returning(|_| Ok(Some(settlement_job_id())));
    state_store
        .expect_get_settlement_job_result()
        .with(eq(settlement_job_id()))
        .once()
        .returning(|_| Ok(Some(job_result(ContractCallOutcome::Revert))));

    let service = service_with_state(state_store);
    service
        .ensure_no_live_settlement_job(&replacement, pre_existing_id, replacement_id)
        .expect("a terminally reverted settlement job must allow replacement");
}

#[test]
fn replacement_allowed_without_settlement_job() {
    let (replacement, pre_existing_id, replacement_id) = ids();

    let mut state_store = MockStateStore::new();
    state_store
        .expect_get_certificate_settlement_job_id()
        .with(eq(pre_existing_id))
        .once()
        .returning(|_| Ok(None));

    let service = service_with_state(state_store);
    service
        .ensure_no_live_settlement_job(&replacement, pre_existing_id, replacement_id)
        .expect("a certificate without a settlement job keeps the legacy replacement behavior");
}
