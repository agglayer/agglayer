use std::sync::Arc;

use agglayer_config::Config;
use agglayer_storage::{
    columns::latest_settled_certificate_per_network::SettledCertificate,
    error::Error as StorageError,
    tests::mocks::{MockDebugStore, MockPendingStore, MockStateStore},
};
use agglayer_types::{
    Certificate, CertificateHeader, CertificateIndex, CertificateStatusError, Digest, Metadata,
    NetworkId, NetworkInfo, NetworkType, SettledClaim,
};
use alloy::providers::{
    mock::{Asserter, MockTransport},
    ProviderBuilder,
};
use mockall::predicate::eq;

const DEFAULT_NETWORK_INFO: NetworkInfo = NetworkInfo::from_network_id(NetworkId::new(1));
const NETWORK_1: NetworkId = NetworkId::new(1);

#[tokio::test]
async fn transient_network_info() {
    let certificate_sender = tokio::sync::mpsc::channel(1).0;

    let mut pending_store = MockPendingStore::new();
    let mut state_store = MockStateStore::new();
    state_store
        .expect_get_network_info()
        .with(eq(NETWORK_1))
        .return_once(|_network_id| Ok(DEFAULT_NETWORK_INFO));

    state_store
        .expect_is_network_disabled()
        .with(eq(NETWORK_1))
        .return_once(|_network_id| Ok(false));

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
        .expect_get_certificate_header()
        .with(eq(pending_certificate_id))
        .once()
        .return_once(move |_| Ok(Some(pending_certificate_header.clone())));

    pending_store
        .expect_get_latest_pending_certificate_for_network()
        .with(eq(NETWORK_1))
        .returning(move |_| Ok(Some((pending_certificate_id, 0.into()))));

    let debug_store = MockDebugStore::new();
    let config = Arc::new(Config::default());

    // Create a mock provider for the default case
    let asserter = Asserter::new();
    let _transport = MockTransport::new(asserter.clone());
    let l1_rpc_provider = Arc::new(ProviderBuilder::new().connect_mocked_client(asserter));

    let service = crate::AgglayerService::new(
        certificate_sender,
        Arc::new(pending_store),
        Arc::new(state_store),
        Arc::new(debug_store),
        config,
        l1_rpc_provider,
    );

    let info = service.get_network_info(1.into()).await.unwrap();
    // The record holds no type yet, and the request reports that rather than
    // decoding a certificate to recover it or failing outright.
    assert_eq!(info.network_type, NetworkType::Unspecified);
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
    assert_eq!(
        info.latest_pending_certificate_id,
        Some(pending_certificate_id)
    );
    assert_eq!(info.latest_pending_height, Some(0.into()));
}

