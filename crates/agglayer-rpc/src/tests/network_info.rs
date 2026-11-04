use std::sync::Arc;

use agglayer_config::Config;
use agglayer_storage::{
    backup::BackupClient,
    columns::latest_settled_certificate_per_network::{
        LatestSettledCertificatePerNetworkColumn, SettledCertificate,
    },
    error::Error as StorageError,
    stores::{state::StateStore, LatestCertificateHeaders, NetworkInfoReader, StateWriter},
    tests::{
        mocks::{MockDebugStore, MockPendingStore, MockStateStore},
        TempDBDir,
    },
};
use agglayer_types::{
    Certificate, CertificateHeader, CertificateIndex, CertificateStatus, CertificateStatusError,
    Digest, Metadata, NetworkId, NetworkInfo, NetworkStatus, NetworkType, SettledClaim,
};
use alloy::providers::{mock::Asserter, ProviderBuilder};
use mockall::predicate::eq;

const DEFAULT_NETWORK_INFO: NetworkInfo = NetworkInfo::from_network_id(NetworkId::new(1));
const NETWORK_1: NetworkId = NetworkId::new(1);

/// The header of a test certificate of network 1 at `height`.
fn header(height: u64, status: CertificateStatus) -> CertificateHeader {
    let certificate = Certificate::new_for_test(NETWORK_1, height.into());
    CertificateHeader {
        network_id: NETWORK_1,
        height: certificate.height,
        epoch_number: None,
        certificate_index: None,
        certificate_id: certificate.hash(),
        prev_local_exit_root: certificate.prev_local_exit_root,
        new_local_exit_root: certificate.new_local_exit_root,
        metadata: Metadata::DEFAULT,
        status,
        settlement_tx_hash: None,
    }
}

macro_rules! service {
    ($state_store:expr) => {
        crate::AgglayerService::new(
            tokio::sync::mpsc::channel(1).0,
            Arc::new(MockPendingStore::new()),
            Arc::new($state_store),
            Arc::new(MockDebugStore::new()),
            Arc::new(Config::default()),
            Arc::new(ProviderBuilder::new().connect_mocked_client(Asserter::new())),
        )
    };
}

/// Storage joins the cached pending pointer with its header; the response
/// carries the certificate-owned fields that header holds.
#[tokio::test]
async fn transient_network_info() {
    let pending = header(0, CertificateStatus::Pending);
    let pending_certificate_id = pending.certificate_id;
    let mut state_store = MockStateStore::new();
    state_store
        .expect_get_network_info()
        .with(eq(NETWORK_1))
        .return_once(move |_| {
            Ok(Some(NetworkInfo {
                latest_pending_certificate_id: Some(pending.certificate_id),
                latest_pending_height: Some(pending.height),
                latest_pending_status: Some(pending.status),
                ..DEFAULT_NETWORK_INFO
            }))
        });
    state_store
        .expect_is_network_disabled()
        .with(eq(NETWORK_1))
        .return_once(|_network_id| Ok(false));

    let info = service!(state_store)
        .get_network_info(1.into())
        .await
        .unwrap();
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
    assert_eq!(info.latest_pending_status, Some(CertificateStatus::Pending));
    assert_eq!(
        info.latest_pending_certificate_id,
        Some(pending_certificate_id)
    );
    assert_eq!(info.latest_pending_height, Some(0.into()));
    assert_eq!(info.network_status, NetworkStatus::Active);
}

