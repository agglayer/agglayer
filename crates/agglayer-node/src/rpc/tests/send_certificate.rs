use std::{net::IpAddr, sync::Arc};

use agglayer_config::Config;
use agglayer_types::{Certificate, CertificateId, NetworkId};
use ethers::{providers, signers::Signer as _};
use jsonrpsee::{core::client::ClientT, http_client::HttpClientBuilder, rpc_params};

use super::next_available_addr;
use crate::{
    kernel::Kernel,
    rpc::{tests::DummyStore, AgglayerImpl},
    service::AgglayerService,
};

#[test_log::test(tokio::test)]
async fn send_certificate_method_can_be_called_and_succeed() {
    let mut config = Config::new_for_test();
    config
        .proof_signers
        .insert(1, Certificate::wallet_for_test(NetworkId::new(1)).address());
    let addr = next_available_addr();
    if let IpAddr::V4(ip) = addr.ip() {
        config.rpc.host = ip;
    }
    config.rpc.port = addr.port();

    let config = Arc::new(config);

    let (provider, _mock) = providers::Provider::mocked();
    let (certificate_sender, mut certificate_receiver) = tokio::sync::mpsc::channel(1);

    let kernel = Kernel::new(Arc::new(provider), config.clone());

    let service = AgglayerService::new(
        kernel,
        certificate_sender,
        Arc::new(DummyStore {}),
        Arc::new(DummyStore {}),
        Arc::new(DummyStore {}),
        config.clone(),
    );

    let _server_handle = AgglayerImpl::new(Arc::new(service)).start().await.unwrap();

    let url = format!("http://{}/", config.rpc_addr());
    let client = HttpClientBuilder::default().build(url).unwrap();

    let _: CertificateId = client
        .request(
            "interop_sendCertificate",
            rpc_params![Certificate::new_for_test(1.into(), 0)],
        )
        .await
        .unwrap();

    assert!(certificate_receiver.try_recv().is_ok());
}

#[test_log::test(tokio::test)]
async fn send_certificate_method_can_be_called_and_fail() {
    let mut config = Config::new_for_test();
    let addr = next_available_addr();
    if let IpAddr::V4(ip) = addr.ip() {
        config.rpc.host = ip;
    }
    config.rpc.port = addr.port();

    let config = Arc::new(config);

    let (provider, _mock) = providers::Provider::mocked();
    let (certificate_sender, certificate_receiver) = tokio::sync::mpsc::channel(1);

    let kernel = Kernel::new(Arc::new(provider), config.clone());

    let service = AgglayerService::new(
        kernel,
        certificate_sender,
        Arc::new(DummyStore {}),
        Arc::new(DummyStore {}),
        Arc::new(DummyStore {}),
        config.clone(),
    );

    let _server_handle = AgglayerImpl::new(Arc::new(service)).start().await.unwrap();

    let url = format!("http://{}/", config.rpc_addr());
    let client = HttpClientBuilder::default().build(url).unwrap();

    drop(certificate_receiver);

    let res: Result<(), _> = client
        .request(
            "interop_sendCertificate",
            rpc_params![Certificate::new_for_test(0.into(), 0)],
        )
        .await;

    assert!(res.is_err());
}

#[test_log::test(tokio::test)]
async fn send_certificate_method_requires_known_signer() {
    let mut config = Config::new_for_test();
    // Willingly insert a signer that is not the one that’ll be used down below
    config
        .proof_signers
        .insert(1, Certificate::wallet_for_test(NetworkId::new(2)).address());
    let addr = next_available_addr();
    if let IpAddr::V4(ip) = addr.ip() {
        config.rpc.host = ip;
    }
    config.rpc.port = addr.port();

    let config = Arc::new(config);

    let (provider, _mock) = providers::Provider::mocked();
    let (certificate_sender, _certificate_receiver) = tokio::sync::mpsc::channel(1);

    let kernel = Kernel::new(Arc::new(provider), config.clone());

    let service = AgglayerService::new(
        kernel,
        certificate_sender,
        Arc::new(DummyStore {}),
        Arc::new(DummyStore {}),
        Arc::new(DummyStore {}),
        config.clone(),
    );
    let _server_handle = AgglayerImpl::new(Arc::new(service)).start().await.unwrap();

    let url = format!("http://{}/", config.rpc_addr());
    let client = HttpClientBuilder::default().build(url).unwrap();

    let send_request: Result<CertificateId, _> = client
        .request(
            "interop_sendCertificate",
            rpc_params![Certificate::new_for_test(1.into(), 0)],
        )
        .await;

    assert!(send_request.is_err());
}
