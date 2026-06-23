use integrations::{agglayer_setup::get_signer, l1_setup::INTEGRATION_ROLLUP_ID};
use pessimistic_proof_test_suite::forest::Forest;

pub fn type_1_multisig_forest() -> Forest {
    let signer = get_signer(0);

    Forest::default()
        .with_network_id(INTEGRATION_ROLLUP_ID)
        .with_signer(signer)
}
