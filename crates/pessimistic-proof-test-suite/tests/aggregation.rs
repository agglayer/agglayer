use std::collections::BTreeMap;

use agglayer_types::{
    aggchain_data::CertificateAggchainDataCtx, L1WitnessCtx, PessimisticRootInput, U256,
};
use pessimistic_proof::{
    core::commitment::{PessimisticRootCommitmentVersion, SignatureCommitmentVersion},
    NetworkState, PessimisticProofOutput, ELF as PESSIMISTIC_PROOF_ELF,
};
use pessimistic_proof_test_suite::{
    forest::Forest,
    sample_data::{self as data, ETH, NETWORK_B, USDC},
};
use sp1_sdk::{ProverClient, SP1Proof, SP1Stdin};
use unified_bridge::NetworkId;

fn u(x: u64) -> U256 {
    x.try_into().unwrap()
}
/// Contiguous pessimistic proofs per network.
#[derive(Default)]
pub struct AggregationData {
    pub proofs_per_network: BTreeMap<NetworkId, Vec<SP1Proof>>,
}

pub fn initial_forest(network_id: NetworkId) -> Forest {
    let large_amount = U256::MAX.checked_div(U256::from(2u64)).unwrap(); // not max to allow importing bridge exits
    let balances = [(ETH, large_amount), (USDC, large_amount)];

    Forest::default()
        .with_network_id(network_id)
        .with_initial_balances(balances)
}

/// Generates on PP and returns the SP1Proof.
pub fn generate_pp(state: &mut Forest, n_exits: usize, with_preconf: bool) -> Result<SP1Proof, ()> {
    let bridge_exits = data::sample_bridge_exits_01().take(n_exits);
    let initial_state: NetworkState = state.local_state().into();
    let certificate = state.clone().apply_bridge_exits_with_preconf(
        vec![(USDC, u(50)), (ETH, u(100)), (USDC, u(10))],
        bridge_exits,
        SignatureCommitmentVersion::V3,
        true,
    );

    let multi_batch_header = state
        .state_b
        .apply_certificate(
            &certificate,
            L1WitnessCtx {
                l1_info_root: certificate.l1_info_root().unwrap().unwrap_or_default(),
                prev_pessimistic_root: PessimisticRootInput::Computed(
                    PessimisticRootCommitmentVersion::V3,
                ),
                aggchain_data_ctx: CertificateAggchainDataCtx::LegacyEcdsa {
                    signer: state.get_signer(),
                },
            },
        )
        .unwrap();

    println!("origin-network: {:?}", multi_batch_header.origin_network);
    let mut stdin = SP1Stdin::new();
    stdin.write(&initial_state);
    stdin.write(&multi_batch_header);

    let client = ProverClient::from_env();

    // Execute to get cycle numbers
    let (pv, report) = client
        .execute(PESSIMISTIC_PROOF_ELF, &stdin)
        .run()
        .expect("execution failed");

    let pv_sp1_execute: PessimisticProofOutput = PessimisticProofOutput::bincode_codec()
        .deserialize(pv.as_slice())
        .unwrap();

    println!("public values: {pv_sp1_execute:?}");
    Err(())
}

/// Generates consecutive PP for a given chain.
pub fn generate_pp_for_chain(_origin_network: NetworkId, _nb_proofs: usize) -> Vec<SP1Proof> {
    todo!();
}

/// Generate a set of PP per network
pub fn generate_aggregation_data() -> Result<AggregationData, ()> {
    let mut forest = initial_forest(NETWORK_B.into());

    let nb_exits = 1;
    let pp = generate_pp(&mut forest, nb_exits, false);

    Ok(Default::default())
}

#[test]
fn test_aggregation() {
    generate_aggregation_data().unwrap();
}

// -----

#[test]
fn test_preconf() {
    generate_aggregation_data().unwrap();
    // generate 2 certs referring to preconf LER

    // generate PP for those

    // generate aggregation of those PP
}
