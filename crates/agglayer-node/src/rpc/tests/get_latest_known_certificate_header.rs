use std::{sync::Arc, time::Duration};

use agglayer_config::Config;
use agglayer_storage::{
    storage::{pending_db_cf_definitions, state_db_cf_definitions, DB},
    stores::{pending::PendingStore, state::StateStore, PendingCertificateWriter, StateWriter},
    tests::TempDBDir,
};
use agglayer_types::{Certificate, CertificateHeader, CertificateStatus};
use insta::assert_snapshot;
use jsonrpsee::{core::client::ClientT, rpc_params};
use rstest::*;
use serde_json::json;

use super::TestContext;
use crate::rpc::AgglayerServer as _;

#[rstest]
#[test_log::test(tokio::test)]
async fn returns_the_pending_certificate_header() {
    let path = TempDBDir::new();

    let config = Config::new(&path.path);
    let pending_db = Arc::new(
        DB::open_cf(&config.storage.pending_db_path, pending_db_cf_definitions())
            .expect("unable to open pending db"),
    );
    let state_db = Arc::new(
        DB::open_cf(&config.storage.state_db_path, state_db_cf_definitions())
            .expect("unable to open state db"),
    );

    let state_db = StateStore::new(state_db);
    let pending_db = PendingStore::new(pending_db);
    let network_id = 1.into();

    let settled_certificate = Certificate::new_for_test(network_id, 0);
    let proven_certificate = Certificate::new_for_test(network_id, 1);
    let pending_certificate = Certificate::new_for_test(network_id, 2);

    state_db
        .insert_certificate_header(&settled_certificate, CertificateStatus::Settled)
        .expect("unable to insert settled certificate header");
    state_db
        .insert_certificate_header(&proven_certificate, CertificateStatus::Proven)
        .expect("unable to insert proven certificate header");
    state_db
        .insert_certificate_header(&pending_certificate, CertificateStatus::Pending)
        .expect("unable to insert pending certificate header");

    state_db
        .set_latest_settled_certificate_for_network(
            &network_id,
            &0,
            &settled_certificate.hash(),
            &0,
            &0,
        )
        .expect("unable to set latest settled certificate");

    pending_db
        .set_latest_proven_certificate_per_network(&network_id, &1, &proven_certificate.hash())
        .expect("unable to set latest proven certificate");

    pending_db
        .insert_pending_certificate(network_id, 2, &pending_certificate)
        .expect("unable to insert pending certificate");

    drop(pending_db);
    drop(state_db);

    let context = TestContext::new_with_config(config).await;

    let payload: CertificateHeader = context
        .client
        .request(
            "interop_getLatestKnownCertificateHeader",
            rpc_params![network_id],
        )
        .await
        .unwrap();

    assert_eq!(payload.certificate_id, pending_certificate.hash());
    assert_eq!(payload.status, CertificateStatus::Pending);

    drop(context);

    // Have some delayu to ensure that the server has been stopped
    tokio::time::sleep(Duration::from_millis(100)).await;

    let config = Config::new(&path.path);
    // Restarting the server in raw mode
    let raw_rpc = TestContext::new_raw_rpc_with_config(config).await;
    let rpc = raw_rpc.rpc.into_rpc();
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "interop_getLatestKnownCertificateHeader",
        "params": vec![network_id],
        "id": 0
    });
    let (response, _) = rpc
        .raw_json_request(&serde_json::to_string(&payload).unwrap(), 1)
        .await
        .unwrap();
    let json = serde_json::from_str::<serde_json::Value>(&response).unwrap();
    let json = serde_json::to_string_pretty(&json).unwrap();

    assert_snapshot!("get_latest_known_certificate_header::pending", json);
}