#[tokio::test]
async fn network_existence_uses_cached_data_and_certificate_pointers() {
    for known_from in [
        "nowhere",
        "cached type",
        "cached claim",
        "cached pending",
        "cached proven",
        "cached settled",
        "legacy pending",
        "legacy proven",
        "legacy settled",
        "unreadable cached proven",
        "unreadable legacy settled",
    ] {
        let certificate_id = Digest([1; 32]).into();
        let mut network_info = DEFAULT_NETWORK_INFO;
        match known_from {
            "cached type" => network_info.network_type = NetworkType::Generic,
            "cached claim" => {
                network_info.settled_claim = Some(SettledClaim {
                    global_index: Digest([2; 32]),
                    bridge_exit_hash: Digest([3; 32]),
                });
            }
            "cached pending" => {
                network_info.latest_pending_certificate_id = Some(certificate_id);
                network_info.latest_pending_height = Some(0.into());
            }
            _ => {}
        }

        let mut state_store = MockStateStore::new();
        state_store
            .expect_get_network_info()
            .with(eq(NETWORK_1))
            .once()
            .return_once(move |_| Ok(network_info));
        state_store
            .expect_get_latest_proven_certificate_id()
            .with(eq(NETWORK_1))
            .times(0..=1)
            .returning(move |_| match known_from {
                "unreadable cached proven" => Err(StorageError::Unexpected(known_from.into())),
                "cached proven" => Ok(Some(certificate_id)),
                _ => Ok(None),
            });
        state_store
            .expect_get_latest_settled_certificate_id()
            .with(eq(NETWORK_1))
            .times(0..=1)
            .returning(move |_| Ok((known_from == "cached settled").then_some(certificate_id)));
        state_store
            .expect_get_latest_settled_certificate_per_network()
            .with(eq(NETWORK_1))
            .times(0..=1)
            .returning(move |_| match known_from {
                "unreadable legacy settled" => Err(StorageError::Unexpected(known_from.into())),
                "legacy settled" => Ok(Some((
                    NETWORK_1,
                    SettledCertificate(
                        certificate_id,
                        0.into(),
                        0.into(),
                        CertificateIndex::new(0),
                    ),
                ))),
                _ => Ok(None),
            });
        // A pointer still identifies a known network when its header is absent.
        state_store
            .expect_get_certificate_header()
            .with(eq(certificate_id))
            .times(0..=1)
            .returning(|_| Ok(None));
        state_store
            .expect_is_network_disabled()
            .with(eq(NETWORK_1))
            .times(0..=1)
            .returning(|_| Ok(false));

        let mut pending_store = MockPendingStore::new();
        pending_store
            .expect_get_latest_pending_certificate_for_network()
            .with(eq(NETWORK_1))
            .times(0..=1)
            .returning(move |_| {
                Ok((known_from == "legacy pending").then_some((certificate_id, 0.into())))
            });
        pending_store
            .expect_get_latest_proven_certificate_per_network()
            .with(eq(NETWORK_1))
            .times(0..=1)
            .returning(move |_| {
                Ok(
                    (known_from == "legacy proven").then_some((
                        NETWORK_1,
                        0.into(),
                        certificate_id,
                    )),
                )
            });

        let service = crate::AgglayerService::new(
            tokio::sync::mpsc::channel(1).0,
            Arc::new(pending_store),
            Arc::new(state_store),
            Arc::new(MockDebugStore::new()),
            Arc::new(Config::default()),
            Arc::new(ProviderBuilder::new().connect_mocked_client(Asserter::new())),
        );

        let result = service.get_network_info(NETWORK_1).await;
        match known_from {
            "nowhere" => assert!(matches!(
                result,
                Err(crate::GetNetworkInfoError::UnknownNetworkType { network_id })
                    if network_id == NETWORK_1
            )),
            "unreadable cached proven" | "unreadable legacy settled" => {
                let Err(crate::GetNetworkInfoError::InternalError { network_id, source }) = result
                else {
                    panic!("Expected an internal error for {known_from}");
                };
                assert_eq!(network_id, NETWORK_1);
                assert!(source.to_string().contains(known_from));
            }
            _ => {
                let info = result.unwrap_or_else(|error| panic!("{known_from}: {error}"));
                assert_eq!(info.network_id, NETWORK_1);
                assert_eq!(
                    info.network_type,
                    if known_from == "cached type" {
                        NetworkType::Generic
                    } else {
                        NetworkType::Unspecified
                    }
                );
                assert_eq!(info.latest_pending_certificate_id, None);
                assert_eq!(info.settled_certificate_id, None);
            }
        }
    }
}

#[tokio::test]
async fn pending_certificate_defined() {
    let certificate_sender = tokio::sync::mpsc::channel(1).0;

    let mut pending_store = MockPendingStore::new();
    let mut state_store = MockStateStore::new();
    state_store
        .expect_is_network_disabled()
        .with(eq(NETWORK_1))
        .return_once(|_network_id| Ok(false));

    let settled_certificate = Certificate::new_for_test(NETWORK_1, 0.into());
    let settled_certificate_id = settled_certificate.hash();
    let network_info = NetworkInfo {
        settled_certificate_id: Some(settled_certificate_id),
        settled_height: Some(settled_certificate.height),
        settled_ler: Some(settled_certificate.new_local_exit_root),
        latest_epoch_with_settlement: Some(0),
        ..DEFAULT_NETWORK_INFO
    };
    state_store
        .expect_get_network_info()
        .with(eq(NETWORK_1))
        .once()
        .return_once(move |_| Ok(network_info));

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

    pending_store
        .expect_get_latest_pending_certificate_for_network()
        .with(eq(NETWORK_1))
        .returning(move |_| Ok(Some((pending_certificate_id, 1.into()))));

    let get_pending_header = pending_certificate_header.clone();
    state_store
        .expect_get_certificate_header()
        .with(eq(pending_certificate_id))
        .returning(move |_| Ok(Some(get_pending_header.clone())));

    let debug_store = MockDebugStore::new();

    let config = Arc::new(Config::default());

    // Create a mock provider for the default case
    let asserter = Asserter::new();
    let _transport = MockTransport::new(asserter.clone());
    let l1_rpc_provider = Arc::new(ProviderBuilder::new().connect_mocked_client(asserter));

    let service = crate::AgglayerService::new(
        certificate_sender,
        Arc::new(pending_store),
        Arc::new(state_store),
        Arc::new(debug_store),
        config,
        l1_rpc_provider,
    );

    let info = service.get_network_info(1.into()).await.unwrap();

    assert_eq!(info.settled_certificate_id, Some(settled_certificate_id));
    assert_eq!(info.settled_claim, None);
    assert_eq!(info.settled_height, Some(0.into()));
    assert_eq!(
        info.settled_ler,
        Some(settled_certificate.new_local_exit_root)
    );
    assert_eq!(info.settled_pp_root, None);
    assert_eq!(info.settled_let_leaf_count, None);

    assert_eq!(info.latest_pending_error, None);
    assert_eq!(
        info.latest_pending_status,
        Some(agglayer_types::CertificateStatus::Pending)
    );
    assert_eq!(
        info.latest_pending_certificate_id,
        Some(pending_certificate_id)
    );
    assert_eq!(info.latest_pending_height, Some(1.into()));
}

