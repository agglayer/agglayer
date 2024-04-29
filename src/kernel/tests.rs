use ethers::core::utils;
use ethers::prelude::*;
use ethers::signers::LocalWallet;
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::{
    abi::AbiEncode,
    providers,
    types::{Signature, H256, U256},
};
use jsonrpsee_test_utils::{helpers::ok_response, mocks::Id, TimeoutFutureExt as _};

use crate::config::L1;
use crate::contracts::polygon_rollup_manager::{
    RollupIDToRollupDataCall, RollupIDToRollupDataReturn, VerifyBatchesTrustedAggregatorCall,
};
use crate::contracts::polygon_zk_evm::{TrustedSequencerCall, TrustedSequencerReturn};
use crate::{
    config::Config,
    kernel::{Kernel, ZkevmNodeVerificationError},
    signed_tx::{Proof, SignedTx, HASH_LENGTH, PROOF_LENGTH},
    zkevm_node_client::BatchByNumberResponse,
};

macro_rules! push_response {
    ($m:ident, to_hex: $response:expr) => {
        push_response!($m, $response.encode_hex());
    };
    ($m:ident, $response:expr) => {
        $m.push_response(MockResponse::Value(serde_json::Value::String($response)));
    };
}

macro_rules! transaction_request {
    (to: $to:expr, data: $data:expr) => {
        utils::serialize(&TypedTransaction::Eip1559(
            Eip1559TransactionRequest::new()
                .to($to)
                .data($data.encode()),
        ))
    };
}

/// Test to check if the rollup_id is registered
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

    let mut signed_tx = signed_tx();

    assert!(kernel.check_rollup_registered(signed_tx.tx.rollup_id));
    assert!(kernel
        .get_zkevm_node_client_for_rollup(signed_tx.tx.rollup_id)
        .is_ok());

    // Assigned an unknown rollup id
    signed_tx.tx.rollup_id = 2;

    assert!(!kernel.check_rollup_registered(signed_tx.tx.rollup_id));
    assert!(matches!(
        kernel.get_zkevm_node_client_for_rollup(signed_tx.tx.rollup_id),
        Err(ZkevmNodeVerificationError::InvalidRollupId(2))
    ));
}

/// Test the verify_zkp method
#[tokio::test]
async fn interop_executor_verify_zkp() {
    let config = Config::default();

    let (provider, mock) = providers::Provider::mocked();

    let l1 = config.l1.clone();
    let kernel = Kernel::new(provider, config);

    let signed_tx = signed_tx();

    let response = rollup_data(&l1).encode_hex();

    mock.push_response(MockResponse::Value(
        serde_json::Value::String(String::new()),
    ));

    let sequencer_address = Address::random();

    push_response!(mock, to_hex: TrustedSequencerReturn(sequencer_address));
    push_response!(mock, response);

    assert!(kernel.verify_proof_eth_call(&signed_tx).await.is_ok());

    let tx_rollup_data = transaction_request!(
        to: l1.rollup_manager_contract,
        data: RollupIDToRollupDataCall { rollup_id: 1 }
    );

    let tx_trusted_sequencer =
        transaction_request!(to: l1.rollup_manager_contract, data: TrustedSequencerCall {});

    let tx_verify_batch = transaction_request!(
        to: l1.rollup_manager_contract,
        data: VerifyBatchesTrustedAggregatorCall {
            rollup_id: 1,
            pending_state_num: 0,
            init_num_batch: signed_tx.tx.last_verified_batch.as_u64(),
            final_new_batch: signed_tx.tx.new_verified_batch.as_u64(),
            new_local_exit_root: signed_tx.tx.zkp.new_local_exit_root.to_fixed_bytes(),
            new_state_root: signed_tx.tx.zkp.new_state_root.to_fixed_bytes(),
            beneficiary: sequencer_address,
            proof: signed_tx.tx.zkp.proof.to_fixed_bytes(),
        }
    );

    let block = utils::serialize(&(BlockNumber::Latest));

    // Check if the calls are made
    mock.assert_request("eth_call", [tx_rollup_data, block.clone()])
        .unwrap();
    mock.assert_request("eth_call", [tx_trusted_sequencer, block.clone()])
        .unwrap();
    mock.assert_request("eth_call", [tx_verify_batch, block])
        .unwrap();
}

