use agglayer_types::{
    aggchain_data::CertificateAggchainDataCtx, primitives::U256, testutils::multisig_1_of_1_ctx,
    Certificate, Digest, Error, L1WitnessCtx, PessimisticRootInput,
};
use pessimistic_proof::{
    core::{
        commitment::{PessimisticRootCommitmentValues, PessimisticRootCommitmentVersion},
        generate_pessimistic_proof, AggchainData, AggchainProof, MultiSignature,
    },
    local_state::LocalNetworkState,
    unified_bridge::TokenInfo,
    NetworkState,
};
use pessimistic_proof_test_suite::{
    forest::Forest,
    sample_data::{ETH, USDC},
    PESSIMISTIC_PROOF_ELF,
};
use rand::random;
use sp1_sdk::{
    blocking::{Prover, ProverClient},
    utils, Elf, HashableKey, SP1Stdin,
};
use unified_bridge::Claim;

fn u(x: u64) -> U256 {
    x.try_into().unwrap()
}

fn multisig_ctx(certificate: &Certificate, signer: agglayer_types::Address) -> L1WitnessCtx {
    L1WitnessCtx {
        l1_info_root: certificate.l1_info_root().unwrap().unwrap_or_default(),
        prev_pessimistic_root: PessimisticRootInput::Computed(
            PessimisticRootCommitmentVersion::V2,
        ),
        aggchain_data_ctx: CertificateAggchainDataCtx::MultisigOnly(multisig_1_of_1_ctx(
            certificate, signer,
        )),
    }
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
    let multi_batch_header = initial_state
        .make_multi_batch_header(&certificate, multisig_ctx(&certificate, forest.get_signer()))
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
fn e2e_local_pp_overflow_attempt() {
    e2e_local_pp_simple_helper(
        [],
        vec![
            (USDC, U256::MAX),
            (USDC, u(3)),
            (ETH, u(100)),
            (USDC, u(10)),
        ],
        vec![(USDC, u(20)), (ETH, u(50)), (USDC, u(30))],
    )
}

#[test]
fn e2e_local_pp_random() {
    let target = u(u64::MAX);
    let upper = u64::MAX / 10;
    let mut forest = Forest::new(vec![(USDC, target), (ETH, target)]);
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

    let multi_batch_header = initial_state
        .make_multi_batch_header(&certificate, multisig_ctx(&certificate, forest.get_signer()))
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

    {
        let Claim::Mainnet(ref mut claim_0) = certificate.imported_bridge_exits[0].claim_data
        else {
            unreachable!("expect from mainnet");
        };

        claim_0.l1_leaf.inner.global_exit_root = Digest::default();
    }

    let res = initial_state.make_multi_batch_header(
        &certificate,
        multisig_ctx(&certificate, forest.get_signer()),
    );

    assert!(matches!(res, Err(Error::InconsistentGlobalExitRoot)))
}

#[test]
fn multisig_pp_root_version_is_v3() {
    let mut forest = Forest::new(vec![(USDC, u(100)), (ETH, u(200))]);
    let initial_state = forest.state_b.clone();
    let certificate = forest.apply_events(&[], &[(USDC, u(1))]);
    let signer = forest.get_signer();
    certificate.verify_multisig_1_of_1(signer).unwrap();

    let expected_prev_pp_root = PessimisticRootCommitmentValues {
        balance_root: initial_state.balance_tree.root.into(),
        nullifier_root: initial_state.nullifier_tree.root.into(),
        ler_leaf_count: initial_state.exit_tree.leaf_count(),
        height: certificate.height.as_u64(),
        origin_network: certificate.network_id,
    }
    .compute_pp_root(PessimisticRootCommitmentVersion::V2);

    let multi_batch_header = initial_state
        .make_multi_batch_header(
            &certificate,
            L1WitnessCtx {
                l1_info_root: certificate.l1_info_root().unwrap().unwrap_or_default(),
                prev_pessimistic_root: PessimisticRootInput::Fetched(expected_prev_pp_root),
                aggchain_data_ctx: CertificateAggchainDataCtx::MultisigOnly(
                    multisig_1_of_1_ctx(&certificate, signer),
                ),
            },
        )
        .unwrap();

    let (pv, _) = generate_pessimistic_proof(initial_state.clone().into(), &multi_batch_header).unwrap();
    assert_eq!(pv.prev_pessimistic_root, expected_prev_pp_root);
}

// Same as `e2e_local_pp_simple` with an SP1 proof on top
#[test]
#[ignore]
fn test_sp1_simple() {
    utils::setup_logger();

    let mut forest = Forest::new(vec![(USDC, u(100)), (ETH, u(200))]);
    let imported_bridge_events = vec![(USDC, u(50)), (ETH, u(100)), (USDC, u(10))];
    let bridge_events = vec![(USDC, u(20)), (ETH, u(50)), (USDC, u(130))];

    let initial_state = forest.state_b.clone();
    let (certificate, aggchain_vkey, aggchain_params, aggchain_proof, signature) =
        forest.apply_events_with_aggchain_proof(&imported_bridge_events, &bridge_events);

    let mut multi_batch_header = initial_state
        .make_multi_batch_header(
            &certificate,
            multisig_ctx(&certificate, forest.get_signer()),
        )
        .unwrap();

    multi_batch_header.aggchain_data = AggchainData::MultisigAndAggchainProof {
        multisig: MultiSignature {
            signatures: vec![Some(signature)],
            expected_signers: vec![forest.get_signer()],
            threshold: 1,
        },
        aggchain_proof: AggchainProof {
            aggchain_params: aggchain_params.into(),
            aggchain_vkey: aggchain_vkey.hash_u32(),
        },
    };

    let initial_state: NetworkState = LocalNetworkState::from(initial_state).into();
    let mut stdin = SP1Stdin::new();
    stdin.write(&initial_state);
    stdin.write(&multi_batch_header);
    stdin.write_proof(
        *aggchain_proof.try_as_compressed().unwrap(),
        aggchain_vkey.vk,
    );

    let client = ProverClient::from_env();
    let (_public_vals, _report) = client
        .execute(Elf::Static(PESSIMISTIC_PROOF_ELF), stdin)
        .run()
        .unwrap();
}