/// Storage decides existence from the cache in the snapshot it serves:
/// a network without any cached row is unknown, and one with only a type
/// row is known.
#[tokio::test]
async fn network_existence_comes_from_the_cache() {
    let mut state_store = MockStateStore::new();
    state_store
        .expect_get_network_info()
        .with(eq(NETWORK_1))
        .once()
        .return_once(|_| Ok(None));
    assert!(matches!(
        service!(state_store).get_network_info(NETWORK_1).await,
        Err(crate::GetNetworkInfoError::UnknownNetworkType { network_id })
            if network_id == NETWORK_1
    ));

    let mut state_store = MockStateStore::new();
    state_store
        .expect_get_network_info()
        .with(eq(NETWORK_1))
        .once()
        .return_once(|_| {
            Ok(Some(NetworkInfo {
                network_type: NetworkType::Generic,
                ..DEFAULT_NETWORK_INFO
            }))
        });
    state_store
        .expect_is_network_disabled()
        .with(eq(NETWORK_1))
        .once()
        .return_once(|_| Ok(false));
    let info = service!(state_store)
        .get_network_info(NETWORK_1)
        .await
        .unwrap();
    assert_eq!(info.network_type, NetworkType::Generic);
    assert_eq!(info.network_status, NetworkStatus::Unknown);
    assert_eq!(info.latest_pending_certificate_id, None);
    assert_eq!(info.settled_certificate_id, None);
}

/// After the backfill migration, nothing reads the legacy indexes: a settled
/// cursor without a cached row is neither a known network nor a settled
/// certificate.
#[tokio::test]
async fn legacy_only_settled_network_is_unknown_after_migration() {
    let tmp = TempDBDir::new();
    let db = Arc::new(StateStore::init_db(tmp.path.as_path(), None).unwrap());
    let state_store = StateStore::new(db.clone(), BackupClient::noop());
    let certificate = Certificate::new_for_test(NETWORK_1, 0.into());
    let certificate_id = certificate.hash();
    state_store
        .insert_certificate_header(&certificate, CertificateStatus::Settled)
        .unwrap();
    state_store
        .assign_certificate_to_epoch(&certificate_id, &0.into(), &CertificateIndex::ZERO)
        .unwrap();
    // Seed only the legacy cursor after the empty database has migrated, so
    // neither backfill nor a settlement writer populates the cache.
    db.put::<LatestSettledCertificatePerNetworkColumn>(
        &NETWORK_1,
        &SettledCertificate(certificate_id, 0.into(), 0.into(), CertificateIndex::ZERO),
    )
    .unwrap();
    assert_eq!(state_store.get_network_info(NETWORK_1).unwrap(), None);
    assert_eq!(
        state_store
            .get_latest_settled_certificate_id(NETWORK_1)
            .unwrap(),
        None
    );

    let service = service!(state_store);
    assert!(matches!(
        service.get_network_info(NETWORK_1).await,
        Err(crate::GetNetworkInfoError::UnknownNetworkType { network_id })
            if network_id == NETWORK_1
    ));
    assert_eq!(
        service
            .get_latest_settled_certificate_header(NETWORK_1)
            .await
            .unwrap(),
        None
    );
    assert_eq!(
        service
            .get_latest_known_certificate_header(NETWORK_1)
            .await
            .unwrap(),
        None
    );
}

#[tokio::test]
async fn pending_certificate_defined() {
    let settled_certificate = Certificate::new_for_test(NETWORK_1, 0.into());
    let settled_certificate_id = settled_certificate.hash();
    let pending = header(1, CertificateStatus::Pending);
    let pending_certificate_id = pending.certificate_id;
    let mut state_store = MockStateStore::new();
    state_store
        .expect_get_network_info()
        .with(eq(NETWORK_1))
        .once()
        .return_once(move |_| {
            Ok(Some(NetworkInfo {
                settled_certificate_id: Some(settled_certificate_id),
                settled_height: Some(settled_certificate.height),
                settled_ler: Some(settled_certificate.new_local_exit_root),
                latest_epoch_with_settlement: Some(0),
                latest_pending_certificate_id: Some(pending.certificate_id),
                latest_pending_height: Some(pending.height),
                latest_pending_status: Some(pending.status),
                ..DEFAULT_NETWORK_INFO
            }))
        });
    state_store
        .expect_is_network_disabled()
        .with(eq(NETWORK_1))
        .return_once(|_network_id| Ok(false));

    let info = service!(state_store)
        .get_network_info(1.into())
        .await
        .unwrap();

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
    assert_eq!(info.latest_pending_status, Some(CertificateStatus::Pending));
    assert_eq!(
        info.latest_pending_certificate_id,
        Some(pending_certificate_id)
    );
    assert_eq!(info.latest_pending_height, Some(1.into()));
    assert_eq!(info.network_status, NetworkStatus::Active);
}

