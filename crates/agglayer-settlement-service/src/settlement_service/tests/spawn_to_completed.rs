use std::{sync::Arc, time::SystemTime};

use agglayer_types::{
    ContractCallOutcome, ContractCallResult, Digest, Nonce, SettlementAttempt,
    SettlementAttemptNumber, SettlementAttemptResult, SettlementJobResult, SettlementTxHash, B256,
};
use alloy::{
    consensus::{Signed, TxEip1559, TxEnvelope},
    network::EthereumWallet,
    primitives::{Address, Signature, TxKind, U256},
    providers::{mock::Asserter, ProviderBuilder},
    signers::local::PrivateKeySigner,
};

use super::*;
use crate::settlement_task::{SettlementTask, StoredSettlementJob, TaskControlHandle};

fn test_signer() -> PrivateKeySigner {
    PrivateKeySigner::from_slice(&[0x11; 32]).expect("valid test signing key")
}

fn mk_tx_hash(seed: u8) -> SettlementTxHash {
    SettlementTxHash::new(Digest::from([seed; 32]))
}

fn mk_tx(hash_seed: u8) -> TxEnvelope {
    TxEnvelope::Eip1559(Signed::new_unchecked(
        TxEip1559 {
            chain_id: 1,
            nonce: 2,
            gas_limit: 100_000,
            max_fee_per_gas: 100,
            max_priority_fee_per_gas: 10,
            to: TxKind::Call(Address::from([6; 20])),
            value: U256::from(7_u64),
            input: vec![8].into(),
            access_list: Default::default(),
        },
        Signature::test_signature(),
        B256::from([hash_seed; 32]),
    ))
}

fn mk_rpc_block(number: u64, hash: B256) -> alloy::rpc::types::Block {
    let mut block: alloy::rpc::types::Block = Default::default();
    block.header.hash = hash;
    block.header.inner.number = number;
    block
}

fn mk_rpc_receipt(
    tx_hash: SettlementTxHash,
    block_hash: B256,
    block_number: u64,
) -> alloy::rpc::types::TransactionReceipt {
    alloy::rpc::types::TransactionReceipt {
        inner: alloy::consensus::ReceiptEnvelope::Eip1559(alloy::consensus::ReceiptWithBloom {
            receipt: alloy::consensus::Receipt {
                status: true.into(),
                cumulative_gas_used: 0,
                logs: vec![],
            },
            logs_bloom: Default::default(),
        }),
        transaction_hash: tx_hash.into(),
        transaction_index: Some(0),
        block_hash: Some(block_hash),
        block_number: Some(block_number),
        gas_used: 0,
        effective_gas_price: 0,
        blob_gas_used: None,
        blob_gas_price: None,
        from: Address::from([9; 20]),
        to: None,
        contract_address: None,
    }
}

fn mk_rpc_transaction(
    tx: TxEnvelope,
    from: Address,
    block_number: u64,
) -> alloy::rpc::types::Transaction {
    alloy::rpc::types::Transaction {
        inner: alloy::consensus::transaction::Recovered::new_unchecked(tx, from),
        block_hash: Some(B256::from([2; 32])),
        block_number: Some(block_number),
        transaction_index: Some(0),
        effective_gas_price: Some(0),
    }
}

fn mk_completion_provider(
    wallet: Address,
    block_hash: B256,
    block_number: u64,
    stored_result: &ContractCallResult,
) -> impl Provider + WalletProvider + 'static {
    let asserter = Asserter::new();
    // The other wallet's nonce is scanned first and is not on L1.
    asserter.push_failure(alloy::rpc::json_rpc::ErrorPayload {
        code: -32001,
        message: "not found".into(),
        data: None,
    });
    // The winning nonce replays settlement checks against L1.
    asserter.push_success(&mk_rpc_transaction(mk_tx(60), wallet, block_number));
    asserter.push_success(&mk_rpc_receipt(
        stored_result.tx_hash,
        block_hash,
        block_number,
    ));
    asserter.push_success(&mk_rpc_block(1_000, B256::from([1; 32])));
    asserter.push_success(&mk_rpc_receipt(
        stored_result.tx_hash,
        block_hash,
        block_number,
    ));
    asserter.push_success(&mk_rpc_block(block_number, block_hash));
    ProviderBuilder::new()
        .wallet(EthereumWallet::from(test_signer()))
        .connect_mocked_client(asserter)
}

