use std::path::Path;
use std::sync::Arc;

use agglayer_types::primitives::address;
use agglayer_types::{
    BalanceTreeLeaf, BalanceTreeTransition, Certificate, Digest, LocalNetworkStateData, NetworkId,
    NullifierTreeLeaf, NullifierTreeTransition, PessimisticRootInput, U256,
};
use pessimistic_proof::nullifier_tree::NullifierKey;
use pessimistic_proof::unified_bridge::token_info::TokenInfo;
use pessimistic_proof::unified_bridge::CommitmentVersion;
use pessimistic_proof::{core::generate_pessimistic_proof, LocalNetworkState};
use rstest::{fixture, rstest};
use serde::Serialize;
use tracing::info;

use crate::{
    columns::latest_settled_certificate_per_network::{
        LatestSettledCertificatePerNetworkColumn, SettledCertificate,
    },
    error::Error,
    storage::{backup::BackupClient, state_db_cf_definitions, DB},
    stores::{state::StateStore, StateReader as _, StateWriter as _},
    tests::TempDBDir,
};

mod metadata;

#[test]
fn can_retrieve_list_of_network() {
    let tmp = TempDBDir::new();
    let db = Arc::new(DB::open_cf(tmp.path.as_path(), state_db_cf_definitions()).unwrap());
    let store = StateStore::new(db.clone(), BackupClient::noop());
    assert!(store.get_active_networks().unwrap().is_empty());

    db.put::<LatestSettledCertificatePerNetworkColumn>(
        &1.into(),
        &SettledCertificate([0; 32].into(), 0, 0, 0),
    )
    .expect("Unable to put certificate into storage");
    assert!(store.get_active_networks().unwrap().len() == 1);
}

