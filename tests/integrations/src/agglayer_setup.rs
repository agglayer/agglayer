use std::{
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use agglayer_config::{log::LogLevel, Config};
use agglayer_types::{CertificateHeader, CertificateId, CertificateStatus};
use alloy::{
    network::Ethereum,
    providers::{Provider as _, RootProvider},
    signers::local::{coins_bip39::English, MnemonicBuilder, PrivateKeySigner},
};
use fs2::FileExt;
use jsonrpsee::ws_client::{WsClient, WsClientBuilder};
use prover_config::{MockProverConfig, ProverType};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use crate::l1_setup::{self, next_available_addr, L1Docker};

const PHRASE: &str = "test test test test test test test test test test test junk";
// Generous so node startup can finish when several integration tests (each
// spinning up a multi-threaded agglayer node) run concurrently under the
// `resource-limited` nextest group on a loaded CI runner.
const AGGLAYER_RPC_CONNECT_TIMEOUT: Duration = Duration::from_secs(90);
const AGGLAYER_RPC_POLL_INTERVAL: Duration = Duration::from_millis(250);
const CERTIFICATE_STATUS_POLL_INTERVAL: Duration = Duration::from_millis(250);
const DEFAULT_WAIT_TIMEOUT: Duration = Duration::from_secs(30);

#[macro_export]
macro_rules! wait_for_settlement_or_error {
    ($client:ident, $certificate_id:expr) => {{
        async {
            integrations::agglayer_setup::wait_for_terminal_certificate_status(
                &$client,
                $certificate_id,
            )
            .await
        }
    }};
}

pub async fn start_l1() -> L1Docker {
    // nextest runs each test in its own process, and the thread name is not
    // guaranteed to be unique across those processes, so it cannot name the
    // Docker container on its own. Combine the process id with a per-process
    // counter so the container name stays unique even when tests run in parallel
    // (the `resource-limited` group), avoiding `docker run --name` collisions.
    static L1_CONTAINER_SEQ: AtomicU64 = AtomicU64::new(0);
    let name = format!(
        "agglayer-integration-l1-{}-{}",
        std::process::id(),
        L1_CONTAINER_SEQ.fetch_add(1, Ordering::Relaxed),
    );
    l1_setup::L1Docker::new(name).await
}

/// Like [`start_agglayer`], but also returns the effective [`Config`] (with
/// the randomized ports, e.g. `telemetry.addr`) so tests can reach the
/// node's auxiliary endpoints.
pub async fn start_agglayer_with_config(
    config_path: &Path,
    l1: &L1Docker,
    config: Option<agglayer_config::Config>,
    token: Option<CancellationToken>,
) -> (oneshot::Receiver<()>, WsClient, CancellationToken, Config) {
    let (shutdown, receiver) = oneshot::channel();

    // Make the mock prover pass
    fail::cfg(
        "notifier::certifier::certify::before_verifying_proof",
        "return()",
    )
    .unwrap();

    let mut config = config.unwrap_or_else(|| agglayer_config::Config::new(config_path));

    config.prover = ProverType::MockProver(MockProverConfig::default());

    let cancellation = token.unwrap_or_default();

    // Create keystore file with embedded content for Docker compatibility
    let key_path = config_path.join("test_keystore.json");
    let password = "randpsswd";

    // Write the keystore content to a temporary file
    let keystore_content = get_test_keystore_content();
    std::fs::write(&key_path, keystore_content).unwrap();

    // Configure authentication to use the keystore file
    config.auth = agglayer_config::AuthConfig::Local(agglayer_config::LocalConfig {
        private_keys: vec![agglayer_config::PrivateKey {
            path: key_path,
            password: password.to_string(),
        }],
    });

    let (grpc_addr, readrpc_addr, admin_addr, telemetry_addr) = {
        let _port_reservation_lock = tokio::task::spawn_blocking(PortReservationLock::acquire)
            .await
            .expect("Port reservation lock task panicked");

        (
            next_available_addr(),
            next_available_addr(),
            next_available_addr(),
            next_available_addr(),
        )
    };
    config.rpc.grpc_port = grpc_addr.port().into();
    config.rpc.readrpc_port = readrpc_addr.port().into();
    config.rpc.admin_port = admin_addr.port().into();

    config.telemetry.addr = telemetry_addr;
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

    // Tune settlement for the test L1: it mines ~1s blocks but its "safe" head
    // lags many blocks, so the prod SafeBlock policy would idle each certificate
    // for tens of seconds. Settle on latest with one confirmation, and poll a few
    // seconds apart (kept above the ~1s block time so an attempt is never
    // abandoned before it can mine, which would resubmit and double-settle).
    {
        let tx_config = &mut config.settlement.pessimistic_proof_tx_config;
        tx_config.settlement_policy =
            agglayer_config::settlement_service::SettlementPolicy::LatestBlock;
        tx_config.confirmations = 1;
        tx_config.retry_on_not_included_on_l1.initial_interval = std::time::Duration::from_secs(3);
        tx_config.retry_on_not_included_on_l1.max_interval = std::time::Duration::from_secs(6);
        tx_config.retry_on_transient_failure.initial_interval = std::time::Duration::from_secs(3);
        tx_config.retry_on_transient_failure.max_interval = std::time::Duration::from_secs(6);
    }

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

    let start = tokio::time::Instant::now();
    let client = loop {
        if let Ok(client) = WsClientBuilder::default().build(&url).await {
            break client;
        }

        if start.elapsed() >= AGGLAYER_RPC_CONNECT_TIMEOUT {
            panic!("Failed to connect to the server");
        }

        if handle.is_finished() {
            let _result = handle.join();
            println!("Agglayer result: {_result:?}");
            panic!("Server has finished");
        }

        tokio::time::sleep(AGGLAYER_RPC_POLL_INTERVAL).await;
    };

    assert!(!handle.is_finished());

    (receiver, client, cancellation, config)
}

pub async fn start_agglayer(
    config_path: &Path,
    l1: &L1Docker,
    config: Option<agglayer_config::Config>,
    token: Option<CancellationToken>,
) -> (oneshot::Receiver<()>, WsClient, CancellationToken) {
    let (receiver, client, cancellation, _config) =
        start_agglayer_with_config(config_path, l1, config, token).await;
    (receiver, client, cancellation)
}

pub async fn setup_network(
    tmp_dir: &Path,
    config: Option<Config>,
    token: Option<CancellationToken>,
) -> (oneshot::Receiver<()>, L1Docker, WsClient) {
    let (receiver, l1, client, _config) = setup_network_with_config(tmp_dir, config, token).await;

    (receiver, l1, client)
}

/// Like [`setup_network`], but also returns the effective [`Config`] (with
/// the randomized ports, e.g. `telemetry.addr`) so tests can reach the
/// node's auxiliary endpoints.
pub async fn setup_network_with_config(
    tmp_dir: &Path,
    config: Option<Config>,
    token: Option<CancellationToken>,
) -> (oneshot::Receiver<()>, L1Docker, WsClient, Config) {
    let l1 = start_l1().await;
    let (receiver, client, _token, config) =
        start_agglayer_with_config(tmp_dir, &l1, config, token).await;

    (receiver, l1, client, config)
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

pub async fn wait_for_terminal_certificate_status(
    client: &WsClient,
    certificate_id: CertificateId,
) -> CertificateHeader {
    wait_for_terminal_certificate_status_with_timeout(client, certificate_id, None).await
}

pub async fn wait_for_terminal_certificate_status_with_timeout(
    client: &WsClient,
    certificate_id: CertificateId,
    timeout: Option<Duration>,
) -> CertificateHeader {
    use jsonrpsee::{core::client::ClientT, rpc_params};

    let start = tokio::time::Instant::now();
    let mut attempts = 0usize;

    loop {
        attempts += 1;

        let current_observation = match client
            .request::<CertificateHeader, _>(
                "interop_getCertificateHeader",
                rpc_params![certificate_id],
            )
            .await
        {
            Ok(response) => {
                let current_observation = format!("status={:?}", response.status);

                if matches!(
                    response.status,
                    CertificateStatus::InError { .. } | CertificateStatus::Settled
                ) {
                    return response;
                }

                current_observation
            }
            Err(error) => {
                if timeout.is_none() {
                    panic!(
                        "Failed to fetch certificate {certificate_id} status after {} attempts: \
                         {error}",
                        attempts,
                    );
                }

                format!("rpc_error={error}")
            }
        };

        if timeout.is_some_and(|timeout| start.elapsed() >= timeout) {
            panic!(
                "Timed out waiting for certificate {certificate_id} to settle after {} attempts. \
                 Last observation: {}",
                attempts, current_observation,
            );
        }

        tokio::time::sleep(certificate_status_poll_interval()).await;
    }
}

pub const fn certificate_status_poll_interval() -> Duration {
    CERTIFICATE_STATUS_POLL_INTERVAL
}

pub async fn wait_for_condition<F, Fut>(description: &str, timeout: Duration, mut check: F)
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = tokio::time::Instant::now();

    loop {
        if check().await {
            return;
        }

        if start.elapsed() >= timeout {
            panic!("Timed out waiting for {description}");
        }

        tokio::time::sleep(AGGLAYER_RPC_POLL_INTERVAL).await;
    }
}

pub async fn l1_block_number(l1: &L1Docker) -> Option<u64> {
    let url = reqwest::Url::parse(&l1.rpc).ok()?;
    RootProvider::<Ethereum>::new_http(url)
        .get_block_number()
        .await
        .ok()
}

pub async fn wait_for_l1_blocks(l1: &L1Docker, additional_blocks: u64) {
    // Poll tolerantly: a transient L1 RPC error yields `None` and re-polls within
    // the timeout budget instead of panicking through the loop. The target is
    // anchored to the first successful block read.
    let start = tokio::time::Instant::now();
    let mut target = None;

    loop {
        if let Some(current) = l1_block_number(l1).await {
            if current >= *target.get_or_insert(current + additional_blocks) {
                return;
            }
        }

        if start.elapsed() >= DEFAULT_WAIT_TIMEOUT {
            panic!("Timed out waiting for L1 to advance by {additional_blocks} blocks");
        }

        tokio::time::sleep(AGGLAYER_RPC_POLL_INTERVAL).await;
    }
}

struct PortReservationLock {
    _file: std::fs::File,
}

impl PortReservationLock {
    fn acquire() -> Self {
        let path = std::env::temp_dir().join("agglayer-integrations-port.lock");
        let file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&path)
            .unwrap_or_else(|error| {
                panic!("Failed to open port reservation lock {path:?}: {error}")
            });

        file.lock_exclusive().unwrap_or_else(|error| {
            panic!("Failed to lock port reservation file {path:?}: {error}")
        });

        Self { _file: file }
    }
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
