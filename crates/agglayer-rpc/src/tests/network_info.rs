use std::sync::Arc;

use agglayer_config::Config;
use agglayer_storage::{
    tests::mocks::{MockDebugStore, MockEpochsStore, MockPendingStore, MockStateStore},
    types::{BasicPendingCertificateInfo, BasicSettledCertificateInfo},
};
use agglayer_types::{
    aggchain_data::CertificateAggchainDataCtx, Certificate, CertificateHeader, CertificateIndex,
    Digest, EpochNumber, Height, L1WitnessCtx, Metadata, NetworkId, NetworkInfo,
    PessimisticRootInput,
};
use alloy::providers::{
    mock::{Asserter, MockTransport},
    ProviderBuilder,
};
use mockall::predicate::eq;
use pessimistic_proof::core::commitment::PessimisticRootCommitmentVersion;
use pessimistic_proof_test_suite::forest::Forest;

const DEFAULT_NETWORK_INFO: NetworkInfo = NetworkInfo::from_network_id(NetworkId::new(1));
const NETWORK_1: NetworkId = NetworkId::new(1);

#[test]
fn transient_network_info() {
    let certificate_sender = tokio::sync::mpsc::channel(1).0;

    let mut pending_store = MockPendingStore::new();
    let mut state_store = MockStateStore::new();
    state_store
        .expect_get_network_info()
        .with(eq(NETWORK_1))
        .return_once(|_network_id| Ok(DEFAULT_NETWORK_INFO));

    state_store
        .expect_get_latest_settled_certificate_per_network()
        .with(eq(NETWORK_1))
        .once()
        .return_once(|_| Ok(None));

    let pending_certificate = Certificate::new_for_test(NETWORK_1, 0.into());
    let pending_certificate_id = pending_certificate.hash();
    let pending_certificate_header = CertificateHeader {
        network_id: NETWORK_1,
        height: 0.into(),
        epoch_number: None,
        certificate_index: None,
        certificate_id: pending_certificate_id,
        prev_local_exit_root: pending_certificate.prev_local_exit_root,
        new_local_exit_root: pending_certificate.new_local_exit_root,
        metadata: Metadata::DEFAULT,
        status: agglayer_types::CertificateStatus::Pending,
        settlement_tx_hash: None,
    };

    state_store
        .expect_get_latest_settled_certificate_per_network()
        .with(eq(NETWORK_1))
        .once()
        .returning(|_| Ok(None));

    state_store
        .expect_get_certificate_header()
        .with(eq(pending_certificate_id))
        .once()
        .return_once(move |_| Ok(Some(pending_certificate_header.clone())));

    pending_store
        .expect_get_certificate()
        .with(eq(NETWORK_1), eq(Height::new(0)))
        .once()
        .return_once(move |_, _| Ok(Some(pending_certificate.clone())));

    state_store
        .expect_get_latest_pending_certificate_info()
        .with(eq(NETWORK_1))
        .returning(move |_| {
            Ok(Some(BasicPendingCertificateInfo {
                certificate_id: pending_certificate_id,
                height: 0.into(),
            }))
        });

    pending_store
        .expect_get_latest_proven_certificate_per_network()
        .with(eq(NETWORK_1))
        .returning(move |_| Ok(None));

    let debug_store = MockDebugStore::new();
    let epochs_store = MockEpochsStore::new();
    let config = Arc::new(Config::default());

    // Create a mock provider for the default case
    let asserter = Asserter::new();
    let _transport = MockTransport::new(asserter.clone());
    let l1_rpc_provider = Arc::new(ProviderBuilder::new().on_mocked_client(asserter));

    let service = crate::AgglayerService::new(
        certificate_sender,
        Arc::new(pending_store),
        Arc::new(state_store),
        Arc::new(debug_store),
        Arc::new(epochs_store),
        config,
        l1_rpc_provider,
    );

    let info = service.get_network_info(1.into()).unwrap();
    assert_eq!(info.settled_certificate_id, None);
    assert_eq!(info.settled_claim, None);
    assert_eq!(info.settled_height, None);
    assert_eq!(info.settled_ler, None);
    assert_eq!(info.settled_pp_root, None);
    assert_eq!(info.settled_let_leaf_count, None);

    assert_eq!(info.latest_pending_error, None);
    assert_eq!(
        info.latest_pending_status,
        Some(agglayer_types::CertificateStatus::Pending)
    );
    assert_eq!(info.latest_pending_height, Some(0.into()));
}

