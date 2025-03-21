use agglayer_types::primitives::U256;
use agglayer_types::{Claim, Digest, Error, PessimisticRootInput};
use pessimistic_proof::core::commitment::{PPRootVersion, PessimisticRoot, SignatureCommitment};
use pessimistic_proof::core::{generate_pessimistic_proof, AggchainData};
use pessimistic_proof::imported_bridge_exit::commit_imported_bridge_exits;
use pessimistic_proof::local_state::LocalNetworkState;
use pessimistic_proof::{bridge_exit::TokenInfo, core};
use pessimistic_proof::{NetworkState, PessimisticProofOutput, ProofError};
use pessimistic_proof_test_suite::{
    forest::Forest,
    sample_data::{ETH, USDC},
    PESSIMISTIC_PROOF_ELF,
};
use rand::random;
use rstest::rstest;
use sp1_sdk::{utils, HashableKey, ProverClient, SP1Stdin};

fn u(x: u64) -> U256 {
    x.try_into().unwrap()
}

fn pp_root_migration_helper(
    previous_version: PPRootVersion,
    new_version: PPRootVersion,
) -> (Result<PessimisticProofOutput, ProofError>, Digest, Digest) {
    let mut forest = Forest::new(vec![(USDC, u(100)), (ETH, u(200))]);
    let imported_bridge_events = vec![(USDC, u(50)), (ETH, u(100)), (USDC, u(10))];
    let bridge_events = vec![(USDC, u(20)), (ETH, u(50)), (USDC, u(130))];

    let initial_state = forest.state_b.clone();
    let certificate = forest.apply_events(&imported_bridge_events, &bridge_events);
    let l1_info_root = certificate.l1_info_root().unwrap().unwrap_or_default();
    let mut multi_batch_header = initial_state
        .make_multi_batch_header(
            &certificate,
            forest.get_signer(),
            l1_info_root,
            PessimisticRootInput::Computed(PPRootVersion::V2),
            None,
        )
        .unwrap();

    let new_state = {
        let mut state = initial_state.clone();
        state
            .apply_certificate(
                &certificate,
                forest.get_signer(),
                l1_info_root,
                PessimisticRootInput::Computed(PPRootVersion::V2),
                None,
            )
            .unwrap();
        state
    };

    // Previous state settled in L1
    let prev_pp_root = PessimisticRoot {
        balance_root: initial_state.balance_tree.root,
        nullifier_root: initial_state.nullifier_tree.root,
        ler_leaf_count: initial_state.exit_tree.leaf_count(),
        height: certificate.height,
        origin_network: *certificate.network_id,
    };

    // Signed transition
    let signature_data = SignatureCommitment {
        new_local_exit_root: multi_batch_header.target.exit_root,
        commit_imported_bridge_exits: commit_imported_bridge_exits(
            multi_batch_header
                .imported_bridge_exits
                .iter()
                .map(|ib| ib.0.global_index),
        ),
        height: certificate.height,
    };

    // New state about to be settled in L1
    let new_pp_root = PessimisticRoot {
        balance_root: new_state.balance_tree.root,
        nullifier_root: new_state.nullifier_tree.root,
        ler_leaf_count: new_state.exit_tree.leaf_count(),
        height: certificate.height + 1,
        origin_network: *certificate.network_id,
    };

    multi_batch_header.prev_pessimistic_root = prev_pp_root.compute_pp_root(previous_version);
    let (signature, signer) = forest.sign(signature_data.commitment(new_version)).unwrap();
    multi_batch_header.aggchain_proof = AggchainData::ECDSA { signer, signature };

    (
        generate_pessimistic_proof(initial_state.into(), &multi_batch_header),
        prev_pp_root.compute_pp_root(previous_version),
        new_pp_root.compute_pp_root(new_version),
    )
}

#[rstest]
#[case(PPRootVersion::V2, PPRootVersion::V2)] // pre-migration
#[case(PPRootVersion::V2, PPRootVersion::V3)] // migration
#[case(PPRootVersion::V3, PPRootVersion::V3)] // post-migration
fn pp_root_migration(#[case] prev_version: PPRootVersion, #[case] new_version: PPRootVersion) {
    let (result, expected_prev_pp_root, expected_new_pp_root) =
        pp_root_migration_helper(prev_version, new_version);

    let PessimisticProofOutput {
        prev_pessimistic_root,
        new_pessimistic_root,
        ..
    } = result.unwrap();

    assert_eq!(expected_prev_pp_root, prev_pessimistic_root);
    assert_eq!(expected_new_pp_root, new_pessimistic_root);
}

#[test]
fn forbidden_pp_root_transition() {
    assert!(matches!(
        pp_root_migration_helper(PPRootVersion::V3, PPRootVersion::V2).0,
        Err(ProofError::InconsistentSignedPayload)
    ));
}

fn e2e_local_pp_simple_helper(
    initial_balances: impl IntoIterator<Item = (TokenInfo, U256)>,
    imported_events: impl IntoIterator<Item = (TokenInfo, U256)>,
    events: impl IntoIterator<Item = (TokenInfo, U256)>,
) {
    let imported_events = imported_events.into_iter().collect::<Vec<_>>();
    let events = events.into_iter().collect::<Vec<_>>();

    let mut forest = Forest::new(initial_balances);
    let initial_state = forest.state_b.clone();
    let certificate = forest.apply_events(&imported_events, &events);
    let l1_info_root = certificate.l1_info_root().unwrap().unwrap_or_default();
    let multi_batch_header = initial_state
        .make_multi_batch_header(
            &certificate,
            forest.get_signer(),
            l1_info_root,
            PessimisticRootInput::Computed(PPRootVersion::V2),
            None,
        )
        .unwrap();
    generate_pessimistic_proof(initial_state.into(), &multi_batch_header).unwrap();
}

