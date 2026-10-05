use std::sync::Arc;

use alloy::{
    primitives::{Address, B256},
    providers::{mock::Asserter, ProviderBuilder},
    rpc::types::{Block, Log},
    sol_types::SolEvent,
};

use super::*;
use crate::{
    contracts::{PolygonRollupManager, PolygonZkEvmGlobalExitRootV2::UpdateL1InfoTreeV2},
    GasPriceParams,
};

const LEAF_COUNT: u32 = 42;
const EVENT_BLOCK_NUMBER: u64 = 100;
const EVENT_BLOCK_HASH: B256 = B256::repeat_byte(1);
const L1_INFO_ROOT: B256 = B256::repeat_byte(2);

fn mock_client(asserter: Asserter) -> L1RpcClient<impl Provider + Clone> {
    let rpc = ProviderBuilder::new().connect_mocked_client(asserter);
    L1RpcClient::new(
        Arc::new(rpc.clone()),
        PolygonRollupManager::new(Address::ZERO, rpc),
        Address::ZERO,
        (0, [0; 32]),
        100,
        GasPriceParams::default(),
        10_000,
    )
}

fn event_log() -> Log {
    let event = UpdateL1InfoTreeV2 {
        currentL1InfoRoot: L1_INFO_ROOT,
        leafCount: LEAF_COUNT,
        blockhash: U256::ZERO,
        minTimestamp: 0,
    };
    Log {
        inner: alloy::primitives::Log {
            address: Address::ZERO,
            data: event.encode_log_data(),
        },
        block_number: Some(EVENT_BLOCK_NUMBER),
        block_hash: Some(EVENT_BLOCK_HASH),
        ..Default::default()
    }
}

fn event_block(hash: B256) -> Block {
    let mut block: Block = Block::default();
    block.header.inner.number = EVENT_BLOCK_NUMBER;
    block.header.hash = hash;
    block
}

#[tokio::test]
async fn accepts_mined_l1_info_root_without_querying_finality() {
    let asserter = Asserter::new();
    // Only the logs and the event's block are available. An additional
    // latest/safe/finalized head query would exhaust the mock responses.
    asserter.push_success(&vec![event_log()]);
    asserter.push_success(&event_block(EVENT_BLOCK_HASH));
    let client = mock_client(asserter.clone());

    assert_eq!(
        client.get_l1_info_root(LEAF_COUNT).await.unwrap(),
        L1_INFO_ROOT.0,
    );
    assert!(asserter.read_q().is_empty());
}

#[rstest::rstest]
#[case::reorg(Some(B256::repeat_byte(3)))]
#[case::missing_block(None)]
#[tokio::test]
async fn rejects_inconsistent_event_block_without_caching(#[case] block_hash: Option<B256>) {
    let asserter = Asserter::new();
    asserter.push_success(&vec![event_log()]);
    asserter.push_success(&block_hash.map(event_block));
    let client = mock_client(asserter.clone());

    let error = client.get_l1_info_root(LEAF_COUNT).await.unwrap_err();
    match block_hash {
        Some(_) => assert!(matches!(
            error,
            L1RpcError::ReorgDetected(EVENT_BLOCK_NUMBER)
        )),
        None => assert!(matches!(
            error,
            L1RpcError::BlockHashNotFound(EVENT_BLOCK_NUMBER)
        )),
    }
    assert!(client.l1_info_roots.read().unwrap().is_empty());
    assert!(asserter.read_q().is_empty());
}

#[tokio::test]
async fn rejects_missing_l1_info_root_event() {
    let asserter = Asserter::new();
    asserter.push_success(&Vec::<Log>::new());
    let client = mock_client(asserter);

    assert!(matches!(
        client.get_l1_info_root(LEAF_COUNT).await,
        Err(L1RpcError::UpdateL1InfoTreeV2EventNotFound)
    ));
}