#[test]
fn pending_certificate_defined() {
    let certificate_sender = tokio::sync::mpsc::channel(1).0;

    let mut pending_store = MockPendingStore::new();
    let mut state_store = MockStateStore::new();
    state_store
        .expect_get_network_info()
        .with(eq(NETWORK_1))
        .return_once(|_network_id| Ok(DEFAULT_NETWORK_INFO));
    let settled_certificate = Certificate::new_for_test(NETWORK_1, 0.into());
    let settled_certificate_id = settled_certificate.hash();
    let settled_certificate_header = CertificateHeader {
        network_id: NETWORK_1,
        height: 0.into(),
        epoch_number: Some(0.into()),
        certificate_index: Some(CertificateIndex::new(0)),
        certificate_id: settled_certificate_id,
        prev_local_exit_root: settled_certificate.prev_local_exit_root,
        new_local_exit_root: settled_certificate.new_local_exit_root,
        metadata: Metadata::DEFAULT,
        status: agglayer_types::CertificateStatus::Settled,
        settlement_tx_hash: Some(Digest::ZERO.into()),
    };

    let mut network_state = Forest::default();

    let l1_info_root = settled_certificate
        .l1_info_root()
        .unwrap()
        .unwrap_or_default();
    let signer = network_state.get_signer();
    network_state
        .state_b
        .apply_certificate(
            &settled_certificate,
            L1WitnessCtx {
                l1_info_root,
                prev_pessimistic_root: PessimisticRootInput::Computed(
                    PessimisticRootCommitmentVersion::V2,
                ),
                aggchain_data_ctx: CertificateAggchainDataCtx::LegacyEcdsa { signer },
            },
        )
        .unwrap();

    let pending_certificate = Certificate::new_for_test(NETWORK_1, 1.into());
    let pending_certificate_id = pending_certificate.hash();
    let pending_certificate_header = CertificateHeader {
        network_id: NETWORK_1,
        height: 1.into(),
        epoch_number: None,
        certificate_index: None,
        certificate_id: pending_certificate_id,
        prev_local_exit_root: pending_certificate.prev_local_exit_root,
        new_local_exit_root: pending_certificate.new_local_exit_root,
        metadata: Metadata::DEFAULT,
        status: agglayer_types::CertificateStatus::Pending,
        settlement_tx_hash: None,
    };

    state_store
        .expect_read_local_network_state()
        .returning(move |_| Ok(Some(network_state.state_b.clone())));

    state_store
        .expect_get_latest_pending_certificate_info()
        .with(eq(NETWORK_1))
        .returning(move |_| {
            Ok(Some(BasicPendingCertificateInfo {
                certificate_id: pending_certificate_id,
                height: 1.into(),
            }))
        });

    pending_store
        .expect_get_latest_proven_certificate_per_network()
        .with(eq(NETWORK_1))
        .returning(move |_| Ok(None));

    pending_store.expect_get_proof().returning(|_| Ok(None));

    pending_store
        .expect_get_certificate()
        .with(eq(NETWORK_1), eq(Height::new(0)))
        .returning(move |_, _| Ok(None));

    let get_pending_header = pending_certificate_header.clone();
    let get_settled_header = settled_certificate_header.clone();
    state_store
        .expect_get_certificate_header()
        .with(eq(pending_certificate_id))
        .returning(move |_| Ok(Some(get_pending_header.clone())));

    state_store
        .expect_get_certificate_header()
        .with(eq(settled_certificate_id))
        .returning(move |_| Ok(Some(get_settled_header.clone())));

    state_store
        .expect_get_certificate_header_by_cursor()
        .with(eq(NETWORK_1), eq(Height::new(0)))
        .returning(move |_, _| Ok(Some(settled_certificate_header.clone())));

    state_store
        .expect_get_latest_settled_certificate_per_network()
        .with(eq(NETWORK_1))
        .returning(move |_| {
            Ok(Some((
                NETWORK_1,
                BasicSettledCertificateInfo {
                    certificate_id: settled_certificate_id,
                    height: 0.into(),
                    epoch_number: 0.into(),
                    certificate_index: CertificateIndex::new(0),
                },
            )))
        });

    let debug_store = MockDebugStore::new();

    let mut epochs_store = MockEpochsStore::new();
    let get_settled_certificate = settled_certificate.clone();
    epochs_store.expect_get_proof().returning(|_, _| Ok(None));
    epochs_store
        .expect_get_certificate()
        .with(eq(EpochNumber::new(0)), eq(CertificateIndex::new(0)))
        .returning(move |_, _| Ok(Some(get_settled_certificate.clone())));

    let config = Arc::new(Config::default());

    // Create a mock provider for the default case
    let asserter = Asserter::new();
    let _transport = MockTransport::new(asserter.clone());
    let l1_rpc_provider = Arc::new(ProviderBuilder::new().on_mocked_client(asserter));

    let service = crate::AgglayerService::new(
        certificate_sender,
        Arc::new(pending_store),
        Arc::new(state_store),
        Arc::new(debug_store),
        Arc::new(epochs_store),
        config,
        l1_rpc_provider,
    );

    let info = service.get_network_info(1.into()).unwrap();

    assert_eq!(info.settled_certificate_id, Some(settled_certificate_id));
    assert_eq!(info.settled_claim, None);
    assert_eq!(info.settled_height, Some(0.into()));
    assert_eq!(
        info.settled_ler,
        Some(settled_certificate.new_local_exit_root)
    );
    assert_eq!(info.settled_pp_root, None);
    assert_eq!(info.settled_let_leaf_count, Some(0));

    assert_eq!(info.latest_pending_error, None);
    assert_eq!(
        info.latest_pending_status,
        Some(agglayer_types::CertificateStatus::Pending)
    );
    assert_eq!(info.latest_pending_height, Some(1.into()));
}