#[test]
fn e2e_local_pp_simple() {
    e2e_local_pp_simple_helper(
        vec![(USDC, u(100)), (ETH, u(200))],
        vec![(USDC, u(50)), (ETH, u(100)), (USDC, u(10))],
        vec![(USDC, u(20)), (ETH, u(50)), (USDC, u(130))],
    )
}

#[test]
fn e2e_local_pp_simple_zero_initial_balances() {
    e2e_local_pp_simple_helper(
        [],
        vec![(USDC, u(50)), (ETH, u(100)), (USDC, u(10))],
        vec![(USDC, u(20)), (ETH, u(50)), (USDC, u(30))],
    )
}

#[test]
fn e2e_local_pp_random() {
    let target = u(u64::MAX);
    let upper = u64::MAX / 10;
    let mut forest = Forest::new(vec![(USDC, target), (ETH, target)]);
    // Generate random bridge events such that the sum of the USDC and ETH amounts
    // is less than `target`
    let get_events = || {
        let mut usdc_acc = U256::ZERO;
        let mut eth_acc = U256::ZERO;
        let mut events = Vec::new();
        loop {
            let amount = u(random::<u64>() % upper);
            let token = if random::<u64>() & 1 == 1 { USDC } else { ETH };
            if token == USDC {
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

    let initial_state = forest.state_b.clone();
    let certificate = forest.apply_events(&imported_bridge_events, &bridge_events);

    let l1_info_root = certificate.l1_info_root().unwrap().unwrap_or_default();
    let multi_batch_header = initial_state
        .make_multi_batch_header(
            &certificate,
            forest.get_signer(),
            l1_info_root,
            PessimisticRootInput::Computed(PPRootVersion::V2),
            None,
        )
        .unwrap();

    generate_pessimistic_proof(initial_state.into(), &multi_batch_header).unwrap();
}

#[test]
fn inconsistent_ger() {
    let mut forest = Forest::new(vec![(USDC, u(100)), (ETH, u(200))]);
    let imported_bridge_events = vec![(USDC, u(50)), (ETH, u(100)), (USDC, u(10))];
    let bridge_events = vec![(USDC, u(20)), (ETH, u(50)), (USDC, u(130))];

    let initial_state = forest.state_b.clone();
    let mut certificate = forest.apply_events(&imported_bridge_events, &bridge_events);

    // Change the global exit root
    {
        let Claim::Mainnet(ref mut claim_0) = certificate.imported_bridge_exits[0].claim_data
        else {
            unreachable!("expect from mainnet");
        };

        claim_0.l1_leaf.inner.global_exit_root = Digest::default();
    }

    let l1_info_root = certificate.l1_info_root().unwrap().unwrap_or_default();
    let res = initial_state.make_multi_batch_header(
        &certificate,
        forest.get_signer(),
        l1_info_root,
        PessimisticRootInput::Computed(PPRootVersion::V2),
        None,
    );

    assert!(matches!(res, Err(Error::InconsistentGlobalExitRoot)))
}

// Same as `e2e_local_pp_simple` with an SP1 proof on top
#[test]
#[ignore]
fn test_sp1_simple() {
    // Setup logging.
    utils::setup_logger();

    let mut forest = Forest::new(vec![(USDC, u(100)), (ETH, u(200))]);
    let imported_bridge_events = vec![(USDC, u(50)), (ETH, u(100)), (USDC, u(10))];
    let bridge_events = vec![(USDC, u(20)), (ETH, u(50)), (USDC, u(130))];

    let initial_state = forest.state_b.clone();
    let (certificate, aggchain_vkey, aggchain_params, aggchain_proof) =
        forest.apply_events_with_aggchain_proof(&imported_bridge_events, &bridge_events);
    let l1_info_root = certificate.l1_info_root().unwrap().unwrap_or_default();

    let mut multi_batch_header = initial_state
        .make_multi_batch_header(
            &certificate,
            forest.get_signer(),
            l1_info_root,
            PessimisticRootInput::Computed(PPRootVersion::V2),
            None,
        )
        .unwrap();

    // Set the aggchain proof to the sp1 variant
    multi_batch_header.aggchain_proof = core::AggchainData::Generic {
        aggchain_params: aggchain_params.into(),
        aggchain_vkey: aggchain_vkey.hash_u32(),
    };

    let initial_state: NetworkState = LocalNetworkState::from(initial_state).into();
    let mut stdin = SP1Stdin::new();
    stdin.write(&initial_state);
    stdin.write(&multi_batch_header);
    stdin.write_proof(
        *aggchain_proof.try_as_compressed().unwrap(),
        aggchain_vkey.vk,
    );

    // Execute the PP within SP1
    let client = ProverClient::from_env();
    let (_public_vals, _report) = client.execute(PESSIMISTIC_PROOF_ELF, &stdin).run().unwrap();
}
