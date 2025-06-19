use std::{net::SocketAddr, sync::Arc};

use agglayer_config::Config;
use agglayer_storage::{
    storage::{backup::BackupClient, pending_db_cf_definitions, state_db_cf_definitions, DB},
    stores::{pending::PendingStore, state::StateStore},
    tests::TempDBDir,
};
use agglayer_types::{Certificate, CertificateId, Height, NetworkId};
use rstest::*;
use tokio_util::sync::CancellationToken;

// Mock implementations for testing
pub struct MockClient;

impl MockClient {
    pub async fn request<R>(
        &self,
        _method: &str,
        _params: jsonrpsee::core::params::ArrayParams,
    ) -> Result<R, jsonrpsee::core::ClientError>
    where
        R: serde::de::DeserializeOwned,
    {
        Err(jsonrpsee::core::ClientError::Custom("Mock client".into()))
    }
}

pub struct MockRawRpc;

impl MockRawRpc {
    pub fn into_rpc(self) -> MockJsonRpc {
        MockJsonRpc
    }
}

pub struct MockJsonRpc;

impl MockJsonRpc {
    pub async fn raw_json_request(
        &self,
        _request: &str,
        _max_response_size: u32,
    ) -> Result<(String, bool), jsonrpsee::core::BoxError> {
        Ok((
            r#"{"jsonrpc": "2.0", "result": null, "id": 0}"#.to_string(),
            true,
        ))
    }
}

pub struct RawRpcContext {
    pub rpc: MockRawRpc,
    pub config: Arc<Config>,
    pub state_store: Arc<StateStore>,
    pub pending_store: Arc<PendingStore>,
}

pub struct TestContext {
    pub cancellation_token: CancellationToken,
    pub state_store: Arc<StateStore>,
    pub pending_store: Arc<PendingStore>,
    pub client: MockClient,
    pub admin_client: MockClient,
    pub config: Arc<Config>,
    pub certificate_receiver: tokio::sync::mpsc::Receiver<(NetworkId, Height, CertificateId)>,
}

impl TestContext {
    pub fn next_available_address() -> SocketAddr {
        next_available_addr()
    }

    async fn new() -> Self {
        let config = Self::get_default_config();
        Self::new_with_config(config).await
    }

    pub async fn new_with_config(config: Config) -> Self {
        let cancellation_token = CancellationToken::new();
        let config = Arc::new(config);

        let state_db = Arc::new(
            DB::open_cf(&config.storage.state_db_path, state_db_cf_definitions()).unwrap(),
        );
        let pending_db = Arc::new(
            DB::open_cf(&config.storage.pending_db_path, pending_db_cf_definitions()).unwrap(),
        );

        let state_store = Arc::new(StateStore::new(state_db, BackupClient::noop()));
        let pending_store = Arc::new(PendingStore::new(pending_db));

        let (_sender, certificate_receiver) = tokio::sync::mpsc::channel(1);

        Self {
            cancellation_token,
            state_store,
            pending_store,
            client: MockClient,
            admin_client: MockClient,
            config,
            certificate_receiver,
        }
    }

    pub fn get_default_config() -> Config {
        let tmp = TempDBDir::new();
        let mut cfg = Config::new(&tmp.path);
        for network_id in 0..10 {
            cfg.proof_signers.insert(
                network_id,
                Certificate::wallet_for_test(NetworkId::new(network_id)).address(),
            );
        }
        cfg
    }

    pub async fn new_raw_rpc() -> RawRpcContext {
        let config = Self::get_default_config();
        Self::new_raw_rpc_with_config(config).await
    }

    pub async fn new_raw_rpc_with_config(config: Config) -> RawRpcContext {
        let config = Arc::new(config);

        let state_db = Arc::new(
            DB::open_cf(&config.storage.state_db_path, state_db_cf_definitions()).unwrap(),
        );
        let pending_db = Arc::new(
            DB::open_cf(&config.storage.pending_db_path, pending_db_cf_definitions()).unwrap(),
        );

        let state_store = Arc::new(StateStore::new(state_db, BackupClient::noop()));
        let pending_store = Arc::new(PendingStore::new(pending_db));

        RawRpcContext {
            rpc: MockRawRpc,
            config,
            state_store,
            pending_store,
        }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}

#[fixture]
pub async fn context() -> TestContext {
    TestContext::new().await
}

#[fixture]
pub async fn raw_rpc() -> RawRpcContext {
    TestContext::new_raw_rpc().await
}

pub fn next_available_addr() -> std::net::SocketAddr {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
}
