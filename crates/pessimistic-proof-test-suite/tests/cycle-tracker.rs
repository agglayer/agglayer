use std::time::Duration;

use pessimistic_proof::bridge_exit::BridgeExit;
use pessimistic_proof_test_suite::{forest::Forest, runner::Runner, sample_data as data};

#[rstest::rstest]
#[timeout(Duration::from_secs(10))]
fn sanity_check() {
    cycles_on_sample_inputs("s00_be000", Forest::new([]), std::iter::empty());
}

#[rstest::rstest]
#[timeout(Duration::from_secs(30))]
fn cycles_on_state01(#[values(0, 1, 2, 20, 50, 100, usize::MAX)] n_exits: usize) {
    let bridge_exits = data::sample_bridge_exits_01().take(n_exits);
    let name = format!("s01_be{:03}", bridge_exits.len());
    cycles_on_sample_inputs(&name, data::sample_state_01(), bridge_exits);
}

fn cycles_on_sample_inputs(
    name: &str,
    mut state: Forest,
    bridge_exits: impl IntoIterator<Item = BridgeExit>,
) {
    let old_state = state.local_state();
    let certificate = state.clone().apply_bridge_exits([], bridge_exits);

    let multi_batch_header = state
        .state_b
        .apply_certificate(
            &certificate,
            state.get_signer(),
            certificate.l1_info_root().unwrap().unwrap_or_default(),
        )
        .unwrap();

    let (new_roots, stats) = Runner::new()
        .execute(&old_state.into(), &multi_batch_header)
        .expect("execution failed");

    // Double check the roots match what is calculated by the proof-external state.
    state.assert_output_matches(&new_roots);

    insta::assert_snapshot!(name, stats);
}
