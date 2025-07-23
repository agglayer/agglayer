#![no_main]

use pessimistic_proof_core::{
    generate_pessimistic_proof, keccak::Keccak256Hasher, multi_batch_header::MultiBatchHeader,
    NetworkState, PessimisticProofOutput,
};

sp1_zkvm::entrypoint!(main);
pub fn main() {
    let raw_data = sp1_zkvm::io::read_vec();
    let initial_state = NetworkState::from_bytes_zero_copy(&raw_data).unwrap();
    
    // Read the full MultiBatchHeader using regular serialization
    // TODO: Optimize this with a hybrid approach:
    // 1. Zero-copy for fixed-size header (origin_network, height, roots, aggchain_proof)
    // 2. read_vec for dynamic data (bridge_exits, imported_bridge_exits, balances_proofs)
    let batch_header = sp1_zkvm::io::read::<MultiBatchHeader<Keccak256Hasher>>();

    let (outputs, _targets) = generate_pessimistic_proof(initial_state, &batch_header).unwrap();

    let pp_inputs = PessimisticProofOutput::bincode_codec()
        .serialize(&outputs)
        .unwrap();

    sp1_zkvm::io::commit_slice(&pp_inputs);
}
