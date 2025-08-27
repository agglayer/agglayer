use agglayer_types::{Certificate, CertificateId, Height, NetworkId, NetworkStatus};
use jsonrpsee::{
    core::{client::ClientT, ClientError},
    rpc_params,
};

use crate::testutils::TestContext;

#[test_log::test(tokio::test)]
async fn get_network_status_for_network_with_no_certificates() {
    let context = TestContext::new_with_config(TestContext::get_default_config()).await;
    let network_id = NetworkId::new(1);

    let result: Result<NetworkStatus, ClientError> = context
        .api_client
        .request("interop_getNetworkStatus", rpc_params![network_id])
        .await;

    // Should succeed and return a NetworkStatus with default values when no
    // certificates exist
    match result {
        Ok(status) => {
            assert_eq!(status.network_id, network_id);
            assert_eq!(status.network_status, "TBD");
            assert_eq!(status.network_type, "Unknown"); // Default when no certificates
            assert_eq!(status.settled_height, Height::from(0u64));
            assert_eq!(status.latest_pending_height, 0);
            assert_eq!(status.latest_pending_status, "Unknown");
            assert!(status.latest_pending_error.is_empty());
            assert_eq!(status.latest_epoch_with_settlement, 0);
        }
        Err(error) => {
            panic!("Expected success but got error: {}", error);
        }
    }
}

#[test_log::test(tokio::test)]
async fn get_network_status_for_network_with_certificates() {
    let mut config = TestContext::get_default_config();
    config.proof_signers.insert(
        1,
        Certificate::wallet_for_test(NetworkId::new(1))
            .address()
            .into(),
    );
    let mut context = TestContext::new_with_config(config).await;
    let network_id = NetworkId::new(1);

    // First, send a certificate to have some data
    let certificate = Certificate::new_for_test(network_id, Height::ZERO);
    let _cert_id: CertificateId = context
        .api_client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    // Receive the certificate to ensure it's processed
    assert!(context.certificate_receiver.try_recv().is_ok());

    let result: Result<NetworkStatus, ClientError> = context
        .api_client
        .request("interop_getNetworkStatus", rpc_params![network_id])
        .await;

    // Should succeed and return a NetworkStatus
    match result {
        Ok(status) => {
            assert_eq!(status.network_id, network_id);
            assert_eq!(status.network_status, "TBD");
            assert_eq!(status.latest_pending_height, 0);
            assert_eq!(status.latest_pending_status, "Pending");
            assert!(status.latest_pending_error.is_empty());
        }
        Err(error) => {
            panic!("Expected success but got error: {}", error);
        }
    }
}

#[test_log::test(tokio::test)]
async fn get_network_status_validates_network_id() {
    let context = TestContext::new_with_config(TestContext::get_default_config()).await;
    let result: Result<NetworkStatus, ClientError> = context
        .api_client
        .request("interop_getNetworkStatus", rpc_params![])
        .await;

    // Should return an error for invalid parameters
    assert!(result.is_err());
}