/// The pending status and error are the referenced header's, as storage
/// read them; an errored latest pending certificate puts the network in
/// error.
#[tokio::test]
async fn pending_certificate_defined_with_network_info() {
    let pending_error = CertificateStatusError::InternalError("cached header error".to_string());
    let pending = header(11, CertificateStatus::error(pending_error.clone()));
    let pending_certificate_id = pending.certificate_id;
    let network_info = NetworkInfo {
        settled_certificate_id: Some(Digest::from([1u8; 32]).into()),
        settled_claim: None,
        settled_height: Some(10.into()),
        settled_ler: Some([1u8; 32].into()),
        settled_pp_root: None,
        settled_let_leaf_count: Some(2),
        latest_pending_status: Some(pending.status),
        latest_pending_height: Some(pending.height),
        latest_pending_certificate_id: Some(pending.certificate_id),
        latest_pending_error: Some(pending_error.clone()),
        network_status: NetworkStatus::Unknown,
        network_type: NetworkType::Generic,
        network_id: NETWORK_1,
        latest_epoch_with_settlement: Some(0),
    };
    let mut state_store = MockStateStore::new();
    state_store
        .expect_get_network_info()
        .with(eq(NETWORK_1))
        .return_once(move |_| Ok(Some(network_info)));
    state_store
        .expect_is_network_disabled()
        .with(eq(NETWORK_1))
        .return_once(|_network_id| Ok(false));

    let info = service!(state_store)
        .get_network_info(1.into())
        .await
        .unwrap();

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
        Some(CertificateStatus::error(pending_error))
    );
    assert_eq!(
        info.latest_pending_certificate_id,
        Some(pending_certificate_id)
    );
    assert_eq!(info.latest_pending_height, Some(11.into()));
    assert_eq!(info.latest_epoch_with_settlement, Some(0));
    assert_eq!(info.network_status, NetworkStatus::Error);
}

/// A cached pending pointer whose header has settled since must not leak
/// stale pending fields.
#[tokio::test]
async fn settled_cached_pending_header_is_omitted() {
    let settled = header(11, CertificateStatus::Settled);
    let mut state_store = MockStateStore::new();
    state_store
        .expect_get_network_info()
        .with(eq(NETWORK_1))
        .return_once(move |_| {
            Ok(Some(NetworkInfo {
                latest_pending_certificate_id: Some(settled.certificate_id),
                latest_pending_height: Some(settled.height),
                latest_pending_status: Some(settled.status),
                ..DEFAULT_NETWORK_INFO
            }))
        });
    state_store
        .expect_is_network_disabled()
        .with(eq(NETWORK_1))
        .return_once(|_| Ok(false));

    let info = service!(state_store)
        .get_network_info(NETWORK_1)
        .await
        .unwrap();
    assert_eq!(info.latest_pending_certificate_id, None);
    assert_eq!(info.latest_pending_height, None);
    assert_eq!(info.latest_pending_status, None);
    assert_eq!(info.latest_pending_error, None);
    assert_eq!(info.network_status, NetworkStatus::Unknown);
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
            let certificate = Certificate::new_for_test(NETWORK_1, 10_000.into());
            let certificate_id = certificate.hash();
            let network_info = NetworkInfo {
                settled_certificate_id: Some(certificate_id),
                settled_height: Some(certificate.height),
                settled_claim: claim.clone(),
                settled_let_leaf_count: leaf_count,
                ..NetworkInfo::from_network_id(NETWORK_1)
            };
            let mut state_store = MockStateStore::new();
            state_store
                .expect_get_network_info()
                .with(eq(NETWORK_1))
                .return_once(move |_| Ok(Some(network_info)));
            state_store
                .expect_is_network_disabled()
                .with(eq(NETWORK_1))
                .return_once(|_| Ok(false));

            let info = service!(state_store)
                .get_network_info(NETWORK_1)
                .await
                .unwrap();
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

    let crate::error::GetNetworkInfoError::InternalError { network_id, source } =
        service!(state_store)
            .get_network_info(NETWORK_1)
            .await
            .unwrap_err()
    else {
        panic!("Expected an internal error");
    };
    assert_eq!(network_id, NETWORK_1);
    assert!(source.to_string().contains("unreadable network info"));
}

