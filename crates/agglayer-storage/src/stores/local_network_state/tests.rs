use std::sync::Arc;

use agglayer_types::{Hash, LocalNetworkStateData, NetworkId};
use pessimistic_proof::keccak::Digest;
use rstest::{fixture, rstest};

use crate::{
    storage::{local_network_state_db_cf_definitions, DB},
    stores::{
        interfaces::{reader::LocalNetworkStateReader, writer::LocalNetworkStateWriter},
        local_network_state::LocalNetworkStateStore,
    },
    tests::TempDBDir,
};

fn equal_state(lhs: &LocalNetworkStateData, rhs: &LocalNetworkStateData) -> bool {
    // local exit tree
    assert_eq!(lhs.exit_tree.get_root(), rhs.exit_tree.get_root());

    // balance tree
    assert_eq!(lhs.balance_tree.root, rhs.balance_tree.root);
    assert_eq!(lhs.balance_tree.tree, rhs.balance_tree.tree);

    // nullifier tree
    assert_eq!(lhs.nullifier_tree.root, rhs.nullifier_tree.root);
    assert_eq!(lhs.nullifier_tree.tree, rhs.nullifier_tree.tree);

    true
}

#[fixture]
fn network_id() -> NetworkId {
    0.into()
}

#[fixture]
fn store() -> LocalNetworkStateStore {
    let tmp = TempDBDir::new();
    let db =
        Arc::new(DB::open_cf(tmp.path.as_path(), local_network_state_db_cf_definitions()).unwrap());

    LocalNetworkStateStore::new(db.clone())
}

#[rstest]
fn can_handle_empty_state(
    #[from(network_id)] unknown_network_id: NetworkId,
    store: LocalNetworkStateStore,
) {
    // return none for unknown network
    assert!(matches!(
        store.read_local_network_state(unknown_network_id),
        Ok(None)
    ));

    // can write one state from scratch
    assert!(store
        .write_local_network_state(&unknown_network_id, &LocalNetworkStateData::default(), &[])
        .is_ok());
}

#[rstest]
fn can_retrieve_state(network_id: NetworkId, store: LocalNetworkStateStore) {
    let lns = LocalNetworkStateData::default();

    // write arbitrary state
    assert!(store
        .write_local_network_state(&network_id, &lns, &[])
        .is_ok());

    // retrieve it
    assert!(
        matches!(store.read_local_network_state(network_id), Ok(Some(retrieved)) if equal_state(&lns, &retrieved))
    );
}

#[rstest]
fn can_update_existing_state(network_id: NetworkId, store: LocalNetworkStateStore) {
    let mut lns = LocalNetworkStateData::default();

    // write initial state
    assert!(store
        .write_local_network_state(&0.into(), &lns, &[])
        .is_ok());

    // update state
    let bridge_exit = Digest::default();
    lns.exit_tree.add_leaf(bridge_exit);

    // write new state
    assert!(store
        .write_local_network_state(&network_id, &lns, &[Hash(bridge_exit)])
        .is_ok());

    // retrieve new state
    assert!(
        matches!(store.read_local_network_state(network_id), Ok(Some(retrieved)) if equal_state(&lns, &retrieved))
    );
}
