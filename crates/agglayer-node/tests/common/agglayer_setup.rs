use std::{path::Path, thread::JoinHandle, time::Duration};

use agglayer_config::{log::LogLevel, TelemetryConfig};
use agglayer_prover::fake::FakeProver;
use anyhow::Result;
use ethers::{
    core::k256::ecdsa::SigningKey,
    signers::{coins_bip39::English, LocalWallet, MnemonicBuilder, Wallet},
};
use jsonrpsee::ws_client::{WsClient, WsClientBuilder};
use tokio_util::sync::CancellationToken;

use super::l1_setup::L1Docker;
use crate::common::l1_setup::{self, next_available_addr};

const PHRASE: &str = "test test test test test test test test test test test junk";

pub async fn setup_agglayer(tmp_dir: &Path) -> (JoinHandle<Result<()>>, L1Docker, WsClient) {
    let name = std::thread::current().name().unwrap().replace("::", "_");
    let l1 = l1_setup::L1Docker::new(name).await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("L1 RPC: {}", l1.rpc);
    println!("L1 WS RPC: {}", l1.ws);

    // Make the mock prover pass
    fail::cfg(
        "notifier::certifier::certify::before_verifying_proof",
        "return()",
    )
    .unwrap();

    let mut config = agglayer_config::Config::new(tmp_dir);
    let prover_config = agglayer_config::prover::ProverConfig {
        grpc_endpoint: next_available_addr(),
        telemetry: TelemetryConfig {
            addr: next_available_addr(),
        },
        ..Default::default()
    };

    // spawning fake prover as we don't want to hit SP1
    let fake_prover = FakeProver::default();
    let endpoint = prover_config.grpc_endpoint;

    config.prover_entrypoint = format!("http://{}", endpoint);
    let cancellation = CancellationToken::new();

    FakeProver::spawn_at(fake_prover, endpoint, cancellation.clone())
        .await
        .unwrap();

    let wallet = get_signer(1);

    let mut rng = rand::thread_rng();
    let (_key, uuid) = LocalWallet::encrypt_keystore(
        tmp_dir,
        &mut rng,
        wallet.signer().to_bytes(),
        "randpsswd",
        None,
    )
    .unwrap();

    let key_path = tmp_dir.join(uuid);

    let addr = next_available_addr();
    config.rpc.port = addr.port();
    println!("RPC port: {}", config.rpc.port);

    config.telemetry.addr = next_available_addr();
    config.log.level = LogLevel::Debug;
    config.l1.node_url = l1.rpc.parse().unwrap();
    config.l1.ws_node_url = l1.ws.parse().unwrap();
    config.l1.rollup_manager_contract = "0xA51c1fc2f0D1a1b8494Ed1FE312d7C3a78Ed91C0"
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

    let config_file = tmp_dir.join("config.toml");
    println!("config_file: {:?}", config_file);
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
            let _result = handle.join();
            println!("{:?}", _result);
            panic!("Server has finished");
        }

        max_attempts -= 1;
    };

    assert!(!handle.is_finished());

    (handle, l1, client)
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
