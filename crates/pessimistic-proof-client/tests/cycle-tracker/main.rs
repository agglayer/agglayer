use pessimistic_proof::{BridgeExit, NetworkId};
use pessimistic_proof_client::{Certificate, Client, LocalNetworkState};
use tracing::{debug, info};

mod data;

#[rstest::rstest]
#[case::empty(data::empty_state(), std::iter::empty())]
#[case::s01_n000(data::sample_state_01(), std::iter::empty())]
#[case::s01_n001(data::sample_state_01(), data::sample_bridge_exits_01().take(1))]
#[case::s01_n002(data::sample_state_01(), data::sample_bridge_exits_01().take(2))]
#[case::s01_n020(data::sample_state_01(), data::sample_bridge_exits_01().take(20))]
#[case::s01_n100(data::sample_state_01(), data::sample_bridge_exits_01().take(100))]
#[case::s01_full(data::sample_state_01(), data::sample_bridge_exits_01())]
fn cycles_on_sample_inputs(
    #[case] state: LocalNetworkState,
    #[case] bridge_exits: impl Iterator<Item = BridgeExit>,
) {
    sp1_sdk::utils::setup_logger();

    let origin_network = NetworkId::from(0);
    let exit_root = state.exit_tree.get_root();
    let bridge_exits: Vec<_> = bridge_exits.collect();
    let n_exits = bridge_exits.len();

    let certificate = Certificate::new(origin_network, exit_root, bridge_exits);

    let (roots, stats) = Client::new()
        .execute(&state, &certificate)
        .expect("execution failed");

    debug!("result: {roots:?}");
    info!("execution summary: n={n_exits}, {stats}");
}
