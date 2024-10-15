use std::time::Duration;

use agglayer_config::log::LogLevel;
use agglayer_prover::fake::FakeProver;
use agglayer_storage::tests::TempDBDir;
use agglayer_types::{
    Certificate, CertificateHeader, CertificateId, CertificateStatusError, LocalNetworkStateData,
};
use ethers::{signers::LocalWallet, utils::Anvil};
use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClientBuilder};
use pessimistic_proof_test_suite::forest::Forest;
use rstest::rstest;
use tokio_util::sync::CancellationToken;

#[rstest]
#[ignore = "ignore until signature"]
#[tokio::test]
#[timeout(Duration::from_secs(60))]
async fn successfully_push_certificate() {
    let anvil = Anvil::new().block_time(1u64).spawn();
    let tmp_dir = TempDBDir::new();
    let mut config = agglayer_config::Config::new(&tmp_dir.path);
    let prover_config = agglayer_config::prover::ProverConfig::default();

    // spawning fake prover as we don't want to hit SP1
    let fake_prover = FakeProver::default();
    let endpoint = prover_config.grpc_endpoint;
    let cancellation = CancellationToken::new();

    FakeProver::spawn_at(fake_prover, endpoint, cancellation.clone())
        .await
        .unwrap();

    let account = anvil.keys().first().cloned().unwrap();

    let mut rng = rand::thread_rng();
    let (_key, uuid) = LocalWallet::encrypt_keystore(
        &tmp_dir.path,
        &mut rng,
        account.to_bytes(),
        "randpsswd",
        None,
    )
    .unwrap();

    let key_path = tmp_dir.path.join(uuid);

    config.log.level = LogLevel::Debug;
    config.l1.node_url = anvil.endpoint().parse().unwrap();
    config.l1.ws_node_url = anvil.ws_endpoint().parse().unwrap();
    config.auth = agglayer_config::AuthConfig::Local(agglayer_config::LocalConfig {
        private_keys: vec![agglayer_config::PrivateKey {
            path: key_path,
            password: "randpsswd".into(),
        }],
    });

    let config_file = tmp_dir.path.join("config.toml");
    let toml = toml::to_string_pretty(&config).unwrap();
    std::fs::write(&config_file, toml).unwrap();

    let handle = std::thread::spawn(move || agglayer_node::main(config_file));
    let url = format!("ws://{}/", config.rpc_addr());

    let mut interval = tokio::time::interval(Duration::from_secs(10));
    let mut max_attempts = 20;
    let client = loop {
        if max_attempts == 0 {
            panic!("Failed to connect to the server");
        }
        interval.tick().await;
        if let Ok(client) = WsClientBuilder::default().build(&url).await {
            break client;
        }

        if handle.is_finished() {
            let result = handle.join();
            println!("{:?}", result);
            panic!("Server has finished");
        }

        max_attempts -= 1;
    };

    assert!(!handle.is_finished());
    let state = Forest::new(vec![]);

    let withdrawals = vec![];

    let (certificate, _signer) = state.clone().apply_events(&[], &withdrawals);

    let certificate_id: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    loop {
        let response: CertificateHeader = client
            .request("interop_getCertificateHeader", rpc_params![certificate_id])
            .await
            .unwrap();

        match response.status {
            agglayer_types::CertificateStatus::Pending => {
                println!("Certificate is pending");
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            // In success the test is valid
            agglayer_types::CertificateStatus::Proven => {
                println!("Certificate is proven");
                break;
            }

            // We can't go further than that with the current test setup
            // TODO: Add a way to generate valide certificate here
            agglayer_types::CertificateStatus::InError {
                error: CertificateStatusError::TrustedSequencerNotFound(_),
            } => break,
            agglayer_types::CertificateStatus::InError { error } => {
                panic!("{}", error)
            }
            _ => break,
        }
    }
}
