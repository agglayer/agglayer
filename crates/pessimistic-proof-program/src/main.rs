#![no_main]

use pessimistic_proof_core::{
    generate_pessimistic_proof, keccak::Keccak256Hasher, multi_batch_header::MultiBatchHeader,
    NetworkState, PessimisticProofOutput,
};

sp1_zkvm::entrypoint!(main);
pub fn main() {
    let raw_data = sp1_zkvm::io::read_vec();
    let initial_state = NetworkState::from_bytes_zero_copy(&raw_data).unwrap();
    
    // Note: This zero-copy approach only captures the fixed-size header data.
    // The variable-length data (bridge_exits, imported_bridge_exits, balances_proofs)
    // would need to be serialized separately for a complete solution.
    let batch_header_bytes = sp1_zkvm::io::read_vec();
    let batch_header = MultiBatchHeader::<Keccak256Hasher>::from_bytes_zero_copy(&batch_header_bytes).unwrap();

    let (outputs, _targets) = generate_pessimistic_proof(initial_state, &batch_header).unwrap();

    let pp_inputs = PessimisticProofOutput::bincode_codec()
        .serialize(&outputs)
        .unwrap();

    sp1_zkvm::io::commit_slice(&pp_inputs);
}
