use pessimistic_proof::{
    bridge_exit::TokenInfo, local_balance_tree::LocalBalanceTree,
    multi_batch_header::MultiBatchHeader, nullifier_tree::NullifierTree, LocalNetworkState,
    PPPublicInputs,
};
use pessimistic_proof_prover::aggregation::{
    prove_pessimistic_proofs_aggregation, vk_pessimistic_proofs_aggregation,
};
use pessimistic_proof_prover::pessimistic::{prove_pessimistic_proof, vk_pessimistic_proof};
use pessimistic_proof_test_suite::{
    forest::{compute_signature_info, Forest},
    sample_data::{ETH, NETWORK_A, NETWORK_B, USDC},
};
use rand::random;
use reth_primitives::U256;
use sp1_sdk::{utils, ProverClient};

fn u(x: u64) -> U256 {
    x.try_into().unwrap()
}

fn e2e_local_pp_simple_helper(
    initial_balances: impl IntoIterator<Item = (TokenInfo, U256)>,
    imported_events: impl IntoIterator<Item = (TokenInfo, U256)>,
    events: impl IntoIterator<Item = (TokenInfo, U256)>,
) {
    let imported_events = imported_events.into_iter().collect::<Vec<_>>();
    let events = events.into_iter().collect::<Vec<_>>();

    let mut forest = Forest::new(initial_balances);
    let mut local_state = forest.local_state();
    let batch_header = forest.apply_events(&imported_events, &events);

    local_state.apply_batch_header(&batch_header).unwrap()
}

#[test]
fn e2e_local_pp_simple() {
    e2e_local_pp_simple_helper(
        vec![(*USDC, u(100)), (*ETH, u(200))],
        vec![(*USDC, u(50)), (*ETH, u(100)), (*USDC, u(10))],
        vec![(*USDC, u(20)), (*ETH, u(50)), (*USDC, u(130))],
    )
}

#[test]
fn e2e_local_pp_simple_zero_initial_balances() {
    e2e_local_pp_simple_helper(
        [],
        vec![(*USDC, u(50)), (*ETH, u(100)), (*USDC, u(10))],
        vec![(*USDC, u(20)), (*ETH, u(50)), (*USDC, u(30))],
    )
}

#[test]
fn e2e_local_pp_random() {
    let target = u(u64::MAX);
    let upper = u64::MAX / 10;
    let mut forest = Forest::new(vec![(*USDC, target), (*ETH, target)]);
    let prev_local_exit_root = forest.local_exit_tree.get_root();
    let prev_balance_root = forest.local_balance_tree.root;
    let prev_nullifier_root = forest.nullifier_set.root;
    let mut local_state = LocalNetworkState {
        exit_tree: forest.local_exit_tree.clone(),
        balance_tree: LocalBalanceTree::new_with_root(prev_balance_root),
        nullifier_set: NullifierTree::new_with_root(prev_nullifier_root),
    };
    // Generate random bridge events such that the sum of the USDC and ETH amounts
    // is less than `target`
    let get_events = || {
        let mut usdc_acc = U256::ZERO;
        let mut eth_acc = U256::ZERO;
        let mut events = Vec::new();
        loop {
            let amount = u(random::<u64>() % upper);
            let token = if random::<u64>() & 1 == 1 {
                *USDC
            } else {
                *ETH
            };
            if token == *USDC {
                usdc_acc += amount;
                if usdc_acc > target {
                    break;
                }
            } else {
                eth_acc += amount;
                if eth_acc > target {
                    break;
                }
            }
            events.push((token, amount));
        }
        events
    };
    let imported_bridge_events = get_events();
    let bridge_events = get_events();
    let balances_proofs = forest.balances_proofs(&imported_bridge_events, &bridge_events);
    let imported_bridge_exits = forest.imported_bridge_exits(&imported_bridge_events);
    let bridge_exits = forest.bridge_exits(&bridge_events);
    let new_local_exit_root = forest.local_exit_tree.get_root();
    let (imported_exits_root, signer, signature) =
        compute_signature_info(new_local_exit_root, &imported_bridge_exits);
    let batch_header = MultiBatchHeader {
        origin_network: *NETWORK_B,
        prev_local_exit_root,
        new_local_exit_root,
        bridge_exits,
        imported_bridge_exits,
        imported_exits_root: Some(imported_exits_root),
        imported_local_exit_roots: [(*NETWORK_A, forest.local_exit_tree_data_a.get_root())].into(),
        balances_proofs,
        prev_balance_root,
        new_balance_root: forest.local_balance_tree.root,
        prev_nullifier_root,
        new_nullifier_root: forest.nullifier_set.root,
        signer,
        signature,
    };

    local_state.apply_batch_header(&batch_header).unwrap()
}

