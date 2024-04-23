use ethers::prelude::*;
use ethers::{
    providers,
    types::{Signature, H256, U256},
};
use jsonrpsee_test_utils::{helpers::ok_response, mocks::Id, TimeoutFutureExt as _};

use crate::{
    config::Config,
    kernel::{Kernel, ZkevmNodeVerificationError},
    signed_tx::{Proof, SignedTx, HASH_LENGTH, PROOF_LENGTH},
    zkevm_node_client::BatchByNumberResponse,
};

#[tokio::test]
async fn interop_executor_check_tx() {
    let mut config = Config::default();
    let response = BatchByNumberResponse {
        state_root: TxHash::from_slice(&[0; 32]),
        local_exit_root: TxHash::zero(),
    };
    let response = ok_response(serde_json::to_value(response).unwrap(), Id::Num(0_u64));

    let server_addr = jsonrpsee_test_utils::helpers::http_server_with_hardcoded_response(response)
        .with_default_timeout()
        .await
        .unwrap();

    let uri = format!("http://{server_addr}");
    config.full_node_rpcs.insert(1, uri.parse().unwrap());

    let (provider, _mock) = providers::Provider::mocked();

    let kernel = Kernel::new(provider, config);

    let mut signed_tx = SignedTx {
        tx: crate::signed_tx::ProofManifest {
            rollup_id: 1,
            last_verified_batch: 0.into(),
            new_verified_batch: 1.into(),
            zkp: crate::signed_tx::Zkp {
                new_state_root: H256::zero(),
                new_local_exit_root: H256::zero(),
                proof: Proof::try_from_slice(&[0; HASH_LENGTH * PROOF_LENGTH]).unwrap(),
            },
        },
        signature: Signature {
            r: U256::zero(),
            s: U256::zero(),
            v: 0,
        },
    };

    assert!(kernel.verify_proof_zkevm_node(&signed_tx).await.is_ok());

    // Assigned an unknown rollup id
    signed_tx.tx.rollup_id = 2;

    assert!(matches!(
        kernel.verify_proof_zkevm_node(&signed_tx).await,
        Err(ZkevmNodeVerificationError::InvalidRollupId(2))
    ));
}
