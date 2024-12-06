use pessimistic_proof::{
    bridge_exit::BridgeExit,
    local_exit_tree::{hasher::Keccak256Hasher, LocalExitTree},
};
use pessimistic_proof_test_suite::event_data::{load_json_data_file, BridgeEvent, EventData};

const JSON_FILE_NAME: &str = "bridge_events_10k.json";

#[test]
fn test_local_exit_root() {
    let mut local_exit_tree: LocalExitTree<Keccak256Hasher> = LocalExitTree::new();

    let bridge_events: Vec<BridgeEvent> = read_sorted_bridge_events();

    let mut deposit_count: u32 = 0;
    for event in bridge_events {
        match event.event_data {
            EventData::UpdateL1InfoTree {
                mainnet_exit_root,
                rollup_exit_root: _,
            } => {
                let computed_root = local_exit_tree.get_root();

                assert_eq!(computed_root, mainnet_exit_root.into());
            }
            EventData::Deposit(deposit_event_data) => {
                assert_eq!(deposit_event_data.deposit_count, deposit_count);
                deposit_count += 1;

                let bridge_exit: BridgeExit = deposit_event_data.into();
                local_exit_tree.add_leaf(bridge_exit.hash()).unwrap();
            }
            EventData::Claim(_) => {
                // do nothing
            }
        }
    }
}

/// Reads the bridge events from disk,
/// and sorts by (block number, tx index, log index).
fn read_sorted_bridge_events() -> Vec<BridgeEvent> {
    let mut bridge_events: Vec<BridgeEvent> = load_json_data_file(JSON_FILE_NAME);
    bridge_events.sort_unstable_by(|a, b| {
        use std::cmp::Ordering;
        match a.block_number.cmp(&b.block_number) {
            Ordering::Equal => match a.transaction_index.cmp(&b.transaction_index) {
                Ordering::Equal => a.log_index.cmp(&b.log_index),
                not_eq => not_eq,
            },
            not_eq => not_eq,
        }
    });

    bridge_events
}

#[test]
fn bridge_events_loading_works() {
    let _ = read_sorted_bridge_events();
}
