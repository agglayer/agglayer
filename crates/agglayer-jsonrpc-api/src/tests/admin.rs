use agglayer_storage::stores::{
    NetworkInfoReader as _, PendingCertificateReader as _, StateReader as _,
};
use agglayer_types::{
    Certificate, CertificateStatus, CertificateStatusError, Digest, Height, NetworkId,
};
use jsonrpsee::{
    core::{client::ClientT as _, ClientError},
    rpc_params,
};
use rstest::*;

use crate::testutils::{context, TestContext};

#[test]
fn certificate_update_errors_keep_jsonrpc_codes_through_eyre_context() {
    use agglayer_storage::error::Error as StorageError;
    use jsonrpsee::types::ErrorObjectOwned;

    let actual = ErrorObjectOwned::from(crate::Error::from_certificate_update(
        eyre::Error::from(StorageError::Unexpected("database write".into()))
            .wrap_err("state pending cache"),
    ));
    let expected = ErrorObjectOwned::from(crate::Error::internal("Unable to update certificate"));
    assert_eq!(actual, expected);

    let detail = "Cannot change status of a settled certificate";
    let actual = ErrorObjectOwned::from(crate::Error::from_certificate_update(
        eyre::Error::from(StorageError::UnprocessedAction(detail.into()))
            .wrap_err("certificate header"),
    ));
    let expected = ErrorObjectOwned::from(crate::Error::InvalidArgument(detail.into()));
    assert_eq!(actual, expected);
}

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

#[rstest]
#[awt]
#[test_log::test(tokio::test)]
async fn admin_pending_proven_and_force_push_update_cache(#[future] context: TestContext) {
    let network = NetworkId::new(1);
    let certificate = Certificate::new_for_test(network, Height::new(5));
    let id = certificate.hash();
    let status =
        CertificateStatus::error(CertificateStatusError::InternalError("admin retry".into()));
    let () = context
        .admin_client
        .request(
            "admin_forcePushPendingCertificate",
            rpc_params![certificate.clone(), status.clone()],
        )
        .await
        .unwrap();
    assert_eq!(
        context
            .state_store
            .get_certificate_header(&id)
            .unwrap()
            .unwrap()
            .status,
        status
    );
    assert_eq!(
        context
            .state_store
            .get_latest_pending_certificate_id(network)
            .unwrap(),
        Some(id)
    );
    assert_eq!(
        context
            .state_store
            .get_latest_pending_height(network)
            .unwrap(),
        Some(certificate.height)
    );
    assert_eq!(
        context
            .pending_store
            .get_latest_pending_certificate_for_network(&network)
            .unwrap(),
        Some((id, certificate.height))
    );
    assert_eq!(
        context
            .state_store
            .get_network_info(network)
            .unwrap()
            .network_type,
        agglayer_types::NetworkType::Unspecified
    );
    assert_eq!(
        context
            .state_store
            .get_latest_proven_certificate_id(network)
            .unwrap(),
        None
    );
    let () = context
        .admin_client
        .request("admin_setLatestProvenCertificate", rpc_params![id])
        .await
        .unwrap();
    assert_eq!(
        context
            .state_store
            .get_latest_proven_certificate_id(network)
            .unwrap(),
        Some(id)
    );
    assert_eq!(
        context
            .pending_store
            .get_latest_proven_certificate_per_network(&network)
            .unwrap(),
        Some((network, certificate.height, id))
    );
    let older = Certificate::new_for_test(network, Height::ZERO);
    context
        .state_store
        .insert_certificate_header(&older, CertificateStatus::Pending)
        .unwrap();
    let () = context
        .admin_client
        .request(
            "admin_setLatestPendingCertificate",
            rpc_params![older.hash()],
        )
        .await
        .unwrap();
    assert_eq!(
        context
            .state_store
            .get_latest_pending_certificate_id(network)
            .unwrap(),
        Some(older.hash())
    );
    assert_eq!(
        context
            .pending_store
            .get_latest_pending_certificate_for_network(&network)
            .unwrap(),
        Some((older.hash(), older.height))
    );
    // An arbitrary admin status edit must leave latest-proven ownership alone.
    let () = context
        .admin_client
        .request(
            "admin_forceEditCertificate",
            rpc_params![
                older.hash(),
                "process-now=false",
                "set-status,from=Pending,to=Proven",
                Option::<String>::None
            ],
        )
        .await
        .unwrap();
    assert_eq!(
        context
            .state_store
            .get_latest_proven_certificate_id(network)
            .unwrap(),
        Some(id)
    );
}

#[rstest]
#[awt]
#[test_log::test(tokio::test)]
async fn unknown_network_keeps_jsonrpc_error_conversion(#[future] context: TestContext) {
    let result: Result<agglayer_types::NetworkInfo, ClientError> = context
        .api_client
        .request("interop_getNetworkInfo", rpc_params![NetworkId::new(99)])
        .await;
    let ClientError::Call(error) = result.unwrap_err() else {
        panic!("Expected JSON-RPC error");
    };
    let expected = jsonrpsee::types::ErrorObjectOwned::from(crate::Error::from(
        agglayer_rpc::GetNetworkInfoError::UnknownNetworkType {
            network_id: NetworkId::new(99),
        },
    ));
    assert_eq!(error, expected);
}