#[test_log::test(tokio::test)]
async fn returns_the_proven_certificate_header() {
    let path = TempDBDir::new();

    let config = Config::new(&path.path);
    let pending_db = Arc::new(
        DB::open_cf(&config.storage.pending_db_path, pending_db_cf_definitions())
            .expect("unable to open pending db"),
    );
    let state_db = Arc::new(
        DB::open_cf(&config.storage.state_db_path, state_db_cf_definitions())
            .expect("unable to open state db"),
    );

    let state_db = StateStore::new(state_db);
    let pending_db = PendingStore::new(pending_db);
    let network_id = 1.into();

    let settled_certificate = Certificate::new_for_test(network_id, 0);
    let proven_certificate = Certificate::new_for_test(network_id, 1);

    state_db
        .insert_certificate_header(&settled_certificate, CertificateStatus::Settled)
        .expect("unable to insert settled certificate header");
    state_db
        .insert_certificate_header(&proven_certificate, CertificateStatus::Proven)
        .expect("unable to insert proven certificate header");

    state_db
        .set_latest_settled_certificate_for_network(
            &network_id,
            &settled_certificate.height,
            &settled_certificate.hash(),
            &0,
            &0,
        )
        .expect("unable to set latest settled certificate");
    pending_db
        .set_latest_proven_certificate_per_network(&network_id, &1, &proven_certificate.hash())
        .expect("unable to set latest proven certificate");

    drop(pending_db);
    drop(state_db);

    let context = TestContext::new_with_config(config).await;

    let payload: CertificateHeader = context
        .client
        .request(
            "interop_getLatestKnownCertificateHeader",
            rpc_params![network_id],
        )
        .await
        .unwrap();

    assert_eq!(payload.certificate_id, proven_certificate.hash());
    assert_eq!(payload.status, CertificateStatus::Proven);

    drop(context);

    // Have some delayu to ensure that the server has been stopped
    tokio::time::sleep(Duration::from_millis(100)).await;

    let config = Config::new(&path.path);
    // Restarting the server in raw mode
    let raw_rpc = TestContext::new_raw_rpc_with_config(config).await;
    let rpc = raw_rpc.rpc.into_rpc();
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "interop_getLatestKnownCertificateHeader",
        "params": vec![network_id],
        "id": 0
    });
    let (response, _) = rpc
        .raw_json_request(&serde_json::to_string(&payload).unwrap(), 1)
        .await
        .unwrap();
    let json = serde_json::from_str::<serde_json::Value>(&response).unwrap();
    let json = serde_json::to_string_pretty(&json).unwrap();

    assert_snapshot!("get_latest_known_certificate_header::proven", json);
}

#[test_log::test(tokio::test)]
async fn returns_the_settled_certificate_header() {
    let path = TempDBDir::new();

    let config = Config::new(&path.path);
    let pending_db = Arc::new(
        DB::open_cf(&config.storage.pending_db_path, pending_db_cf_definitions())
            .expect("unable to open pending db"),
    );
    let state_db = Arc::new(
        DB::open_cf(&config.storage.state_db_path, state_db_cf_definitions())
            .expect("unable to open state db"),
    );

    let state_db = StateStore::new(state_db);
    let pending_db = PendingStore::new(pending_db);
    let network_id = 1.into();

    let settled_certificate = Certificate::new_for_test(network_id, 0);

    state_db
        .insert_certificate_header(&settled_certificate, CertificateStatus::Settled)
        .expect("unable to insert settled certificate header");

    state_db
        .set_latest_settled_certificate_for_network(
            &network_id,
            &settled_certificate.height,
            &settled_certificate.hash(),
            &0,
            &0,
        )
        .expect("unable to set latest settled certificate");

    drop(pending_db);
    drop(state_db);

    let context = TestContext::new_with_config(config).await;

    let payload: CertificateHeader = context
        .client
        .request(
            "interop_getLatestKnownCertificateHeader",
            rpc_params![network_id],
        )
        .await
        .unwrap();

    assert_eq!(payload.certificate_id, settled_certificate.hash());
    assert_eq!(payload.status, CertificateStatus::Settled);

    drop(context);

    // Have some delayu to ensure that the server has been stopped
    tokio::time::sleep(Duration::from_millis(100)).await;

    let config = Config::new(&path.path);
    // Restarting the server in raw mode
    let raw_rpc = TestContext::new_raw_rpc_with_config(config).await;
    let rpc = raw_rpc.rpc.into_rpc();
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "interop_getLatestKnownCertificateHeader",
        "params": vec![network_id],
        "id": 0
    });
    let (response, _) = rpc
        .raw_json_request(&serde_json::to_string(&payload).unwrap(), 1)
        .await
        .unwrap();
    let json = serde_json::from_str::<serde_json::Value>(&response).unwrap();
    let json = serde_json::to_string_pretty(&json).unwrap();

    assert_snapshot!(
        "get_latest_known_certificate_header::settled_certificate",
        json
    );
}

