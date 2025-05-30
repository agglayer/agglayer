use std::{path::Path, time::Duration};

use agglayer_config::{tracing::TracingLevel, Config};
use agglayer_prover::fake::FakeProver;
use ethers::{
    core::k256::ecdsa::SigningKey,
    signers::{coins_bip39::English, LocalWallet, MnemonicBuilder, Wallet},
};
use jsonrpsee::ws_client::{WsClient, WsClientBuilder};
use pessimistic_proof::ELF;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use crate::l1_setup::{self, next_available_addr, L1Docker};

const PHRASE: &str = "test test test test test test test test test test test junk";

#[macro_export]
macro_rules! wait_for_settlement_or_error {
    ($client:ident, $certificate_id:ident) => {{
        async {
            use jsonrpsee::{core::client::ClientT, rpc_params};
            let mut result;
            loop {
                let response: agglayer_types::CertificateHeader = $client
                    .request(
                        "interop_getCertificateHeader",
                        jsonrpsee::rpc_params![$certificate_id],
                    )
                    .await
                    .unwrap();

                result = response;

                match result.status {
                    agglayer_types::CertificateStatus::InError { .. }
                    | agglayer_types::CertificateStatus::Settled => {
                        break;
                    }
                    _ => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                    }
                }
            }

            return result;
        }
    }};
}

pub async fn start_l1() -> L1Docker {
    let name = std::thread::current().name().unwrap().replace("::", "_");
    let l1 = l1_setup::L1Docker::new(name).await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    l1
}

pub async fn start_agglayer(
    config_path: &Path,
    l1: &L1Docker,
    config: Option<agglayer_config::Config>,
    token: Option<CancellationToken>,
) -> (oneshot::Receiver<()>, WsClient, CancellationToken) {
    let (shutdown, receiver) = oneshot::channel();

    // Make the mock prover pass
    fail::cfg(
        "notifier::certifier::certify::before_verifying_proof",
        "return()",
    )
    .unwrap();

    let mut config = config.unwrap_or_else(|| agglayer_config::Config::new(config_path));
    let prover_config = agglayer_prover_config::ProverConfig {
        grpc_endpoint: next_available_addr(),
        telemetry: agglayer_prover_config::TelemetryConfig {
            addr: next_available_addr(),
        },
        ..Default::default()
    };

    // spawning fake prover as we don't want to hit SP1
    let fake_prover = FakeProver::new(ELF);
    let endpoint = prover_config.grpc_endpoint;

    config.prover_entrypoint = format!("http://{endpoint}");
    let cancellation = token.unwrap_or_default();
    FakeProver::spawn_at(fake_prover, endpoint, cancellation.clone())
        .await
        .unwrap();

    let wallet = get_signer(1);

    let (_key, uuid) = LocalWallet::encrypt_keystore(
        config_path,
        &mut ethers::core::rand::thread_rng(),
        wallet.signer().to_bytes(),
        "randpsswd",
        None,
    )
    .unwrap();

    let key_path = config_path.join(uuid);

    let grpc_addr = next_available_addr();
    let readrpc_addr = next_available_addr();
    let admin_addr = next_available_addr();
    config.rpc.grpc_port = grpc_addr.port();
    config.rpc.readrpc_port = readrpc_addr.port();
    config.rpc.admin_port = admin_addr.port();

    config.telemetry.addr = next_available_addr();
    config.log.level = TracingLevel::Debug;
    config.l1.node_url = l1.rpc.parse().unwrap();
    config.l1.ws_node_url = l1.ws.parse().unwrap();
    config.l1.rollup_manager_contract = "0x0B306BF915C4d645ff596e518fAf3F9669b97016"
        .parse()
        .unwrap();
    config.l1.polygon_zkevm_global_exit_root_v2_contract =
        "0x610178dA211FEF7D417bC0e6FeD39F05609AD788"
            .parse()
            .unwrap();
    config.auth = agglayer_config::AuthConfig::Local(agglayer_config::LocalConfig {
        private_keys: vec![agglayer_config::PrivateKey {
            path: key_path,
            password: "randpsswd".into(),
        }],
    });

    let config_file = config_path.join("config.toml");
    let toml = toml::to_string_pretty(&config).unwrap();
    std::fs::write(&config_file, toml).unwrap();

    let graceful_shutdown_token = cancellation.clone();
    let handle = std::thread::spawn(move || {
        if let Err(error) = agglayer_node::main(config_file, "test", Some(graceful_shutdown_token))
        {
            eprintln!("Error: {error}");
        }
        _ = shutdown.send(());
    });
    let url = format!("ws://{}/", config.readrpc_addr());

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
            let _result = handle.join();
            println!("Agglayer result: {_result:?}");
            panic!("Server has finished");
        }

        max_attempts -= 1;
    };

    assert!(!handle.is_finished());

    (receiver, client, cancellation)
}

pub async fn setup_network(
    tmp_dir: &Path,
    config: Option<Config>,
    token: Option<CancellationToken>,
) -> (oneshot::Receiver<()>, L1Docker, WsClient) {
    let l1 = start_l1().await;
    let (receiver, client, _token) = start_agglayer(tmp_dir, &l1, config, token).await;

    (receiver, l1, client)
}

pub fn get_signer(index: u32) -> Wallet<SigningKey> {
    // Access mnemonic phrase with password.
    // Child key at derivation path: m/44'/60'/0'/0/{index}.
    MnemonicBuilder::<English>::default()
        .phrase(PHRASE)
        .index(index)
        .unwrap()
        .build()
        .unwrap()
}