/// The latest-certificate requests serve the headers storage joined with the
/// cached pointers: the settled one as is, the pending one unless it has
/// settled since, and the highest of the three as the latest known.
#[tokio::test]
async fn latest_certificate_headers_come_from_the_cached_pointers() {
    let pending = header(2, CertificateStatus::Pending);
    let proven = header(1, CertificateStatus::Proven);
    let settled = header(0, CertificateStatus::Settled);
    let headers = LatestCertificateHeaders {
        pending: Some(pending.clone()),
        proven: Some(proven),
        settled: Some(settled.clone()),
    };
    let mut state_store = MockStateStore::new();
    state_store
        .expect_get_latest_certificate_headers()
        .with(eq(NETWORK_1))
        .times(3)
        .returning(move |_| Ok(headers.clone()));
    let service = service!(state_store);

    assert_eq!(
        service
            .get_latest_known_certificate_header(NETWORK_1)
            .await
            .unwrap(),
        Some(pending.clone())
    );
    assert_eq!(
        service
            .get_latest_pending_certificate_header(NETWORK_1)
            .await
            .unwrap(),
        Some(pending)
    );
    assert_eq!(
        service
            .get_latest_settled_certificate_header(NETWORK_1)
            .await
            .unwrap(),
        Some(settled)
    );

    // A pending pointer still naming the certificate that settled last is the
    // latest known certificate, but no longer a pending one.
    let settled = header(3, CertificateStatus::Settled);
    let headers = LatestCertificateHeaders {
        pending: Some(settled.clone()),
        proven: None,
        settled: Some(settled.clone()),
    };
    let mut state_store = MockStateStore::new();
    state_store
        .expect_get_latest_certificate_headers()
        .with(eq(NETWORK_1))
        .times(2)
        .returning(move |_| Ok(headers.clone()));
    let service = service!(state_store);
    assert_eq!(
        service
            .get_latest_known_certificate_header(NETWORK_1)
            .await
            .unwrap(),
        Some(settled)
    );
    assert_eq!(
        service
            .get_latest_pending_certificate_header(NETWORK_1)
            .await
            .unwrap(),
        None
    );
}

/// A cached pointer without its header is a storage error on every request,
/// never an absent certificate.
#[tokio::test]
async fn dangling_pointer_fails_every_request() {
    let certificate_id = Certificate::new_for_test(NETWORK_1, 0.into()).hash();
    let dangling = move || StorageError::DanglingCertificatePointer {
        network_id: NETWORK_1,
        kind: "pending",
        certificate_id,
    };
    let mut state_store = MockStateStore::new();
    state_store
        .expect_get_latest_certificate_headers()
        .with(eq(NETWORK_1))
        .times(3)
        .returning(move |_| Err(dangling()));
    state_store
        .expect_get_network_info()
        .with(eq(NETWORK_1))
        .once()
        .returning(move |_| Err(dangling()));
    let service = service!(state_store);

    assert!(matches!(
        service.get_latest_known_certificate_header(NETWORK_1).await,
        Err(crate::CertificateRetrievalError::Storage(
            StorageError::DanglingCertificatePointer { .. }
        ))
    ));
    assert!(matches!(
        service
            .get_latest_pending_certificate_header(NETWORK_1)
            .await,
        Err(crate::CertificateRetrievalError::Storage(
            StorageError::DanglingCertificatePointer { .. }
        ))
    ));
    assert!(matches!(
        service
            .get_latest_settled_certificate_header(NETWORK_1)
            .await,
        Err(crate::CertificateRetrievalError::Storage(
            StorageError::DanglingCertificatePointer { .. }
        ))
    ));
    let crate::error::GetNetworkInfoError::InternalError { source, .. } =
        service.get_network_info(NETWORK_1).await.unwrap_err()
    else {
        panic!("Expected an internal error");
    };
    assert!(source.to_string().contains("has no header"), "{source}");
}
