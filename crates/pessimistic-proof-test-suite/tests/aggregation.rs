use std::collections::BTreeMap;

use agglayer_types::{
    aggchain_data::CertificateAggchainDataCtx, L1WitnessCtx, PessimisticRootInput, U256,
};
use pessimistic_proof::{
    core::commitment::{PessimisticRootCommitmentVersion, SignatureCommitmentVersion},
    NetworkState, ELF as PESSIMISTIC_PROOF_ELF,
};
use pessimistic_proof_test_suite::{
    forest::Forest,
    sample_data::{self as data, ETH, USDC},
};
use sp1_sdk::{ProverClient, SP1Proof, SP1Stdin};
use unified_bridge::NetworkId;

/// Contiguious pessimistic proofs per network.
#[derive(Default)]
pub struct AggregationData {
    pub proofs_per_network: BTreeMap<NetworkId, Vec<SP1Proof>>,
}

/// Generates on PP and returns the SP1Proof.
pub fn generate_pp(state: &mut Forest, n_exits: usize) -> Result<SP1Proof, ()> {
    let bridge_exits = data::sample_bridge_exits_01().take(n_exits);
    let initial_state: NetworkState = state.local_state().into();
    let certificate =
        state
            .clone()
            .apply_bridge_exits([], bridge_exits, SignatureCommitmentVersion::V3);

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

    let mut stdin = SP1Stdin::new();
    stdin.write(&initial_state);
    stdin.write(&multi_batch_header);

    let client = ProverClient::from_env();

    // Execute to get cycle numbers
    let (_, report) = client
        .execute(PESSIMISTIC_PROOF_ELF, &stdin)
        .run()
        .expect("execution failed");
    println!("successful execution");
    Err(())
}

/// Generates consecutive PP for a given chain.
pub fn generate_pp_for_chain(_origin_network: NetworkId, _nb_proofs: usize) -> Vec<SP1Proof> {
    todo!();
}

/// Generate a set of PP per network
pub fn generate_aggregation_data() -> Result<AggregationData, ()> {
    let large_amount = U256::MAX.checked_div(U256::from(2u64)).unwrap(); // not max to allow importing bridge exits
    let balances = [(ETH, large_amount), (USDC, large_amount)];
    let mut forest_a = Forest::default()
        .with_network_id(18)
        .with_initial_balances(balances);

    let nb_exits = 1;
    let pp = generate_pp(&mut forest_a, nb_exits);

    Ok(Default::default())
}

#[test]
fn test_aggregation() {
    generate_aggregation_data().unwrap();
}
