use std::{fs, io, path::Path, time::Duration};

use agglayer_config::{log::LogLevel, Config};
use agglayer_prover::fake::FakeProver;
use alloy::signers::local::{coins_bip39::English, MnemonicBuilder, PrivateKeySigner};
use jsonrpsee::ws_client::{WsClient, WsClientBuilder};
use pessimistic_proof::ELF;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

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
    let l1 = l1_setup::L1Docker::new(&name).await;
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
        grpc_endpoint: next_available_addr().into(),
        telemetry: agglayer_prover_config::TelemetryConfig {
            addr: next_available_addr().into(),
        },
        ..Default::default()
    };

    // spawning fake prover as we don't want to hit SP1
    let fake_prover = FakeProver::new(ELF).await.unwrap();
    let endpoint = prover_config.grpc_endpoint;

    config.prover_entrypoint = format!("http://{endpoint}");
    let cancellation = token.unwrap_or_default();
    FakeProver::spawn_at(fake_prover, endpoint, cancellation.clone())
        .await
        .unwrap();

    // Create keystore file with embedded content for Docker compatibility
    let key_path = config_path.join("test_keystore.json");
    let password = "randpsswd";

    // Write the keystore content to a temporary file
    let keystore_content = get_test_keystore_content();
    fs::write(&key_path, keystore_content).unwrap();

    // Configure authentication to use the keystore file
    config.auth = agglayer_config::AuthConfig::Local(agglayer_config::LocalConfig {
        private_keys: vec![agglayer_config::PrivateKey {
            path: key_path,
            password: password.to_string(),
        }],
    });

    config.tls = Some(agglayer_config::TlsConfig {
        certificate: dev_tls_certs::SERVER_CERT_PATH.into(),
        key: dev_tls_certs::SERVER_KEY_PATH.into(),
    });

    assert!(config.tls.is_some());

    let grpc_addr = next_available_addr();
    let grpc_tls_addr = next_available_addr();
    let readrpc_addr = next_available_addr();
    let readrpc_tls_addr = next_available_addr();
    let admin_addr = next_available_addr();
    let admin_tls_addr = next_available_addr();

    config.rpc.grpc.plain = grpc_addr.into();
    config.rpc.grpc.tls = grpc_tls_addr.into();
    config.rpc.readrpc.plain = readrpc_addr.into();
    config.rpc.readrpc.tls = readrpc_tls_addr.into();
    config.rpc.admin.plain = admin_addr.into();
    config.rpc.admin.tls = admin_tls_addr.into();

    config.telemetry.addr = next_available_addr().into();
    config.log.level = LogLevel::Debug;
    config.l1.node_url = l1.rpc.parse().unwrap();
    config.l1.ws_node_url = l1.ws.parse().unwrap();
    config.l1.rollup_manager_contract = "0x0B306BF915C4d645ff596e518fAf3F9669b97016"
        .parse()
        .unwrap();
    config.l1.polygon_zkevm_global_exit_root_v2_contract =
        "0x610178dA211FEF7D417bC0e6FeD39F05609AD788"
            .parse()
            .unwrap();

    let config_file = config_path.join("config.toml");
    let toml = toml::to_string_pretty(&config).unwrap();
    fs::write(&config_file, toml).unwrap();

    let graceful_shutdown_token = cancellation.clone();
    let handle = std::thread::spawn(move || {
        agglayer_node::main(config_file, "test", Some(graceful_shutdown_token))
            .unwrap_or_else(|e| error!("Node failed: {e}"));
        _ = shutdown.send(());
    });
    tokio::time::sleep(Duration::from_secs(5)).await;

    let url = format!("wss://{}/", readrpc_tls_addr);
    info!("Connecting to {url}");

    let mut interval = tokio::time::interval(Duration::from_secs(10));
    let mut max_attempts = 4;
    let client = loop {
        if max_attempts == 0 {
            panic!("Failed to connect to the node RPC server {url}");
        }
        interval.tick().await;

        let client_res = WsClientBuilder::default()
            .with_custom_cert_store({
                // Set up our testing certificate as a CA root.
                let mut certs = rustls::RootCertStore::empty();
                let cert_file = fs::File::open(dev_tls_certs::CA_CERT_PATH)
                    .expect("Certificate file failed to open");
                let mut cert_file = io::BufReader::new(cert_file);
                for cert in rustls_pemfile::certs(&mut cert_file) {
                    let cert = cert.expect("Malformed certificate");
                    certs.add(cert).expect("Cannot add root certificate")
                }
                assert_eq!(certs.len(), 1);

                jsonrpsee::ws_client::CustomCertStore::builder()
                    .with_root_certificates(certs)
                    .with_no_client_auth()
            })
            .build(&url)
            .await
            .inspect_err(|e| warn!("Client startup error: {e}."));

        if let Ok(client) = client_res {
            break client;
        }

        if handle.is_finished() {
            let _result = handle.join();
            println!("Agglayer result: {_result:?}");
            panic!("Server has finished");
        }

        max_attempts -= 1;
        debug!("Client startup failed, {max_attempts} remaining attempts.");
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

pub fn get_signer(index: u32) -> PrivateKeySigner {
    // Access mnemonic phrase with password.
    // Child key at derivation path: m/44'/60'/0'/0/{index}.
    MnemonicBuilder::<English>::default()
        .phrase(PHRASE)
        .index(index)
        .unwrap()
        .build()
        .unwrap()
}

const fn get_test_keystore_content() -> &'static str {
    r#"{
  "crypto": {
    "cipher": "aes-128-ctr",
    "cipherparams": {
      "iv": "192834bb98d005cf1c9f12644c433431"
    },
    "ciphertext": "c8c7274be71641e467a53177b657b86731469f21af33c8f30cac7d4c34e81d96",
    "kdf": "scrypt",
    "kdfparams": {
      "dklen": 32,
      "n": 8192,
      "p": 1,
      "r": 8,
      "salt": "d56f2360d3214a1a95118e69e0cc533f7a5f9b5924041ee7f3f532a41da47e0f"
    },
    "mac": "e11920c6df25d3a25e557b3639481cca1a8702a6b9ca643e338b60e5603de279"
  },
  "id": "27833fa7-1081-474c-9417-bef6d869bd58",
  "version": 3
}"#
}
