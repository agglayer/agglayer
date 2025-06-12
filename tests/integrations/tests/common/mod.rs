use integrations::agglayer_setup::get_signer;
use pessimistic_proof_test_suite::forest::Forest;

pub fn type_0_ecdsa_forest() -> Forest {
    let signer = get_signer(0);

    Forest::default().with_network_id(2).with_signer(signer)
}
