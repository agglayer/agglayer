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
    concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/anvil-l1/state.hex");

enum L1Instance {
    Docker { id: String },
    Anvil { _instance: AnvilInstance },
}

pub struct L1Docker {
    inner: L1Instance,
    pub ws: String,
    pub rpc: String,
}

impl L1Docker {
    pub async fn new(name: String) -> Self {
        match std::env::var(L1_BACKEND_ENV) {
            Ok(backend) if backend == ANVIL_BACKEND => Self::new_anvil().await,
            Ok(backend) if backend == DOCKER_BACKEND => Self::new_docker(name).await,
            Ok(backend) => panic!(
                "Unsupported L1 backend `{backend}`. Supported values: `{ANVIL_BACKEND}`, `{DOCKER_BACKEND}`, or unset"
            ),
            Err(std::env::VarError::NotPresent) => Self::new_anvil().await,
            Err(error) => panic!("Failed to read {L1_BACKEND_ENV}: {error}"),
        }
    }

    async fn new_anvil() -> Self {
        let port = next_available_addr().port();
        let anvil = Anvil::new()
            .port(port)
            .chain_id(1337u64)
            .block_time(1u64)
            .arg("--auto-impersonate")
            .spawn();

        let rpc = anvil.endpoint_url().to_string();
        wait_for_rpc(&rpc).await;
        load_anvil_fixture(&rpc).await;

        Self {
            inner: L1Instance::Anvil { _instance: anvil },
            ws: format!("ws://127.0.0.1:{port}"),
            rpc,
        }
    }

    async fn new_docker(name: String) -> Self {
        let ws_port = next_available_addr().port();
        let rpc_port = next_available_addr().port();

        let docker = Command::new("docker")
            .args([
                "run",
                "-p",
                &format!("{rpc_port}:8545"),
                "-p",
                &format!("{ws_port}:8546"),
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
        let ws = format!("ws://127.0.0.1:{ws_port}");
        let rpc = format!("http://127.0.0.1:{rpc_port}");

        wait_for_rpc(&rpc).await;

        Self {
            inner: L1Instance::Docker { id },
            ws,
            rpc,
        }
    }
}

impl Drop for L1Docker {
    fn drop(&mut self) {
        if let L1Instance::Docker { id } = &self.inner {
            println!("Removing docker container {id}");
            std::process::Command::new("docker")
                .args(["rm", "-f", id])
                .output()
                .expect("Failed to remove docker container");
        }
    }
}

fn fixture_path() -> PathBuf {
    std::env::var_os(L1_FIXTURE_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_ANVIL_FIXTURE))
}

async fn load_anvil_fixture(rpc: &str) {
    let fixture_path = fixture_path();
    let fixture = std::fs::read_to_string(&fixture_path).unwrap_or_else(|error| {
        panic!(
            "Failed to read Anvil fixture {}: {error}",
            fixture_path.display()
        )
    });
    let fixture = fixture.trim().trim_matches('"');

    let client = reqwest::Client::new();
    let response = client
        .post(rpc)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "anvil_loadState",
            "params": [fixture],
        }))
        .send()
        .await
        .unwrap_or_else(|error| panic!("Failed to load Anvil fixture over {rpc}: {error}"));
    let status = response.status();
    let payload: serde_json::Value = response
        .json()
        .await
        .unwrap_or_else(|error| panic!("Failed to decode Anvil fixture load response: {error}"));

    if !status.is_success() {
        panic!("Anvil fixture load failed with status {status}: {payload}");
    }

    if let Some(error) = payload.get("error") {
        panic!("Anvil fixture load returned RPC error: {error}");
    }

    wait_for_rpc(rpc).await;
}

async fn wait_for_rpc(rpc: &str) {
    let url = reqwest::Url::parse(rpc).unwrap();

    for _ in 0..60 {
        let provider = RootProvider::<Ethereum>::new_http(url.clone());

        if provider.get_block_number().await.is_ok() {
            return;
        }

        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }

    panic!("L1 RPC endpoint never became ready: {rpc}");
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
