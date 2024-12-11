use agglayer_types::{Certificate, CertificateHeader, CertificateId, CertificateStatus, Digest};
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
async fn fetch_unknown_certificate_header(#[future] context: TestContext) {
    let payload: Result<CertificateHeader, ClientError> = context
        .client
        .request("interop_getCertificateHeader", rpc_params![Digest([0; 32])])
        .await;

    let error = payload.unwrap_err();

    let expected_message = format!("Resource not found: Certificate({:#})", Digest([0; 32]));
    assert!(matches!(error, ClientError::Call(obj) if obj.message() == expected_message));
}

#[rstest]
#[awt]
#[test_log::test(tokio::test)]
async fn fetch_known_certificate_header(#[future] mut context: TestContext) {
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
        .request("interop_getCertificateHeader", rpc_params![Digest([0; 32])])
        .await;

    let error = payload.unwrap_err();

    let expected_message = format!("Resource not found: Certificate({:#})", Digest([0; 32]));
    assert!(matches!(error, ClientError::Call(obj) if obj.message() == expected_message));
}

#[rstest]
#[awt]
#[test_log::test(tokio::test)]
async fn certificate_error_message(#[future] raw_rpc: RawRpcContext) {
    let rpc = raw_rpc.rpc.into_rpc();
    let params = vec![Digest([0; 32])];
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
#[rstest]
#[test_log::test(tokio::test)]
async fn debug_fetch_unknown_certificate() {
    let mut config = TestContext::get_default_config();
    config.debug_mode = true;

    let context = TestContext::new_with_config(config).await;

    let payload: Result<(Certificate, Option<CertificateHeader>), ClientError> = context
        .client
        .request("interop_debugGetCertificate", rpc_params![Digest([0; 32])])
        .await;

    let error = payload.unwrap_err();

    let expected_message = format!("Resource not found: Certificate({:#})", Digest([0; 32]));
    assert!(matches!(error, ClientError::Call(obj) if obj.message() == expected_message));
}

#[rstest]
#[test_log::test(tokio::test)]
async fn debug_fetch_known_certificate() {
    let mut config = TestContext::get_default_config();
    config.debug_mode = true;

    let mut context = TestContext::new_with_config(config).await;

    let certificate = Certificate::new_for_test(1.into(), 0);
    let id = certificate.hash();

    let res: CertificateId = context
        .client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    assert_eq!(id, res);
    assert!(context.certificate_receiver.try_recv().is_ok());

    let (recv_cert, header): (Certificate, Option<CertificateHeader>) = context
        .client
        .request("interop_debugGetCertificate", rpc_params![id])
        .await
        .unwrap();

    assert!(header.is_some());
    let header = header.unwrap();
    assert_eq!(header.certificate_id, id);
    assert_eq!(recv_cert.hash(), id);
    assert_eq!(header.status, CertificateStatus::Pending);
}

#[rstest]
#[test_log::test(tokio::test)]
async fn debug_get_certificate_after_sending_the_certificate() {
    let mut config = TestContext::get_default_config();
    config.debug_mode = true;

    let mut context = TestContext::new_with_config(config).await;

    let certificate = Certificate::new_for_test(1.into(), 0);
    let id = certificate.hash();

    let res: CertificateId = context
        .client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    assert_eq!(id, res);
    assert!(context.certificate_receiver.try_recv().is_ok());

    let (recv_cert, header): (Certificate, Option<CertificateHeader>) = context
        .client
        .request("interop_debugGetCertificate", rpc_params![id])
        .await
        .unwrap();

    assert!(header.is_some());
    let header = header.unwrap();
    assert_eq!(header.certificate_id, id);
    assert_eq!(recv_cert.hash(), id);
    assert_eq!(header.status, CertificateStatus::Pending);

    let payload: Result<(Certificate, Option<CertificateHeader>), ClientError> = context
        .client
        .request("interop_debugGetCertificate", rpc_params![Digest([0; 32])])
        .await;

    let error = payload.unwrap_err();

    let expected_message = format!("Resource not found: Certificate({:#})", Digest([0; 32]));
    assert!(matches!(error, ClientError::Call(obj) if obj.message() == expected_message));
}

#[rstest]
#[test_log::test(tokio::test)]
async fn debug_get_certificate_after_overwrite() {
    let mut config = TestContext::get_default_config();
    config.debug_mode = true;

    let mut context = TestContext::new_with_config(config).await;

    let certificate = Certificate::new_for_test(1.into(), 0);
    let id = certificate.hash();

    let res: CertificateId = context
        .client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    assert_eq!(id, res);
    assert!(context.certificate_receiver.try_recv().is_ok());

    let (recv_cert, header): (Certificate, Option<CertificateHeader>) = context
        .client
        .request("interop_debugGetCertificate", rpc_params![id])
        .await
        .unwrap();

    assert!(header.is_some());
    let header = header.unwrap();
    assert_eq!(header.certificate_id, id);
    assert_eq!(recv_cert.hash(), id);
    assert_eq!(header.status, CertificateStatus::Pending);

    let mut certificate = Certificate::new_for_test(1.into(), 0);
    certificate.prev_local_exit_root = [2; 32].into();
    let id2 = certificate.hash();

    let res: CertificateId = context
        .client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    assert_eq!(id2, res);
    assert!(context.certificate_receiver.try_recv().is_ok());

    // Retrieve 1
    let (recv_cert, header): (Certificate, Option<CertificateHeader>) = context
        .client
        .request("interop_debugGetCertificate", rpc_params![id])
        .await
        .unwrap();

    assert!(header.is_some());
    let header = header.unwrap();
    assert_eq!(header.certificate_id, id);
    assert_eq!(recv_cert.hash(), id);
    assert_eq!(header.status, CertificateStatus::Pending);

    // Retrieve 2
    let (recv_cert, header): (Certificate, Option<CertificateHeader>) = context
        .client
        .request("interop_debugGetCertificate", rpc_params![id2])
        .await
        .unwrap();

    assert!(header.is_some());
    let header = header.unwrap();
    assert_eq!(header.certificate_id, id2);
    assert_eq!(recv_cert.hash(), id2);
    assert_eq!(header.status, CertificateStatus::Pending);
}

#[rstest]
#[test_log::test(tokio::test)]
async fn debug_get_certificate_after_overwrite_with_debug_false() {
    let mut config = TestContext::get_default_config();
    config.debug_mode = false;

    let mut context = TestContext::new_with_config(config).await;

    let certificate = Certificate::new_for_test(1.into(), 0);
    let id = certificate.hash();

    let res: CertificateId = context
        .client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    assert_eq!(id, res);
    assert!(context.certificate_receiver.try_recv().is_ok());

    let payload: Result<(Certificate, Option<CertificateHeader>), ClientError> = context
        .client
        .request("interop_debugGetCertificate", rpc_params![id])
        .await;

    let error = payload.unwrap_err();

    let expected_message = format!("Resource not found: Certificate({:#})", id);
    assert!(matches!(error, ClientError::Call(obj) if obj.message() == expected_message));

    let mut certificate = Certificate::new_for_test(1.into(), 0);
    certificate.prev_local_exit_root = [2; 32].into();
    let id2 = certificate.hash();

    let res: CertificateId = context
        .client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    assert_eq!(id2, res);
    assert!(context.certificate_receiver.try_recv().is_ok());

    let payload: Result<(Certificate, Option<CertificateHeader>), ClientError> = context
        .client
        .request("interop_debugGetCertificate", rpc_params![id])
        .await;

    let error = payload.unwrap_err();

    let expected_message = format!("Resource not found: Certificate({:#})", id);
    assert!(matches!(error, ClientError::Call(obj) if obj.message() == expected_message));

    let payload: Result<(Certificate, Option<CertificateHeader>), ClientError> = context
        .client
        .request("interop_debugGetCertificate", rpc_params![id2])
        .await;

    let error = payload.unwrap_err();

    let expected_message = format!("Resource not found: Certificate({:#})", id2);
    assert!(matches!(error, ClientError::Call(obj) if obj.message() == expected_message));
}
