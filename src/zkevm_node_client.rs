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
    ) -> Result<BatchByNumberResponse, Error> {
        self.client
            .request(
                "zkevm_getBatchByNumber",
                rpc_params![format!("0x{:x}", batch_number), false],
            )
            .await
    }
}