#[tokio::test]
async fn pending_certificate_defined_with_network_info() {
    let certificate_sender = tokio::sync::mpsc::channel(1).0;

    let pending_store = MockPendingStore::new();
    let mut state_store = MockStateStore::new();
    let debug_store = MockDebugStore::new();
    let config = Arc::new(Config::default());
    let pending_certificate = Certificate::new_for_test(NETWORK_1, 11.into());
    let pending_certificate_id = pending_certificate.hash();
    let pending_error = CertificateStatusError::InternalError("cached header error".to_string());
    let pending_certificate_header = CertificateHeader {
        network_id: NETWORK_1,
        height: 11.into(),
        epoch_number: None,
        certificate_index: None,
        certificate_id: pending_certificate_id,
        prev_local_exit_root: pending_certificate.prev_local_exit_root,
        new_local_exit_root: pending_certificate.new_local_exit_root,
        metadata: Metadata::DEFAULT,
        status: agglayer_types::CertificateStatus::error(pending_error.clone()),
        settlement_tx_hash: None,
    };

    let network_info = NetworkInfo {
        settled_certificate_id: Some(Digest::from([1u8; 32]).into()),
        settled_claim: None,
        settled_height: Some(10.into()),
        settled_ler: Some([1u8; 32].into()),
        settled_pp_root: None,
        settled_let_leaf_count: Some(2),
        latest_pending_status: Some(agglayer_types::CertificateStatus::Pending),
        latest_pending_height: Some(11.into()),
        latest_pending_certificate_id: Some(pending_certificate_id),
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

    state_store
        .expect_get_certificate_header()
        .with(eq(pending_certificate_id))
        .return_once(move |_| Ok(Some(pending_certificate_header)));

    state_store
        .expect_is_network_disabled()
        .with(eq(NETWORK_1))
        .return_once(|_network_id| Ok(false));

    // Create a mock provider for the default case
    let asserter = Asserter::new();
    let _transport = MockTransport::new(asserter.clone());
    let l1_rpc_provider = Arc::new(ProviderBuilder::new().connect_mocked_client(asserter));

    let service = crate::AgglayerService::new(
        certificate_sender,
        Arc::new(pending_store),
        Arc::new(state_store),
        Arc::new(debug_store),
        config,
        l1_rpc_provider,
    );

    let info = service.get_network_info(1.into()).await.unwrap();

    assert_eq!(
        info.settled_certificate_id,
        Some(Digest::from([1u8; 32]).into())
    );
    assert_eq!(info.settled_claim, None);
    assert_eq!(info.settled_height, Some(10.into()));
    assert_eq!(info.settled_ler, Some([1u8; 32].into()));
    assert_eq!(info.settled_pp_root, None);
    assert_eq!(info.settled_let_leaf_count, Some(2));

    assert_eq!(info.latest_pending_error, Some(pending_error.clone()));
    assert_eq!(
        info.latest_pending_status,
        Some(agglayer_types::CertificateStatus::error(pending_error))
    );
    assert_eq!(
        info.latest_pending_certificate_id,
        Some(pending_certificate_id)
    );
    assert_eq!(info.latest_pending_height, Some(11.into()));
    assert_eq!(info.latest_epoch_with_settlement, Some(0));
}

#[tokio::test]
async fn settled_cached_pending_header_is_omitted() {
    let certificate_sender = tokio::sync::mpsc::channel(1).0;

    let pending_store = MockPendingStore::new();
    let mut state_store = MockStateStore::new();
    let debug_store = MockDebugStore::new();
    let config = Arc::new(Config::default());

    let certificate = Certificate::new_for_test(NETWORK_1, 11.into());
    let certificate_id = certificate.hash();
    let header = CertificateHeader {
        network_id: NETWORK_1,
        height: certificate.height,
        epoch_number: Some(0.into()),
        certificate_index: Some(CertificateIndex::new(0)),
        certificate_id,
        prev_local_exit_root: certificate.prev_local_exit_root,
        new_local_exit_root: certificate.new_local_exit_root,
        metadata: Metadata::DEFAULT,
        status: agglayer_types::CertificateStatus::Settled,
        settlement_tx_hash: Some(Digest::ZERO.into()),
    };
    let network_info = NetworkInfo {
        latest_pending_certificate_id: Some(certificate_id),
        latest_pending_height: Some(certificate.height),
        latest_pending_status: Some(agglayer_types::CertificateStatus::Pending),
        ..NetworkInfo::from_network_id(NETWORK_1)
    };

    state_store
        .expect_get_network_info()
        .with(eq(NETWORK_1))
        .return_once(move |_| Ok(network_info));
    state_store
        .expect_get_certificate_header()
        .with(eq(certificate_id))
        .return_once(move |_| Ok(Some(header)));
    state_store
        .expect_is_network_disabled()
        .with(eq(NETWORK_1))
        .return_once(|_| Ok(false));

    let asserter = Asserter::new();
    let _transport = MockTransport::new(asserter.clone());
    let l1_rpc_provider = Arc::new(ProviderBuilder::new().connect_mocked_client(asserter));
    let service = crate::AgglayerService::new(
        certificate_sender,
        Arc::new(pending_store),
        Arc::new(state_store),
        Arc::new(debug_store),
        config,
        l1_rpc_provider,
    );

    let info = service.get_network_info(NETWORK_1).await.unwrap();
    assert_eq!(info.latest_pending_certificate_id, None);
    assert_eq!(info.latest_pending_height, None);
    assert_eq!(info.latest_pending_status, None);
    assert_eq!(info.latest_pending_error, None);
}

/// Every settled field is whatever the network info record holds — claim,
/// network type and leaf count alike. No certificate or proof expectation is
/// set on purpose, so any attempt to recover one of them by reading a
/// certificate or a proof fails the test instead of passing quietly. Storage
/// owns keeping that record self-consistent; see
/// `settled_header_is_read_from_the_same_snapshot`.
#[tokio::test]
async fn settled_claim_is_served_from_the_network_info_record() {
    for claim in [
        None,
        Some(SettledClaim {
            global_index: Digest([7u8; 32]),
            bridge_exit_hash: Digest([9u8; 32]),
        }),
    ] {
        for leaf_count in [None, Some(7)] {
            let certificate_sender = tokio::sync::mpsc::channel(1).0;

            let mut pending_store = MockPendingStore::new();
            let mut state_store = MockStateStore::new();
            let debug_store = MockDebugStore::new();
            let config = Arc::new(Config::default());

            let certificate = Certificate::new_for_test(NETWORK_1, 10_000.into());
            let certificate_id = certificate.hash();
            let network_info = NetworkInfo {
                settled_certificate_id: Some(certificate_id),
                settled_height: Some(certificate.height),
                settled_claim: claim.clone(),
                settled_let_leaf_count: leaf_count,
                ..NetworkInfo::from_network_id(NETWORK_1)
            };

            state_store
                .expect_get_network_info()
                .with(eq(NETWORK_1))
                .return_once(move |_| Ok(network_info));
            state_store
                .expect_is_network_disabled()
                .with(eq(NETWORK_1))
                .return_once(|_| Ok(false));
            pending_store
                .expect_get_latest_pending_certificate_for_network()
                .with(eq(NETWORK_1))
                .return_once(|_| Ok(None));

            let asserter = Asserter::new();
            let _transport = MockTransport::new(asserter.clone());
            let l1_rpc_provider = Arc::new(ProviderBuilder::new().connect_mocked_client(asserter));
            let service = crate::AgglayerService::new(
                certificate_sender,
                Arc::new(pending_store),
                Arc::new(state_store),
                Arc::new(debug_store),
                config,
                l1_rpc_provider,
            );

            let info = service.get_network_info(NETWORK_1).await.unwrap();
            assert_eq!(info.settled_claim, claim);
            assert_eq!(info.network_type, NetworkType::Unspecified);
            assert_eq!(info.settled_let_leaf_count, leaf_count);
            assert_eq!(info.settled_pp_root, None);
        }
    }
}

#[tokio::test]
async fn network_info_storage_error_is_propagated() {
    let mut state_store = MockStateStore::new();
    state_store
        .expect_get_network_info()
        .with(eq(NETWORK_1))
        .once()
        .return_once(|_| Err(StorageError::Unexpected("unreadable network info".into())));

    let service = crate::AgglayerService::new(
        tokio::sync::mpsc::channel(1).0,
        Arc::new(MockPendingStore::new()),
        Arc::new(state_store),
        Arc::new(MockDebugStore::new()),
        Arc::new(Config::default()),
        Arc::new(ProviderBuilder::new().connect_mocked_client(Asserter::new())),
    );

    let crate::error::GetNetworkInfoError::InternalError { network_id, source } =
        service.get_network_info(NETWORK_1).await.unwrap_err()
    else {
        panic!("Expected an internal error");
    };
    assert_eq!(network_id, NETWORK_1);
    assert!(source.to_string().contains("unreadable network info"));
}