/// Regression test for issue 1480: a spawned task that reaches `Completed`
/// must drop both in-memory registrations once the run loop exits.
#[tokio::test]
async fn spawn_clears_result_watchers_after_task_completes() {
    let job_id = mk_job_id(14);
    let job = mk_job(14);
    let wallet = Address::from([4; 20]);
    let other_wallet = Address::from([3; 20]);
    let nonce = Nonce(11);
    let other_nonce = Nonce(12);
    let block_hash = B256::from([7; 32]);
    let block_number = 10;
    let stored_result = ContractCallResult {
        outcome: ContractCallOutcome::Success,
        metadata: Default::default(),
        block_hash,
        block_number,
        tx_hash: mk_tx_hash(60),
    };
    let winning_attempt = SettlementAttempt {
        sender_wallet: wallet.into(),
        nonce,
        hash: stored_result.tx_hash,
        submission_time: SystemTime::UNIX_EPOCH,
        max_fee_per_gas: 0,
        max_priority_fee_per_gas: 0,
    };
    let sibling_attempt = SettlementAttempt {
        sender_wallet: wallet.into(),
        nonce,
        hash: mk_tx_hash(70),
        submission_time: SystemTime::UNIX_EPOCH,
        max_fee_per_gas: 0,
        max_priority_fee_per_gas: 0,
    };
    let other_attempt = SettlementAttempt {
        sender_wallet: other_wallet.into(),
        nonce: other_nonce,
        hash: mk_tx_hash(80),
        submission_time: SystemTime::UNIX_EPOCH,
        max_fee_per_gas: 0,
        max_priority_fee_per_gas: 0,
    };
    let attempt_result = SettlementAttemptResult::ContractCall(stored_result.clone());

    let mut store = MockStateStore::new();
    expect_empty_startup_recovery(&mut store);
    store
        .expect_get_settlement_job()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(move |_| Ok(Some(job)));
    store
        .expect_get_settlement_job_result()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(|_| Ok(None));
    store
        .expect_list_settlement_attempt_results()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once({
            let attempt_result = attempt_result.clone();
            move |_| Ok(vec![(1, attempt_result.clone())])
        });
    store
        .expect_list_settlement_attempts()
        .once()
        .withf(move |requested_job_id| requested_job_id == &job_id)
        .return_once(move |_| {
            Ok(vec![
                (1, winning_attempt.clone()),
                (2, sibling_attempt.clone()),
                (3, other_attempt.clone()),
            ])
        });
    store
        .expect_record_settlement_attempt_result()
        .times(2)
        .returning(|_, _, _| Ok(()));
    store
        .expect_insert_settlement_job_result()
        .once()
        .returning(|_, _| Ok(()));

    let store = Arc::new(store);
    let provider = mk_completion_provider(wallet, block_hash, block_number, &stored_result);
    let service = SettlementService::start(
        SettlementServiceConfig::default(),
        Arc::new(SettlementTransactionConfig::default()),
        Arc::new(provider),
        store.clone(),
        CancellationToken::new(),
    )
    .await
    .expect("settlement service should start")
    .0;

    let (task_control_handle, task_control) = TaskControlHandle::new(&service.cancellation_token);
    let task = match SettlementTask::load(
        job_id,
        service.tx_config.clone(),
        service.provider.clone(),
        service.store.clone(),
        service.wallet_nonce_locks.clone(),
        task_control,
    )
    .await
    .expect("settlement task should load")
    {
        StoredSettlementJob::Pending(task) => task,
        StoredSettlementJob::Completed(_) => panic!("initial load should be pending"),
    };

    assert!(service
        .result_watchers
        .lock()
        .expect("settlement result_watchers lock poisoned")
        .is_empty());
    let mut result_receiver = service
        .spawn_settlement_task(job_id, task, task_control_handle)
        .await;
    assert!(service
        .result_watchers
        .lock()
        .expect("settlement result_watchers lock poisoned")
        .contains_key(&job_id));

    result_receiver
        .changed()
        .await
        .expect("spawned task should publish a terminal result");

    wait_until(|| {
        service
            .result_watchers
            .lock()
            .expect("settlement result_watchers lock poisoned")
            .is_empty()
    })
    .await;
    assert!(service
        .task_controls
        .lock()
        .expect("settlement task_controls lock poisoned")
        .is_empty());
    assert_eq!(
        result_receiver.borrow().as_ref(),
        Some(&SettlementJobResult {
            wallet: wallet.into(),
            nonce,
            attempt_number: SettlementAttemptNumber(1),
            contract_call_result: stored_result,
        })
    );
}