fn equal_state(lhs: &LocalNetworkStateData, rhs: &LocalNetworkStateData) -> bool {
    // local exit tree
    assert_eq!(lhs.exit_tree.leaf_count(), rhs.exit_tree.leaf_count());
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
fn store() -> StateStore {
    let tmp = TempDBDir::new();
    let db = Arc::new(DB::open_cf(tmp.path.as_path(), state_db_cf_definitions()).unwrap());

    StateStore::new(db.clone(), BackupClient::noop())
}

#[rstest]
fn can_handle_empty_state(#[from(network_id)] unknown_network_id: NetworkId, store: StateStore) {
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
fn can_retrieve_state(network_id: NetworkId, store: StateStore) {
    // write arbitrary state
    let mut lns = LocalNetworkStateData::default();
    let leaves = (0..10).map(|_| Digest([5u8; 32])).collect::<Vec<_>>();
    for l in &leaves {
        lns.exit_tree.add_leaf(*l).unwrap();
    }

    assert!(store
        .write_local_network_state(&network_id, &lns, leaves.as_slice())
        .is_ok());

    // retrieve it
    assert!(
        matches!(store.read_local_network_state(network_id), Ok(Some(retrieved)) if equal_state(&lns, &retrieved))
    );
}

#[rstest]
fn can_update_existing_state(network_id: NetworkId, store: StateStore) {
    let mut lns = LocalNetworkStateData::default();

    // write initial state
    assert!(store
        .write_local_network_state(&network_id, &lns, &[])
        .is_ok());

    // update state
    let bridge_exit = [5u8; 32];
    lns.exit_tree.add_leaf(bridge_exit.into()).unwrap();

    // write new state
    assert!(store
        .write_local_network_state(&network_id, &lns, &[Digest(bridge_exit)])
        .is_ok());

    // retrieve new state
    assert!(
        matches!(store.read_local_network_state(network_id), Ok(Some(retrieved)) if equal_state(&lns, &retrieved))
    );
}

#[rstest]
fn can_detect_inconsistent_state(network_id: NetworkId, store: StateStore) {
    let mut lns = LocalNetworkStateData::default();

    // write initial state
    assert!(store
        .write_local_network_state(&network_id, &lns, &[])
        .is_ok());

    // update state
    let bridge_exit = [5u8; 32];
    lns.exit_tree.add_leaf(bridge_exit.into()).unwrap();

    // write new state with missing leaves
    assert!(matches!(
        store.write_local_network_state(&network_id, &lns, &[]),
        Err(Error::InconsistentState { .. })
    ));
}

use pessimistic_proof_test_suite::sample_data::{self as data};

#[rstest]
fn can_read(network_id: NetworkId, store: StateStore) {
    let certificates: Vec<Certificate> = [
        "n15-cert_h0.json",
        "n15-cert_h1.json",
        "n15-cert_h2.json",
        "n15-cert_h3.json",
    ]
    .iter()
    .map(|p| data::load_certificate(p))
    .collect();

    let mut leaves: Vec<Digest> = Vec::new();
    let mut lns = LocalNetworkStateData::default();

    for (idx, certificate) in certificates.iter().enumerate() {
        info!(
            "Certificate ({idx}|{}) | {}, nib:{} b:{}",
            certificate.height,
            certificate.hash(),
            certificate.imported_bridge_exits.len(),
            certificate.bridge_exits.len(),
        );

        let signer = certificate.signer().unwrap().unwrap();
        let l1_info_root = certificate.l1_info_root().unwrap().unwrap_or_default();

        let multi_batch_header = lns
            .make_multi_batch_header(
                certificate,
                signer,
                l1_info_root,
                PessimisticRootInput::Computed(CommitmentVersion::V2),
                None,
            )
            .unwrap();

        info!("Certificate {idx}: successful witness generation");
        let initial_state = LocalNetworkState::from(lns.clone());

        generate_pessimistic_proof(initial_state.into(), &multi_batch_header).unwrap();
        info!("Certificate {idx}: successful native execution");

        for b in &certificate.bridge_exits {
            leaves.push(b.hash());
        }
        lns.apply_certificate(
            certificate,
            signer,
            l1_info_root,
            PessimisticRootInput::Computed(CommitmentVersion::V2),
            None,
        )
        .unwrap();
        info!("Certificate {idx}: successful state transition, waiting for the next");
    }

    let mut before_going_through_disk = lns.clone();

    info!(
        "before DB | root: {}, nb nodes: {}",
        before_going_through_disk.balance_tree.root,
        before_going_through_disk.balance_tree.tree.len()
    );

    before_going_through_disk
        .balance_tree
        .traverse_and_prune()
        .unwrap();
    before_going_through_disk
        .nullifier_tree
        .traverse_and_prune()
        .unwrap();

    info!(
        "before DB (pruned) | root: {}, nb nodes: {}",
        before_going_through_disk.balance_tree.root,
        before_going_through_disk.balance_tree.tree.len()
    );

    // write state
    assert!(store
        .write_local_network_state(&network_id, &before_going_through_disk, leaves.as_slice())
        .is_ok());

    // read state
    let after_going_through_disk = store.read_local_network_state(network_id).unwrap().unwrap();

    info!(
        "after DB | root: {}, nb nodes: {}",
        after_going_through_disk.balance_tree.root,
        after_going_through_disk.balance_tree.tree.len()
    );

    // check that the read succeed and is equal to the state prior to passing by
    // the disk
    assert!(equal_state(
        &before_going_through_disk,
        &after_going_through_disk
    ));
}

#[derive(Serialize, Default)]
pub struct BalanceTreeTestVector {
    transitions: Vec<BalanceTreeTransition>,
}

#[derive(Serialize, Default)]
pub struct NullifierTreeTestVector {
    transitions: Vec<NullifierTreeTransition>,
}
use agglayer_tries::proof::ToBits;

pub struct BalanceLeaf {
    pub amount: U256,
    pub token_info: TokenInfo,
}

#[test]
fn balance_tree_test_vector() {
    let mut lns = LocalNetworkStateData::default();
    let mut test_vector_data = BalanceTreeTestVector::default();

    let mut leaves: Vec<BalanceLeaf> = (1u32..=10u32)
        .map(|i| BalanceLeaf {
            amount: U256::from(i),
            token_info: TokenInfo {
                origin_network: i.into(),
                origin_token_address: address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
            },
        })
        .collect();

    // nullify the amount of the 5th token
    leaves.push(BalanceLeaf {
        amount: U256::from(0),
        token_info: leaves[5].token_info,
    });

    // put 15 on token 5
    leaves.push(BalanceLeaf {
        amount: U256::from(17),
        token_info: leaves[5].token_info,
    });

    // nullify the amount of the 7th token
    leaves.push(BalanceLeaf {
        amount: U256::from(0),
        token_info: leaves[7].token_info,
    });

    // add new token
    leaves.push(BalanceLeaf {
        amount: U256::from(289),
        token_info: TokenInfo {
            origin_network: 19.into(),
            origin_token_address: address!("0000000000000000000000000000000000000000"),
        },
    });

    for leaf in leaves {
        let prev_commitment = lns.get_roots();

        let balance_tree_leaf = BalanceTreeLeaf {
            key: leaf.token_info,
            path: leaf.token_info.to_bits().to_vec(),
            value: leaf.amount.to_be_bytes().into(),
        };

        // update state
        lns.balance_tree
            .update(balance_tree_leaf.key, balance_tree_leaf.value)
            .unwrap();

        let new_commitment = lns.get_roots();

        test_vector_data.transitions.push(BalanceTreeTransition {
            prev_root: prev_commitment.balance_root,
            new_root: new_commitment.balance_root,
            updated_leaf: balance_tree_leaf,
        })
    }

    println!(
        "balance tree test vector: {}",
        serde_json::to_string_pretty(&test_vector_data).unwrap()
    );

    std::fs::write(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_vectors")
            .join("balance_tree_test_vector.json"),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&test_vector_data).unwrap()
        ),
    )
    .expect("failed to write fixture");
}

