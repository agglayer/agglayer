use std::time::Duration;

use tokio::process::Command;

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
                &format!("{}:8545", rpc_port),
                "-p",
                &format!("{}:8546", ws_port),
                "-d",
                "--name",
                &name,
                "hermeznetwork/geth-zkevm-contracts",
            ])
            .output()
            .await
            .expect("Failed to start docker container");

        let id = String::from_utf8(docker.stdout).unwrap().replace('\n', "");
        let ws = format!("ws://127.0.0.1:{}", ws_port);
        let rpc = format!("http://127.0.0.1:{}", rpc_port);

        // Add delay to ensure the container is ready
        tokio::time::sleep(Duration::from_secs(5)).await;
        Self { id, ws, rpc }
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
