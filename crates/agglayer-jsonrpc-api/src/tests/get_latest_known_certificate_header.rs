use std::time::Duration;

use agglayer_config::Config;
use agglayer_storage::{
    stores::{PendingCertificateWriter, StateWriter},
    tests::TempDBDir,
};
use agglayer_types::{
    Certificate, CertificateHeader, CertificateIndex, CertificateStatus, EpochNumber, Height,
};
use insta::assert_snapshot;
use jsonrpsee::{core::client::ClientT, rpc_params};
use serde_json::json;

use crate::{testutils::TestContext, AgglayerServer};

#[test_log::test(tokio::test)]
async fn returns_the_pending_certificate_header() {
    let tmp = TempDBDir::new();
    let config = Config::new(&tmp.path);
    let context = TestContext::new_with_config(config).await;

    let network_id = 1.into();

    let settled_certificate = Certificate::new_for_test(network_id, Height::ZERO);
    let proven_certificate = Certificate::new_for_test(network_id, Height::new(1));
    let pending_certificate = Certificate::new_for_test(network_id, Height::new(2));

    context
        .state_store
        .insert_certificate_header(&settled_certificate, CertificateStatus::Settled)
        .expect("unable to insert settled certificate header");
    context
        .state_store
        .insert_certificate_header(&proven_certificate, CertificateStatus::Proven)
        .expect("unable to insert proven certificate header");
    context
        .state_store
        .insert_certificate_header(&pending_certificate, CertificateStatus::Pending)
        .expect("unable to insert pending certificate header");

    context
        .state_store
        .set_latest_settled_certificate_for_network(
            &network_id,
            &Height::ZERO,
            &settled_certificate.hash(),
            &EpochNumber::ZERO,
            &CertificateIndex::ZERO,
        )
        .expect("unable to set latest settled certificate");

    context
        .pending_store
        .set_latest_proven_certificate_per_network(
            &network_id,
            &Height::new(1),
            &proven_certificate.hash(),
        )
        .expect("unable to set latest proven certificate");

    context
        .pending_store
        .insert_pending_certificate(network_id, Height::new(2), &pending_certificate)
        .expect("unable to insert pending certificate");

    let payload: CertificateHeader = context
        .api_client
        .request(
            "interop_getLatestKnownCertificateHeader",
            rpc_params![network_id],
        )
        .await
        .unwrap();

    assert_eq!(payload.certificate_id, pending_certificate.hash());
    assert_eq!(payload.status, CertificateStatus::Pending);

    drop(context);

    // Have some delay to ensure that the server has been stopped
    tokio::time::sleep(Duration::from_millis(100)).await;
    let config = Config::new(&tmp.path);

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
    let tmp = TempDBDir::new();
    let config = Config::new(&tmp.path);
    let context = TestContext::new_with_config(config).await;

    let network_id = 1.into();

    let settled_certificate = Certificate::new_for_test(network_id, Height::ZERO);
    let proven_certificate = Certificate::new_for_test(network_id, Height::new(1));

    context
        .state_store
        .insert_certificate_header(&settled_certificate, CertificateStatus::Settled)
        .expect("unable to insert settled certificate header");
    context
        .state_store
        .insert_certificate_header(&proven_certificate, CertificateStatus::Proven)
        .expect("unable to insert proven certificate header");

    context
        .state_store
        .set_latest_settled_certificate_for_network(
            &network_id,
            &settled_certificate.height,
            &settled_certificate.hash(),
            &EpochNumber::ZERO,
            &CertificateIndex::ZERO,
        )
        .expect("unable to set latest settled certificate");
    context
        .pending_store
        .set_latest_proven_certificate_per_network(
            &network_id,
            &Height::new(1),
            &proven_certificate.hash(),
        )
        .expect("unable to set latest proven certificate");

    drop(context);

    tokio::time::sleep(Duration::from_millis(200)).await;

    let config = Config::new(&tmp.path);

    let context = TestContext::new_with_config(config).await;
    let payload: CertificateHeader = context
        .api_client
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
    tokio::time::sleep(Duration::from_millis(200)).await;

    let config = Config::new(&tmp.path);

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
    let tmp = TempDBDir::new();
    let config = Config::new(&tmp.path);
    let context = TestContext::new_with_config(config).await;

    let network_id = 1.into();

    let settled_certificate = Certificate::new_for_test(network_id, Height::ZERO);

    context
        .state_store
        .insert_certificate_header(&settled_certificate, CertificateStatus::Settled)
        .expect("unable to insert settled certificate header");

    context
        .state_store
        .set_latest_settled_certificate_for_network(
            &network_id,
            &settled_certificate.height,
            &settled_certificate.hash(),
            &EpochNumber::ZERO,
            &CertificateIndex::ZERO,
        )
        .expect("unable to set latest settled certificate");

    drop(context);

    tokio::time::sleep(Duration::from_millis(200)).await;
    let config = Config::new(&tmp.path);
    let context = TestContext::new_with_config(config).await;

    let payload: CertificateHeader = context
        .api_client
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

    let config = Config::new(&tmp.path);
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
    let tmp = TempDBDir::new();
    let config = Config::new(&tmp.path);
    let context = TestContext::new_with_config(config).await;

    let network_id = 1;

    let payload: Option<CertificateHeader> = context
        .api_client
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

    let config = Config::new(&tmp.path);
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
    let tmp = TempDBDir::new();
    let config = Config::new(&tmp.path);
    let context = TestContext::new_with_config(config).await;

    let network_id = 1.into();

    let settled_certificate = Certificate::new_for_test(network_id, Height::new(10));
    let pending_certificate = Certificate::new_for_test(network_id, Height::new(3));

    context
        .state_store
        .insert_certificate_header(&settled_certificate, CertificateStatus::Settled)
        .expect("unable to insert settled certificate header");
    context
        .state_store
        .insert_certificate_header(&pending_certificate, CertificateStatus::Pending)
        .expect("unable to insert pending certificate header");

    context
        .state_store
        .set_latest_settled_certificate_for_network(
            &network_id,
            &Height::new(10),
            &settled_certificate.hash(),
            &EpochNumber::ZERO,
            &CertificateIndex::ZERO,
        )
        .expect("unable to set latest settled certificate");

    context
        .pending_store
        .insert_pending_certificate(network_id, Height::new(3), &pending_certificate)
        .expect("unable to insert pending certificate");

    drop(context);

    tokio::time::sleep(Duration::from_millis(200)).await;
    let config = Config::new(&tmp.path);
    let context = TestContext::new_with_config(config).await;

    let payload: CertificateHeader = context
        .api_client
        .request(
            "interop_getLatestKnownCertificateHeader",
            rpc_params![network_id],
        )
        .await
        .unwrap();

    assert_eq!(payload.certificate_id, settled_certificate.hash());
    assert_eq!(payload.status, CertificateStatus::Settled);
    assert_eq!(payload.height, Height::new(10));

    drop(context);

    // Have some delayu to ensure that the server has been stopped
    tokio::time::sleep(Duration::from_millis(100)).await;

    let config = Config::new(&tmp.path);
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

#[test_log::test(tokio::test)]
async fn returns_the_settled_one_at_same_height() {
    let tmp = TempDBDir::new();
    let config = Config::new(&tmp.path);
    let context = TestContext::new_with_config(config).await;

    let network_id = 1.into();

    let settled_certificate = Certificate::new_for_test(network_id, Height::new(10));

    let mut pending_certificate = Certificate::new_for_test(network_id, Height::new(5));
    pending_certificate.height = Height::new(10);
    let pending_certificate = pending_certificate.with_new_local_exit_root([2; 32].into());

    let mut proven_certificate = Certificate::new_for_test(network_id, Height::new(3));
    proven_certificate.height = Height::new(10);
    let proven_certificate = proven_certificate.with_new_local_exit_root([1; 32].into());

    context
        .state_store
        .insert_certificate_header(&settled_certificate, CertificateStatus::Settled)
        .expect("unable to insert settled certificate header");
    context
        .state_store
        .insert_certificate_header(&pending_certificate, CertificateStatus::Pending)
        .expect("unable to insert pending certificate header");
    context
        .state_store
        .insert_certificate_header(&proven_certificate, CertificateStatus::Proven)
        .expect("unable to insert pending certificate header");

    context
        .state_store
        .set_latest_settled_certificate_for_network(
            &network_id,
            &Height::new(10),
            &settled_certificate.hash(),
            &EpochNumber::ZERO,
            &CertificateIndex::ZERO,
        )
        .expect("unable to set latest settled certificate");

    context
        .pending_store
        .insert_pending_certificate(network_id, Height::new(10), &pending_certificate)
        .expect("unable to insert pending certificate");

    context
        .pending_store
        .set_latest_proven_certificate_per_network(
            &network_id,
            &Height::new(10),
            &proven_certificate.hash(),
        )
        .expect("unable to set latest proven certificate");

    drop(context);
    tokio::time::sleep(Duration::from_millis(200)).await;
    let config = Config::new(&tmp.path);

    let context = TestContext::new_with_config(config).await;

    let payload: CertificateHeader = context
        .api_client
        .request(
            "interop_getLatestKnownCertificateHeader",
            rpc_params![network_id],
        )
        .await
        .unwrap();

    assert_eq!(payload.certificate_id, settled_certificate.hash());
    assert_eq!(payload.status, CertificateStatus::Settled);
    assert_eq!(payload.height, Height::new(10));

    drop(context);

    // Have some delayu to ensure that the server has been stopped
    tokio::time::sleep(Duration::from_millis(100)).await;

    let config = Config::new(&tmp.path);
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
