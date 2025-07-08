#![no_main]

use pessimistic_proof_core::{
    generate_pessimistic_proof, keccak::Keccak256Hasher, multi_batch_header::MultiBatchHeader,
    NetworkState, PessimisticProofOutput,
};

sp1_zkvm::entrypoint!(main);
pub fn main() {
    println!("cycle-tracker-report-start: all");
    println!("cycle-tracker-report-start: safe_read_network_state");
    let initial_state_bytes = sp1_zkvm::io::read_vec();
    let initial_state = rkyv::from_bytes::<NetworkState, rkyv::rancor::Error>(&initial_state_bytes)
        .expect("Failed to deserialize witness data.");
    println!("cycle-tracker-report-end: safe_read_network_state");
    // let initial_state = sp1_zkvm::io::read::<NetworkState>();
    println!("cycle-tracker-report-start: read_batch_header");
    let batch_header = sp1_zkvm::io::read::<MultiBatchHeader<Keccak256Hasher>>();
    println!("cycle-tracker-report-end: read_batch_header");

    println!("cycle-tracker-report-start: generate_pessimistic_proof");
    let (outputs, _targets) = generate_pessimistic_proof(initial_state, &batch_header).unwrap();
    println!("cycle-tracker-report-end: generate_pessimistic_proof");

    println!("cycle-tracker-report-start: serialize_pessimistic_proof_output");
    let pp_inputs = PessimisticProofOutput::bincode_codec()
        .serialize(&outputs)
        .unwrap();
    println!("cycle-tracker-report-end: serialize_pessimistic_proof_output");

    println!("cycle-tracker-report-start: commit_slice");
    sp1_zkvm::io::commit_slice(&pp_inputs);
    println!("cycle-tracker-report-end: commit_slice");
}
