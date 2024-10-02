use agglayer_types::{Certificate, CertificateHeader, CertificateId, CertificateStatus, Hash};
use insta::assert_snapshot;
use jsonrpsee::{
    core::{client::ClientT, ClientError},
    rpc_params,
};
use rstest::*;
use serde_json::json;

use super::context;
use super::raw_rpc;
use super::TestContext;
use crate::rpc::{tests::RawRpcContext, AgglayerServer};

#[rstest]
#[awt]
#[test_log::test(tokio::test)]
async fn fetch_unkown_certificate_header(#[future] context: TestContext) {
    let payload: Result<CertificateHeader, ClientError> = context
        .client
        .request("interop_getCertificateHeader", rpc_params![Hash([0; 32])])
        .await;

    let error = payload.unwrap_err();

    let expected_message = format!("Resource not found: Certificate({:#})", Hash([0; 32]));
    assert!(matches!(error, ClientError::Call(obj) if obj.message() == expected_message));
}

#[rstest]
#[awt]
#[test_log::test(tokio::test)]
async fn fetch_kown_certificate_header(#[future] mut context: TestContext) {
    let certificate = Certificate::new_for_test(1.into(), 0);
    let id = certificate.hash();

    let res: CertificateId = context
        .client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    assert_eq!(id, res);
    assert!(context.certificate_receiver.try_recv().is_ok());

    let payload: CertificateHeader = context
        .client
        .request("interop_getCertificateHeader", rpc_params![id])
        .await
        .unwrap();

    assert_eq!(payload.certificate_id, id);
    assert_eq!(payload.status, CertificateStatus::Pending);
}

#[rstest]
#[awt]
#[test_log::test(tokio::test)]
async fn get_certificate_header_after_sending_the_certificate(#[future] mut context: TestContext) {
    let certificate = Certificate::new_for_test(1.into(), 0);
    let id = certificate.hash();

    let res: CertificateId = context
        .client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    assert_eq!(id, res);
    assert!(context.certificate_receiver.try_recv().is_ok());

    let payload: CertificateHeader = context
        .client
        .request("interop_getCertificateHeader", rpc_params![id])
        .await
        .unwrap();

    assert_eq!(payload.certificate_id, id);
    assert_eq!(payload.status, CertificateStatus::Pending);

    let payload: Result<CertificateHeader, ClientError> = context
        .client
        .request("interop_getCertificateHeader", rpc_params![Hash([0; 32])])
        .await;

    let error = payload.unwrap_err();

    let expected_message = format!("Resource not found: Certificate({:#})", Hash([0; 32]));
    assert!(matches!(error, ClientError::Call(obj) if obj.message() == expected_message));
}

#[rstest]
#[awt]
#[test_log::test(tokio::test)]
async fn certificate_error_message(#[future] raw_rpc: RawRpcContext) {
    let rpc = raw_rpc.rpc.into_rpc();
    let params = vec![Hash([0; 32])];
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "interop_getCertificateHeader",
        "params": params,
        "id": 0
    });
    let (response, _) = rpc
        .raw_json_request(&serde_json::to_string(&payload).unwrap(), 1)
        .await
        .unwrap();
    let json = serde_json::from_str::<serde_json::Value>(&response).unwrap();
    let json = serde_json::to_string_pretty(&json).unwrap();

    assert_snapshot!(
        "get_certificate_header::not_found",
        json,
        &serde_json::to_string_pretty(&json!({
            "payload": payload,
            "raw_response": response
        }))
        .unwrap()
    );
}

#[rstest]
#[awt]
#[test_log::test(tokio::test)]
async fn certificate_header(#[future] raw_rpc: RawRpcContext) {
    let rpc = raw_rpc.rpc.into_rpc();
    let certificate = Certificate::new_for_test(1.into(), 0);
    let id = certificate.hash();

    let params = vec![certificate];
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "interop_sendCertificate",
        "params": params,
        "id": 0
    });
    let (_, _) = rpc
        .raw_json_request(&serde_json::to_string(&payload).unwrap(), 1)
        .await
        .unwrap();

    let params = vec![id];
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "interop_getCertificateHeader",
        "params": params,
        "id": 0
    });
    let (response, _) = rpc
        .raw_json_request(&serde_json::to_string(&payload).unwrap(), 1)
        .await
        .unwrap();
    let json = serde_json::from_str::<serde_json::Value>(&response).unwrap();
    let json = serde_json::to_string_pretty(&json).unwrap();

    assert_snapshot!(
        "get_certificate_header::found",
        json,
        &serde_json::to_string_pretty(&json!({
            "payload": payload,
            "raw_response": response
        }))
        .unwrap()
    );
}
