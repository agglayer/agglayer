#![no_main]

use pessimistic_proof_core::{
    generate_pessimistic_proof, multi_batch_header::MultiBatchHeader, NetworkState,
    PessimisticProofOutput,
};

sp1_zkvm::entrypoint!(main);
pub fn main() {
    // Read NetworkState using zero-copy deserialization
    let network_state_bytes = sp1_zkvm::io::read_vec();
    let initial_state = NetworkState::try_from(network_state_bytes.as_slice())
        .expect("Failed to deserialize NetworkState");

    // MultiBatchHeader still uses bincode for now
    let batch_header = sp1_zkvm::io::read::<MultiBatchHeader>();

    let (outputs, _targets) = generate_pessimistic_proof(initial_state, &batch_header).unwrap();

    let pp_inputs = PessimisticProofOutput::bincode_codec()
        .serialize(&outputs)
        .unwrap();

    sp1_zkvm::io::commit_slice(&pp_inputs);
}