/// Test that check if the verify_signature method
#[tokio::test]
async fn interop_executor_verify_signature() {
    let config = Config::default();

    let (provider, mock) = providers::Provider::mocked();

    let l1 = config.l1.clone();
    let kernel = Kernel::new(provider, config);

    let sequencer_wallet = LocalWallet::new(&mut rand::thread_rng());
    let sequencer_address = sequencer_wallet.address();

    let mut signed_tx = signed_tx();

    let response = rollup_data(&l1).encode_hex();

    signed_tx.sign(&sequencer_wallet).unwrap();

    // valid signature with valid sequencer_address
    {
        push_response!(mock, to_hex: TrustedSequencerReturn(sequencer_address));
        push_response!(mock, response.clone());

        assert!(kernel.verify_signature(&signed_tx).await.is_ok());
    }

    // Wrong signature with different sequencer_address
    {
        push_response!(mock, to_hex: TrustedSequencerReturn(H160::zero()));
        push_response!(mock, response);

        assert!(matches!(
            kernel.verify_signature(&signed_tx).await,
            Err(crate::kernel::SignatureVerificationError::InvalidSigner { signer, trusted_sequencer })
            if signer == sequencer_address && trusted_sequencer == H160::zero()
        ));
    }

    // Correct signature with configured proof signer for rollup
    {
        // TODO: to be implemented
    }

    // Wrong signature with configured proof signer for rollup
    {
        // TODO: to be implemented
    }

    let tx_rollup_data = transaction_request!(
        to: l1.rollup_manager_contract,
        data: RollupIDToRollupDataCall { rollup_id: 1 }
    );

    let tx_trusted_sequencer = transaction_request!(
        to: l1.rollup_manager_contract,
        data: TrustedSequencerCall {}
    );

    let block = utils::serialize(&(BlockNumber::Latest));

    // Check if the calls are made
    mock.assert_request("eth_call", [tx_rollup_data, block.clone()])
        .unwrap();
    mock.assert_request("eth_call", [tx_trusted_sequencer, block.clone()])
        .unwrap();
}

mod interop_executor_execute {
    use super::*;

    #[tokio::test]
    async fn batch_not_nil_root_match() {
        let mut config = Config::default();
        let sequencer_wallet = LocalWallet::new(&mut rand::thread_rng());
        let mut signed_tx = signed_tx();
        let _ = signed_tx.sign(&sequencer_wallet);

        let response = BatchByNumberResponse {
            state_root: signed_tx.tx.zkp.new_state_root,
            local_exit_root: signed_tx.tx.zkp.new_local_exit_root,
        };
        let response = ok_response(serde_json::to_value(response).unwrap(), Id::Num(0_u64));

        let server_addr =
            jsonrpsee_test_utils::helpers::http_server_with_hardcoded_response(response)
                .with_default_timeout()
                .await
                .unwrap();

        let uri = format!("http://{server_addr}");
        config.full_node_rpcs.insert(1, uri.parse().unwrap());

        let (provider, _mock) = providers::Provider::mocked();

        let kernel = Kernel::new(provider, config);

        assert!(kernel.verify_proof_zkevm_node(&signed_tx).await.is_ok());
    }

    #[tokio::test]
    async fn return_error_when_response_is_null() {
        let mut config = Config::default();
        let sequencer_wallet = LocalWallet::new(&mut rand::thread_rng());
        let mut signed_tx = signed_tx();
        let _ = signed_tx.sign(&sequencer_wallet);

        let response = ok_response(serde_json::Value::Null, Id::Num(0_u64));

        let server_addr =
            jsonrpsee_test_utils::helpers::http_server_with_hardcoded_response(response)
                .with_default_timeout()
                .await
                .unwrap();

        let uri = format!("http://{server_addr}");
        config.full_node_rpcs.insert(1, uri.parse().unwrap());

        let (provider, _mock) = providers::Provider::mocked();

        let kernel = Kernel::new(provider, config);

        assert!(matches!(
            kernel.verify_proof_zkevm_node(&signed_tx).await,
            Err(ZkevmNodeVerificationError::RpcError(_))
        ));
    }

