use std::path::PathBuf;

use alloy::{
    network::Ethereum,
    node_bindings::{Anvil, AnvilInstance},
    providers::{Provider as _, RootProvider},
};
use tokio::process::Command;

const L1_BACKEND_ENV: &str = "AGGLAYER_INTEGRATION_L1_BACKEND";
const L1_FIXTURE_ENV: &str = "AGGLAYER_INTEGRATION_L1_FIXTURE";
const ANVIL_BACKEND: &str = "anvil";
const DOCKER_BACKEND: &str = "docker";
const DEFAULT_ANVIL_FIXTURE: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/anvil-l1/state.json");

// L1 readiness polling budget: 240 * 250ms = 60s, matching the readiness budget
// in generate_anvil_l1_fixture.sh so a cold Docker image (the opt-in backend)
// has time to start. Anvil is ready in well under a second, so the larger cap
// only affects the Docker path.
const L1_READY_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(250);
const L1_READY_ATTEMPTS: usize = 240;

enum L1Instance {
    Docker { _container: DockerContainer },
    Anvil { _instance: AnvilInstance },
}

/// Owns a running Docker container and removes it on drop.
///
/// Holding this from the moment `docker run` succeeds guarantees the container
/// is cleaned up even if a later setup step (port discovery, RPC/WS readiness)
/// panics, instead of leaking an orphaned container that keeps holding its
/// published ports until it is removed manually.
struct DockerContainer {
    id: String,
}

pub struct L1Docker {
    _inner: L1Instance,
    pub ws: String,
    pub rpc: String,
}

impl L1Docker {
    pub async fn new(name: String) -> Self {
        match std::env::var(L1_BACKEND_ENV) {
            Ok(backend) if backend == ANVIL_BACKEND => Self::new_anvil().await,
            Ok(backend) if backend == DOCKER_BACKEND => Self::new_docker(name).await,
            Ok(backend) => panic!(
                "Unsupported L1 backend `{backend}`. Supported values: `{ANVIL_BACKEND}`, \
                 `{DOCKER_BACKEND}`, or unset"
            ),
            Err(std::env::VarError::NotPresent) => Self::new_anvil().await,
            Err(error) => panic!("Failed to read {L1_BACKEND_ENV}: {error}"),
        }
    }

    async fn new_anvil() -> Self {
        let fixture = fixture_path();
        assert!(
            fixture.exists(),
            "Anvil L1 fixture {} not found. Regenerate it with \
             tests/integrations/scripts/generate_anvil_l1_fixture.sh (requires Docker and a \
             matching foundry/anvil version).",
            fixture.display()
        );

        // Load the snapshot through the CLI `--load-state` flag rather than the
        // `anvil_loadState` RPC: only the CLI path restores the per-block
        // historical states captured with `--preserve-historical-states`, which
        // `eth_getTransactionBySenderAndNonce` needs to resolve a settled nonce.
        let anvil = Anvil::new()
            .chain_id(1337u64)
            .block_time(1u64)
            .arg("--auto-impersonate")
            .arg("--load-state")
            .arg(fixture.to_string_lossy().into_owned())
            .spawn();

        let rpc = anvil.endpoint_url().to_string();
        let ws = anvil.ws_endpoint();
        wait_for_rpc(&rpc).await;

        Self {
            _inner: L1Instance::Anvil { _instance: anvil },
            ws,
            rpc,
        }
    }

    async fn new_docker(name: String) -> Self {
        let docker = Command::new("docker")
            .args([
                "run",
                "-p",
                "127.0.0.1::8545",
                "-p",
                "127.0.0.1::8546",
                "-d",
                "--name",
                &name,
                "hermeznetwork/geth-zkevm-contracts",
            ])
            .output()
            .await
            .expect("Failed to start docker container");

        if !docker.status.success() {
            let status = docker.status;
            let err = String::from_utf8_lossy(&docker.stderr);
            panic!("Starting L1 docker container failed (status={status}): {err}");
        }

        let id = String::from_utf8(docker.stdout).unwrap().replace('\n', "");

        // Take ownership of the container immediately so that a panic in any of
        // the following setup steps still runs `DockerContainer`'s `Drop` and
        // removes the container instead of leaking it.
        let container = DockerContainer { id };

        let rpc_port = docker_published_port(&container.id, "8545/tcp").await;
        let ws_port = docker_published_port(&container.id, "8546/tcp").await;
        let ws = format!("ws://127.0.0.1:{ws_port}");
        let rpc = format!("http://127.0.0.1:{rpc_port}");

        wait_for_rpc(&rpc).await;
        wait_for_ws(&ws).await;

        Self {
            _inner: L1Instance::Docker {
                _container: container,
            },
            ws,
            rpc,
        }
    }
}

