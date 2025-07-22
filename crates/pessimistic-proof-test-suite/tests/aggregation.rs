use unified_bridge::NetworkId;

/// Contiguious pessimistic proofs per network.
pub struct AggregationData {
    proofs_per_network: BTreeMap<NetworkId, Vec<SP1Proof>>,
}

/// Generates on PP and returns the SP1Proof.
pub fn generate_pp(origin_network: NetworkId, n_exits: usize) -> SP1Proof {
    let bridge_exits = data::sample_bridge_exits_01().take(n_exits);
}

/// Generates consecutive PP for a given chain.
pub fn generate_pp_for_chain(origin_network: NetworkId, nb_proofs: usize) -> Vec<SP1Proof> {
    todo!();
}

/// Generate a set of PP per network
pub fn generate_aggregation_data() -> Result<AggregationData, ()> {
    todo!();
}