    #[tokio::test]
    async fn return_error_when_state_root_differ() {
        let mut config = Config::default();
        let sequencer_wallet = LocalWallet::new(&mut rand::thread_rng());
        let mut signed_tx = signed_tx();
        let _ = signed_tx.sign(&sequencer_wallet);

        let response = BatchByNumberResponse {
            state_root: H256::zero(),
            local_exit_root: signed_tx.tx.zkp.new_local_exit_root,
        };
        let response = ok_response(serde_json::to_value(response).unwrap(), Id::Num(0_u64));

        let server_addr =
            jsonrpsee_test_utils::helpers::http_server_with_hardcoded_response(response)
                .with_default_timeout()
                .await
                .unwrap();

        let uri = format!("http://{server_addr}");
        config.full_node_rpcs.insert(1, uri.parse().unwrap());

        let (provider, _mock) = providers::Provider::mocked();

        let kernel = Kernel::new(provider, config);

        assert!(matches!(
            kernel.verify_proof_zkevm_node(&signed_tx).await,
            Err(ZkevmNodeVerificationError::InvalidStateRoot { expected, got })
            if expected == signed_tx.tx.zkp.new_state_root && got == H256::zero()
        ));
    }

    #[tokio::test]
    async fn return_error_when_exit_root_differ() {
        let mut config = Config::default();
        let sequencer_wallet = LocalWallet::new(&mut rand::thread_rng());
        let mut signed_tx = signed_tx();
        let _ = signed_tx.sign(&sequencer_wallet);

        let response = BatchByNumberResponse {
            state_root: signed_tx.tx.zkp.new_state_root,
            local_exit_root: H256::zero(),
        };
        let response = ok_response(serde_json::to_value(response).unwrap(), Id::Num(0_u64));

        let server_addr =
            jsonrpsee_test_utils::helpers::http_server_with_hardcoded_response(response)
                .with_default_timeout()
                .await
                .unwrap();

        let uri = format!("http://{server_addr}");
        config.full_node_rpcs.insert(1, uri.parse().unwrap());

        let (provider, _mock) = providers::Provider::mocked();

        let kernel = Kernel::new(provider, config);

        assert!(matches!(
            kernel.verify_proof_zkevm_node(&signed_tx).await,
            Err(ZkevmNodeVerificationError::InvalidExitRoot { expected, got })
            if expected == signed_tx.tx.zkp.new_local_exit_root && got == H256::zero()
        ));
    }
}

fn signed_tx() -> SignedTx {
    SignedTx {
        tx: crate::signed_tx::ProofManifest {
            rollup_id: 1,
            last_verified_batch: 0.into(),
            new_verified_batch: 1.into(),
            zkp: crate::signed_tx::Zkp {
                new_state_root: H256::random(),
                new_local_exit_root: H256::random(),
                proof: Proof::try_from_slice(&[0; HASH_LENGTH * PROOF_LENGTH]).unwrap(),
            },
        },
        signature: Signature {
            r: U256::zero(),
            s: U256::zero(),
            v: 0,
        },
    }
}

fn rollup_data(l1: &L1) -> RollupIDToRollupDataReturn {
    RollupIDToRollupDataReturn {
        chain_id: 1,
        rollup_contract: l1.rollup_manager_contract,
        verifier: H160::random(),
        fork_id: 0,
        last_local_exit_root: [0; 32],
        last_batch_sequenced: 0,
        last_verified_batch: 0,
        last_pending_state: 0,
        last_pending_state_consolidated: 0,
        last_verified_batch_before_upgrade: 0,
        rollup_type_id: 1,
        rollup_compatibility_id: 0,
    }
}