async fn docker_published_port(id: &str, container_port: &str) -> u16 {
    let output = Command::new("docker")
        .args(["port", id, container_port])
        .output()
        .await
        .unwrap_or_else(|error| panic!("Failed to inspect Docker port {container_port}: {error}"));

    if !output.status.success() {
        panic!(
            "Failed to inspect Docker port {container_port}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let binding = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .next()
        .unwrap_or_else(|| panic!("Missing published port for {container_port}"))
        .trim()
        .to_owned();

    binding
        .rsplit(':')
        .next()
        .unwrap_or_else(|| panic!("Unexpected Docker port binding: {binding}"))
        .parse()
        .unwrap_or_else(|error| panic!("Invalid Docker published port in `{binding}`: {error}"))
}

impl Drop for DockerContainer {
    fn drop(&mut self) {
        println!("Removing docker container {}", self.id);
        // Best-effort cleanup: never panic in `Drop`, because this can run while
        // unwinding from a setup panic, where a second panic would abort.
        if let Err(error) = std::process::Command::new("docker")
            .args(["rm", "-f", &self.id])
            .output()
        {
            eprintln!("Failed to remove docker container {}: {error}", self.id);
        }
    }
}

fn fixture_path() -> PathBuf {
    std::env::var_os(L1_FIXTURE_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_ANVIL_FIXTURE))
}

async fn wait_for_rpc(rpc: &str) {
    let url = reqwest::Url::parse(rpc).unwrap();

    for _ in 0..L1_READY_ATTEMPTS {
        let provider = RootProvider::<Ethereum>::new_http(url.clone());

        if provider.get_block_number().await.is_ok() {
            return;
        }

        tokio::time::sleep(L1_READY_POLL_INTERVAL).await;
    }

    panic!("L1 RPC endpoint never became ready: {rpc}");
}

/// Waits until the L1 WebSocket endpoint accepts a JSON-RPC request.
///
/// The Docker L1 backend publishes the HTTP (8545) and WebSocket (8546) ports
/// independently, so HTTP readiness does not imply the WS endpoint is up. The
/// agglayer node connects to L1 over WS at startup (`ws_node_url`) with a short
/// connect timeout, so startup is gated on WS readiness too to avoid spurious
/// boot failures when 8545 becomes reachable before 8546.
async fn wait_for_ws(ws: &str) {
    use jsonrpsee::{core::client::ClientT as _, rpc_params, ws_client::WsClientBuilder};

    for _ in 0..L1_READY_ATTEMPTS {
        if let Ok(client) = WsClientBuilder::default().build(ws).await {
            if client
                .request::<serde_json::Value, _>("eth_blockNumber", rpc_params![])
                .await
                .is_ok()
            {
                return;
            }
        }

        tokio::time::sleep(L1_READY_POLL_INTERVAL).await;
    }

    panic!("L1 WS endpoint never became ready: {ws}");
}

pub fn next_available_addr() -> std::net::SocketAddr {
    use std::net::{TcpListener, TcpStream};

    assert!(
        std::env::var("NEXTEST").is_ok(),
        "Due to concurrency issues, the rpc tests have to be run under `cargo nextest`",
    );

    let host = "127.0.0.1";
    // Request a random available port from the OS
    let listener = TcpListener::bind((host, 0)).expect("Can't bind to an available port");
    let addr = listener.local_addr().expect("Can't find an available port");

    // Create and accept a connection (which we'll promptly drop) in order to force
    // the port into the TIME_WAIT state, ensuring that the port will be
    // reserved from some limited amount of time (roughly 60s on some Linux
    // systems)
    let _sender = TcpStream::connect(addr).expect("Can't connect to an available port");
    let _incoming = listener.accept().expect("Can't accept an available port");

    addr
}
