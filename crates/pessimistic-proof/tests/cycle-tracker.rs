use pessimistic_proof::bridge_exit::BridgeExit;
use tracing::{debug, info};

mod test_utils;

use test_utils::{forest::Forest, runner::Runner, sample_data as data};

#[rstest::rstest]
#[case::empty(Forest::new([]), std::iter::empty())]
#[case::s01_n000(data::sample_state_01(), std::iter::empty())]
#[case::s01_n001(data::sample_state_01(), data::sample_bridge_exits_01().take(1))]
#[case::s01_n002(data::sample_state_01(), data::sample_bridge_exits_01().take(2))]
#[case::s01_n020(data::sample_state_01(), data::sample_bridge_exits_01().take(20))]
#[case::s01_n100(data::sample_state_01(), data::sample_bridge_exits_01().take(100))]
#[case::s01_full(data::sample_state_01(), data::sample_bridge_exits_01())]
#[ignore = "Too expensive to run by default"]
fn cycles_on_sample_inputs(
    #[case] mut state: Forest,
    #[case] bridge_exits: impl Iterator<Item = BridgeExit>,
) {
    sp1_sdk::utils::setup_logger();

    let withdrawals = bridge_exits.map(|be| (be.token_info, be.amount)).collect::<Vec<_>>();
    let n_exits = withdrawals.len();

    let old_state = state.local_state();
    let batch_header = state.apply_events(&[], &withdrawals);

    let (new_roots, stats) =
        Runner::new().execute(&old_state, &batch_header).expect("execution failed");

    debug!("full execution stats:\n{stats}");
    debug!("result: {new_roots:?}");

    let cycles = stats.total_instruction_count();
    let syscalls = stats.total_syscall_count();
    info!("execution summary: n={n_exits}, cycles={cycles}, syscalls={syscalls}");
}
