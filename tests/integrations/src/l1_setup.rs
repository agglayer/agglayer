use std::{str::FromStr, time::Duration};

use agglayer_contracts::contracts::{AggchainBase, PolygonRollupManager};
use alloy::{
    network::EthereumWallet,
    primitives::U256,
    providers::ProviderBuilder,
    signers::{local::PrivateKeySigner, Signer},
};
use tokio::process::Command;

/// Rollup ID for integration tests on the L1 Docker image.
///
/// Rollup 3 is deployed with `CONSENSUS_TYPE = 1` (multisig) and supports
/// `getAggchainHash` for multisig-only certificates. Rollup 2 is legacy ECDSA;
/// rollup 1 is AggchainFEP and reverts on `getAggchainHash`.
pub const INTEGRATION_ROLLUP_ID: u32 = 3;

const L1_DOCKER_CHAIN_ID: u64 = 1337;
const ROLLUP_MANAGER_ADDRESS: &str = "0x0B306BF915C4d645ff596e518fAf3F9669b97016";
/// Hardhat/Anvil account #0 — admin on rollups in the L1 Docker image.
const L1_DEPLOYER_PRIVATE_KEY: &str =
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

pub struct L1Docker {
    id: String,
    pub ws: String,
    pub rpc: String,
}

impl L1Docker {
    pub async fn new(name: String) -> Self {
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

        // Add delay to ensure the container is ready
        tokio::time::sleep(Duration::from_secs(5)).await;

        let l1 = Self { id, ws, rpc };
        configure_integration_multisig(&l1).await;
        l1
    }
}

impl Drop for L1Docker {
    fn drop(&mut self) {
        println!("Removing docker container {}", self.id);
        std::process::Command::new("docker")
            .args(["rm", "-f", &self.id])
            .output()
            .expect("Failed to remove docker container");
    }
}

/// Configure multisig 1-of-1 on the integration rollup using the L1 deployer key.
async fn configure_integration_multisig(l1: &L1Docker) {
    let deployer = PrivateKeySigner::from_str(L1_DEPLOYER_PRIVATE_KEY)
        .expect("valid deployer private key")
        .with_chain_id(Some(L1_DOCKER_CHAIN_ID));
    let signer_address = deployer.address();

    let provider = ProviderBuilder::new()
        .wallet(EthereumWallet::from(deployer))
        .connect_http(
            l1.rpc
                .parse()
                .expect("valid l1 rpc url"),
        );

    let rollup_manager = PolygonRollupManager::new(
        ROLLUP_MANAGER_ADDRESS
            .parse()
            .expect("valid rollup manager address"),
        provider.clone(),
    );

    let rollup_data = rollup_manager
        .rollupIDToRollupData(INTEGRATION_ROLLUP_ID)
        .call()
        .await
        .expect("rollup data for integration network");

    let aggchain = AggchainBase::new(rollup_data.rollupContract, provider);

    aggchain
        .updateSignersAndThreshold(
            vec![],
            vec![(signer_address, "http://localhost".to_string()).into()],
            U256::from(1),
        )
        .send()
        .await
        .expect("send updateSignersAndThreshold")
        .get_receipt()
        .await
        .expect("updateSignersAndThreshold receipt");
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
