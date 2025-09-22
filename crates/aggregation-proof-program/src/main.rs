#![no_main]

use aggregation_proof_core::{AggregationPublicValues, AggregationWitness};

sp1_zkvm::entrypoint!(main);
pub fn main() {
    let agg_witness = sp1_zkvm::io::read::<AggregationWitness>();

    let outputs = agg_witness.verify_aggregation().unwrap();

    let pub_values = AggregationPublicValues::bincode_codec()
        .serialize(&outputs)
        .unwrap();

    sp1_zkvm::io::commit_slice(&pub_values);
}
