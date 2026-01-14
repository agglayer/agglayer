#![no_main]

use pessimistic_proof_core::{
    generate_pessimistic_proof, multi_batch_header::MultiBatchHeader, NetworkState,
    PessimisticProofOutput,
};

sp1_zkvm::entrypoint!(main);
pub fn main() {
    // Read NetworkState using zero-copy deserialization
    let network_state_bytes = sp1_zkvm::io::read_vec();
    let initial_state =
        NetworkState::try_from(network_state_bytes.as_slice()).unwrap_or_else(|e| {
            panic!(
                "Failed to deserialize NetworkState: {:?}, input length: {}, expected: {}",
                e,
                network_state_bytes.len(),
                pessimistic_proof_core::NETWORK_STATE_ZERO_COPY_SIZE
            )
        });

    let packed_header_bytes = sp1_zkvm::io::read_vec();

    let batch_header_ref = MultiBatchHeader::from_zero_copy_packed_bytes(&packed_header_bytes)
        .unwrap_or_else(|err| panic!("Failed to parse MultiBatchHeader zero-copy: {err}"));

    let batch_header = batch_header_ref
        .to_owned()
        .unwrap_or_else(|err| panic!("Failed to materialize MultiBatchHeader: {err}"));

    let (outputs, _targets) = generate_pessimistic_proof(initial_state, &batch_header).unwrap();

    let pp_inputs = PessimisticProofOutput::bincode_codec()
        .serialize(&outputs)
        .unwrap();

    sp1_zkvm::io::commit_slice(&pp_inputs);
}