#[test_log::test(tokio::test)]
async fn returns_no_certificate_header() {
    let path = TempDBDir::new();

    let config = Config::new(&path.path);

    let network_id = 1;

    let context = TestContext::new_with_config(config).await;

    let payload: Option<CertificateHeader> = context
        .client
        .request(
            "interop_getLatestKnownCertificateHeader",
            rpc_params![network_id],
        )
        .await
        .unwrap();

    assert!(payload.is_none());

    drop(context);

    // Have some delayu to ensure that the server has been stopped
    tokio::time::sleep(Duration::from_millis(100)).await;

    let config = Config::new(&path.path);
    // Restarting the server in raw mode
    let raw_rpc = TestContext::new_raw_rpc_with_config(config).await;
    let rpc = raw_rpc.rpc.into_rpc();
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "interop_getLatestKnownCertificateHeader",
        "params": vec![network_id],
        "id": 0
    });
    let (response, _) = rpc
        .raw_json_request(&serde_json::to_string(&payload).unwrap(), 1)
        .await
        .unwrap();
    let json = serde_json::from_str::<serde_json::Value>(&response).unwrap();
    let json = serde_json::to_string_pretty(&json).unwrap();

    assert_snapshot!(
        "get_latest_known_certificate_header::no_certificate_header",
        json
    );
}

#[test_log::test(tokio::test)]
async fn returns_the_highest_height() {
    let path = TempDBDir::new();

    let config = Config::new(&path.path);
    let pending_db = Arc::new(
        DB::open_cf(&config.storage.pending_db_path, pending_db_cf_definitions())
            .expect("unable to open pending db"),
    );
    let state_db = Arc::new(
        DB::open_cf(&config.storage.state_db_path, state_db_cf_definitions())
            .expect("unable to open state db"),
    );

    let state_db = StateStore::new(state_db);
    let pending_db = PendingStore::new(pending_db);
    let network_id = 1.into();

    let settled_certificate = Certificate::new_for_test(network_id, 10);
    let pending_certificate = Certificate::new_for_test(network_id, 3);

    state_db
        .insert_certificate_header(&settled_certificate, CertificateStatus::Settled)
        .expect("unable to insert settled certificate header");
    state_db
        .insert_certificate_header(&pending_certificate, CertificateStatus::Pending)
        .expect("unable to insert pending certificate header");

    state_db
        .set_latest_settled_certificate_for_network(
            &network_id,
            &10,
            &settled_certificate.hash(),
            &0,
            &0,
        )
        .expect("unable to set latest settled certificate");

    pending_db
        .insert_pending_certificate(network_id, 3, &pending_certificate)
        .expect("unable to insert pending certificate");

    drop(pending_db);
    drop(state_db);

    let context = TestContext::new_with_config(config).await;

    let payload: CertificateHeader = context
        .client
        .request(
            "interop_getLatestKnownCertificateHeader",
            rpc_params![network_id],
        )
        .await
        .unwrap();

    assert_eq!(payload.certificate_id, settled_certificate.hash());
    assert_eq!(payload.status, CertificateStatus::Settled);
    assert_eq!(payload.height, 10);

    drop(context);

    // Have some delayu to ensure that the server has been stopped
    tokio::time::sleep(Duration::from_millis(100)).await;

    let config = Config::new(&path.path);
    // Restarting the server in raw mode
    let raw_rpc = TestContext::new_raw_rpc_with_config(config).await;
    let rpc = raw_rpc.rpc.into_rpc();
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "interop_getLatestKnownCertificateHeader",
        "params": vec![network_id],
        "id": 0
    });
    let (response, _) = rpc
        .raw_json_request(&serde_json::to_string(&payload).unwrap(), 1)
        .await
        .unwrap();
    let json = serde_json::from_str::<serde_json::Value>(&response).unwrap();
    let json = serde_json::to_string_pretty(&json).unwrap();

    assert_snapshot!("get_latest_known_certificate_header::highest_height", json);
}
