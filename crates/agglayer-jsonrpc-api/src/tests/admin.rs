use agglayer_storage::stores::{PendingCertificateReader as _, StateWriter as _};
use agglayer_types::{
    Certificate, CertificateStatus, CertificateStatusError, Digest, Height, NetworkId,
};
use jsonrpsee::{
    core::{client::ClientT as _, ClientError},
    rpc_params,
};
use rstest::*;

use crate::testutils::{context, TestContext};

#[rstest]
#[awt]
#[test_log::test(tokio::test)]
async fn set_latest_pending_certificate_moves_the_pointer(#[future] context: TestContext) {
    let network_id = NetworkId::new(1);
    let certificate = Certificate::new_for_test(network_id, Height::ZERO);
    let certificate_id = certificate.hash();

    // An errored header also exercises the post-write metric reconciliation,
    // which rereads the pointer and republishes this status.
    context
        .state_store
        .insert_certificate_header(
            &certificate,
            CertificateStatus::error(CertificateStatusError::InternalError("test".to_string())),
        )
        .unwrap();

    let () = context
        .admin_client
        .request(
            "admin_setLatestPendingCertificate",
            rpc_params![certificate_id],
        )
        .await
        .unwrap();

    assert_eq!(
        context
            .pending_store
            .get_latest_pending_certificate_for_network(&network_id)
            .unwrap(),
        Some((certificate_id, Height::ZERO)),
    );
}

#[rstest]
#[awt]
#[test_log::test(tokio::test)]
async fn set_latest_pending_certificate_rejects_unknown_certificates(
    #[future] context: TestContext,
) {
    let missing_id = Digest([7; 32]);

    let result: Result<(), ClientError> = context
        .admin_client
        .request("admin_setLatestPendingCertificate", rpc_params![missing_id])
        .await;

    let error = result.unwrap_err();
    assert!(
        matches!(&error, ClientError::Call(obj) if obj.message().contains("Resource not found")),
        "unexpected error: {error}"
    );
    assert!(context
        .pending_store
        .get_latest_pending_certificate_for_network(&NetworkId::new(1))
        .unwrap()
        .is_none());
}