#[test]
fn pending_certificate_defined_with_network_info() {
    let certificate_sender = tokio::sync::mpsc::channel(1).0;

    let pending_store = MockPendingStore::new();
    let mut state_store = MockStateStore::new();
    let debug_store = MockDebugStore::new();
    let epochs_store = MockEpochsStore::new();
    let config = Arc::new(Config::default());

    let network_info = NetworkInfo {
        settled_certificate_id: Some(Digest::from([1u8; 32]).into()),
        settled_claim: None,
        settled_height: Some(10.into()),
        settled_ler: Some([1u8; 32].into()),
        settled_pp_root: None,
        settled_let_leaf_count: Some(2),
        latest_pending_status: Some(agglayer_types::CertificateStatus::Pending),
        latest_pending_height: Some(11.into()),
        latest_pending_error: None,
        network_status: agglayer_types::NetworkStatus::Active,
        network_type: agglayer_types::NetworkType::Generic,
        network_id: NETWORK_1,
        latest_epoch_with_settlement: Some(0),
    };
    let get_network_info = network_info.clone();
    state_store
        .expect_get_network_info()
        .with(eq(NETWORK_1))
        .return_once(move |_| Ok(get_network_info.clone()));
    // Create a mock provider for the default case
    let asserter = Asserter::new();
    let _transport = MockTransport::new(asserter.clone());
    let l1_rpc_provider = Arc::new(ProviderBuilder::new().on_mocked_client(asserter));

    let service = crate::AgglayerService::new(
        certificate_sender,
        Arc::new(pending_store),
        Arc::new(state_store),
        Arc::new(debug_store),
        Arc::new(epochs_store),
        config,
        l1_rpc_provider,
    );

    let info = service.get_network_info(1.into()).unwrap();

    assert_eq!(
        info.settled_certificate_id,
        Some(Digest::from([1u8; 32]).into())
    );
    assert_eq!(info.settled_claim, None);
    assert_eq!(info.settled_height, Some(10.into()));
    assert_eq!(info.settled_ler, Some([1u8; 32].into()));
    assert_eq!(info.settled_pp_root, None);
    assert_eq!(info.settled_let_leaf_count, Some(2));

    assert_eq!(info.latest_pending_error, None);
    assert_eq!(
        info.latest_pending_status,
        Some(agglayer_types::CertificateStatus::Pending)
    );
    assert_eq!(info.latest_pending_height, Some(11.into()));
}
