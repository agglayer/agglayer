//! The ZkEVM node JSON RPC client.
use ethers::types::H256;
use jsonrpsee::{
    core::client::{error::Error, ClientT},
    rpc_params,
};
use serde::{Deserialize, Serialize};

/// The ZkEVM node JSON RPC client.
///
/// This client provides functionality for interacting with the ZkEVM node.
/// The ZkEVM node JSON RPC methods are defined [here](https://github.com/0xPolygonHermez/zkevm-node/blob/aae30e9c79bdf363814e7fe2a3df9b34e855c998/jsonrpc/endpoints_zkevm.openrpc.json).
pub(crate) struct ZkevmNodeClient<C> {
    client: C,
}

impl<C> ZkevmNodeClient<C> {
    /// Create a new instance of the ZkEVM node client.
    pub(crate) fn new(client: C) -> Self {
        Self { client }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BatchByNumberResponse {
    pub(crate) state_root: H256,
    pub(crate) local_exit_root: H256,
}

impl<C> ZkevmNodeClient<C>
where
    C: ClientT,
{
    pub(crate) async fn batch_by_number(
        &self,
        batch_number: u64,
    ) -> Result<Option<BatchByNumberResponse>, Error> {
        self.client
            .request(
                "zkevm_getBatchByNumber",
                rpc_params![format!("0x{:x}", batch_number), false],
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use jsonrpsee::{
        http_client::HttpClient,
        server::ServerHandle,
        types::{ErrorCode, ErrorObjectOwned},
        RpcModule,
    };
    use serde_json::{json, Value};

    use super::ZkevmNodeClient;

    fn sample_response(batch_no: u64) -> Value {
        let batch_no = format!("{batch_no:#x}");
        json! {{
            "closed": true,
            "coinbase": "0x148ee7daf16574cd020afa34cc658f8f3fbd2800",
            "globalExitRoot": "0x651346a8d67b1cf6ad10d65281d94cfa2472eb01d4f99ae0642e9f895fc5e31f",
            "localExitRoot": "0x7b30fb9a5836fe652820b07f5bfbdae484b63c7fc3168df2327844f084409e89",
            "number": batch_no,
            "rollupExitRoot": "0x0000000000000000000000000000000000000000000000000000000000000000",
            "stateRoot": "0x988c01c93fac76f6e819419a6ebed65511481b8fee2c7c56422a60f4712bb10d",
            "timestamp": "0x643c41ac",
        }}
    }

    async fn start_server_and_client() -> (ServerHandle, ZkevmNodeClient<HttpClient>) {
        let server = jsonrpsee::server::Server::builder()
            .build("127.0.0.1:0")
            .await
            .unwrap();
        let addr = server.local_addr().unwrap();

        let mut module = RpcModule::new(());
        let _ = module
            .register_method(
                "zkevm_getBatchByNumber",
                |params, _, _| -> Result<Value, ErrorObjectOwned> {
                    let arg: String = params
                        .sequence()
                        .next()
                        .map_err(|_| ErrorCode::ServerError(1))?;
                    let batch_no = arg.strip_prefix("0x").ok_or(ErrorCode::ServerError(2))?;
                    let batch_no =
                        u64::from_str_radix(batch_no, 16).map_err(|_| ErrorCode::ServerError(3))?;
                    if batch_no < 0x2000 {
                        Ok(sample_response(batch_no))
                    } else {
                        Ok(json!(null))
                    }
                },
            )
            .unwrap();

        let server = server.start(module);

        let client = HttpClient::builder()
            .build(format!("http://{addr}"))
            .unwrap();
        let client = super::ZkevmNodeClient::new(client);

        (server, client)
    }

    #[rstest::rstest]
    #[case::success(0x1337, true)]
    #[case::null(0x2337, false)]
    #[tokio::test]
    async fn response(#[case] batch_no: u64, #[case] expected_some: bool) {
        let (server, client) = start_server_and_client().await;

        let res = client.batch_by_number(batch_no).await.expect("an Ok(_)");
        assert_eq!(res.is_some(), expected_some);

        let _ = server.stop();
        server.stopped().await;
    }
}