// Same as `e2e_local_pp_simple` with an SP1 proof on top
#[test]
#[ignore]
fn test_sp1_simple() {
    // Setup logging.
    utils::setup_logger();

    let mut forest = Forest::new(vec![(*USDC, u(100)), (*ETH, u(200))]);
    let prev_local_exit_root = forest.local_exit_tree.get_root();
    let prev_balance_root = forest.local_balance_tree.root;
    let prev_nullifier_root = forest.nullifier_set.root;
    let local_state = LocalNetworkState {
        exit_tree: forest.local_exit_tree.clone(),
        balance_tree: LocalBalanceTree::new_with_root(prev_balance_root),
        nullifier_set: NullifierTree::new_with_root(prev_nullifier_root),
    };
    let imported_bridge_events = vec![(*USDC, u(50)), (*ETH, u(100)), (*USDC, u(10))];
    let bridge_events = vec![(*USDC, u(20)), (*ETH, u(50)), (*USDC, u(130))];
    let balances_proofs = forest.balances_proofs(&imported_bridge_events, &bridge_events);
    let imported_bridge_exits = forest.imported_bridge_exits(&imported_bridge_events);
    let bridge_exits = forest.bridge_exits(&bridge_events);
    let new_local_exit_root = forest.local_exit_tree.get_root();
    let (imported_exits_root, signer, signature) =
        compute_signature_info(new_local_exit_root, &imported_bridge_exits);
    let batch_header = MultiBatchHeader {
        origin_network: *NETWORK_B,
        prev_local_exit_root,
        new_local_exit_root,
        bridge_exits,
        imported_bridge_exits,
        imported_exits_root: Some(imported_exits_root),
        imported_local_exit_roots: [(*NETWORK_A, forest.local_exit_tree_data_a.get_root())].into(),
        balances_proofs,
        prev_balance_root,
        new_balance_root: forest.local_balance_tree.root,
        prev_nullifier_root,
        new_nullifier_root: forest.nullifier_set.root,
        signer,
        signature,
    };

    let proof = prove_pessimistic_proof(local_state, batch_header);
    let vk = vk_pessimistic_proof();

    let pis: PPPublicInputs = bincode::deserialize(proof.public_values.as_slice()).unwrap();
    assert_eq!(pis.signer, signer); // sanity check

    let client = ProverClient::new();
    // Verify proof and public values
    client.verify(&proof, &vk).expect("verification failed");
}

// Same as `test_sp1_simple` where two (identical for now) proofs are
// aggregated.
#[test]
#[ignore]
fn test_sp1_simple_aggregation() {
    // Setup logging.
    utils::setup_logger();

    let mut forest = Forest::new(vec![(*USDC, u(100)), (*ETH, u(200))]);
    let prev_local_exit_root = forest.local_exit_tree.get_root();
    let prev_balance_root = forest.local_balance_tree.root;
    let prev_nullifier_root = forest.nullifier_set.root;
    let local_state = LocalNetworkState {
        exit_tree: forest.local_exit_tree.clone(),
        balance_tree: LocalBalanceTree::new_with_root(prev_balance_root),
        nullifier_set: NullifierTree::new_with_root(prev_nullifier_root),
    };
    let imported_bridge_events = vec![(*USDC, u(50)), (*ETH, u(100)), (*USDC, u(10))];
    let bridge_events = vec![(*USDC, u(20)), (*ETH, u(50)), (*USDC, u(130))];
    let balances_proofs = forest.balances_proofs(&imported_bridge_events, &bridge_events);
    let imported_bridge_exits = forest.imported_bridge_exits(&imported_bridge_events);
    let bridge_exits = forest.bridge_exits(&bridge_events);
    let new_local_exit_root = forest.local_exit_tree.get_root();
    let (imported_exits_root, signer, signature) =
        compute_signature_info(new_local_exit_root, &imported_bridge_exits);
    let batch_header = MultiBatchHeader {
        origin_network: *NETWORK_B,
        prev_local_exit_root,
        new_local_exit_root,
        bridge_exits,
        imported_bridge_exits,
        imported_exits_root: Some(imported_exits_root),
        imported_local_exit_roots: [(*NETWORK_A, forest.local_exit_tree_data_a.get_root())].into(),
        balances_proofs,
        prev_balance_root,
        new_balance_root: forest.local_balance_tree.root,
        prev_nullifier_root,
        new_nullifier_root: forest.nullifier_set.root,
        signer,
        signature,
    };

    let proof = prove_pessimistic_proof(local_state, batch_header);
    let proof2 = proof.clone();
    let vk = vk_pessimistic_proof();
    let agg_proof = prove_pessimistic_proofs_aggregation(vec![proof, proof2], vk);
    let agg_vk = vk_pessimistic_proofs_aggregation();

    let client = ProverClient::new();
    // Verify proof and public values
    client
        .verify(&agg_proof, &agg_vk)
        .expect("verification failed");
}
