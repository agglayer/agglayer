use std::collections::BTreeMap;

use agglayer_types::NetworkId;
use pessimistic_proof_test_suite::{
    sample_data::{NETWORK_A, NETWORK_B},
    scenario::{CertGraph, CertificateHandle},
};
use sp1_sdk::SP1Proof;

/// Contiguous pessimistic proofs per network.
#[derive(Default)]
pub struct AggregationData {
    pub proofs_per_network: BTreeMap<NetworkId, Vec<SP1Proof>>,
    pub certificates_per_network: BTreeMap<NetworkId, Vec<CertificateHandle>>,
    pub scenario: CertGraph,
}

fn insert_handle(
    certificates_per_network: &mut BTreeMap<NetworkId, Vec<CertificateHandle>>,
    network: NetworkId,
    handle: CertificateHandle,
) {
    certificates_per_network
        .entry(network)
        .or_default()
        .push(handle);
}

/// Generate a set of PP per network
pub fn generate_aggregation_data() -> Result<AggregationData, ()> {
    let mut scenario = CertGraph::new();
    let mut certificates_per_network: BTreeMap<NetworkId, Vec<CertificateHandle>> = BTreeMap::new();
    let mut proofs_per_network: BTreeMap<NetworkId, Vec<SP1Proof>> = BTreeMap::new();

    let mut last_b = None;
    for _ in 0..2 {
        let handle = scenario.add_header_with_preconf(NETWORK_B, true).unwrap();
        insert_handle(&mut certificates_per_network, NETWORK_B, handle);
        if let Ok(proof) = common::execute_sp1_for_handle(&scenario, handle) {
            proofs_per_network.entry(NETWORK_B).or_default().push(proof);
        }
        last_b = Some(handle);
    }

    if let Some(last_b) = last_b {
        let handle_a = scenario
            .claims_from_with_preconf(last_b, NETWORK_A, true)
            .unwrap();
        let certificate_a = scenario.certificate(handle_a);
        assert!(certificate_a
            .certificate
            .imported_bridge_exits
            .iter()
            .all(|ib| ib.global_index.network_id() == NETWORK_B));
        insert_handle(&mut certificates_per_network, NETWORK_A, handle_a);
        if let Ok(proof) = common::execute_sp1_for_handle(&scenario, handle_a) {
            proofs_per_network.entry(NETWORK_A).or_default().push(proof);
        }
    }

    Ok(AggregationData {
        proofs_per_network,
        certificates_per_network,
        scenario,
    })
}

#[test]
fn test_aggregation() {
    let aggregation = generate_aggregation_data().unwrap();
    assert!(aggregation.scenario.len() >= 3);

    let mut order = Vec::new();
    let mut scenario = aggregation.scenario.clone();
    while let Some(cert) = scenario.next_to_prove() {
        order.push((cert.network, cert.id));
    }

    assert_eq!(order.len(), aggregation.scenario.len());
}

#[test]
fn test_preconf() {
    let mut aggregation = generate_aggregation_data().unwrap();
    let last_b = aggregation
        .certificates_per_network
        .get(&NETWORK_B)
        .and_then(|handles| handles.last().copied())
        .unwrap();

    let preconf = aggregation
        .scenario
        .contiguous_from_with_preconf(last_b, true);

    assert!(preconf.is_ok());
}