use agglayer_primitives::utils::{FromBool, Hashable};

#[test]
fn nullifier_tree_test_vector() {
    let mut lns = LocalNetworkStateData::default();
    let mut test_vector_data = NullifierTreeTestVector::default();

    for i in 1u32..=10u32 {
        let prev_commitment = lns.get_roots();
        let nullifier_key = NullifierKey {
            network_id: i.into(),
            let_index: i + 7,
        };

        let nullifier_key_path_bytes = nullifier_key.to_bits();
        let nullifier_tree_leaf = NullifierTreeLeaf {
            key: nullifier_key,
            path: nullifier_key_path_bytes.to_vec(),
            value: Digest::from_bool(true),
        };

        // update state
        lns.nullifier_tree
            .update(nullifier_tree_leaf.key, nullifier_tree_leaf.value)
            .unwrap();

        let new_commitment = lns.get_roots();

        test_vector_data.transitions.push(NullifierTreeTransition {
            prev_root: prev_commitment.nullifier_root,
            new_root: new_commitment.nullifier_root,
            updated_leaf: nullifier_tree_leaf,
        })
    }

    println!(
        "nullifier tree test vector: {}",
        serde_json::to_string_pretty(&test_vector_data).unwrap()
    );

    std::fs::write(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_vectors")
            .join("nullifier_tree_test_vector.json"),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&test_vector_data).unwrap()
        ),
    )
    .expect("failed to write fixture");
}

#[test]
fn import_native_tokens() {
    let certificates: Vec<Certificate> = ["cert_h0.json", "cert_h1.json", "cert_h2.json"]
        .iter()
        .map(|p| data::load_certificate(p))
        .collect();

    let mut lns = LocalNetworkStateData::default();

    for (idx, certificate) in certificates.iter().enumerate() {
        info!(
            "Certificate ({idx}|{}) | {}, nib:{} b:{}",
            certificate.height,
            certificate.hash(),
            certificate.imported_bridge_exits.len(),
            certificate.bridge_exits.len(),
        );

        let signer = certificate.signer().unwrap().expect("Signer");
        let l1_info_root = certificate.l1_info_root().unwrap().unwrap_or_default();

        let multi_batch_header = lns
            .make_multi_batch_header(
                certificate,
                signer,
                l1_info_root,
                PessimisticRootInput::Computed(CommitmentVersion::V2),
                None,
            )
            .unwrap();

        info!("Certificate {idx}: successful witness generation");
        let initial_state = LocalNetworkState::from(lns.clone());

        generate_pessimistic_proof(initial_state.into(), &multi_batch_header).unwrap();
        info!("Certificate {idx}: successful native execution");

        lns.apply_certificate(
            certificate,
            signer,
            l1_info_root,
            PessimisticRootInput::Computed(CommitmentVersion::V2),
            None,
        )
        .unwrap();
        info!("Certificate {idx}: successful state transition, waiting for the next");
    }
}

#[rstest]
#[case("n15-cert_h0")]
#[case("n15-cert_h1")]
#[case("n15-cert_h2")]
#[case("n15-cert_h3")]
fn certificate_serialization(#[case] cert_name: &str) {
    use crate::columns::Codec;

    let certificate = data::load_certificate(&format!("{cert_name}.json"));
    let encoded = certificate.encode().unwrap();
    let hash = pessimistic_proof::keccak::keccak256(&encoded);
    insta::assert_debug_snapshot!(cert_name, hash);
}
