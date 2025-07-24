#![no_main]

use pessimistic_proof_core::{
    generate_pessimistic_proof, keccak::Keccak256Hasher, multi_batch_header::MultiBatchHeader,
    NetworkState, PessimisticProofOutput,
};
sp1_zkvm::entrypoint!(main);

pub fn main() {
    // Read NetworkState (zero-copy)
    let network_state_bytes = sp1_zkvm::io::read_vec();
    let initial_state = NetworkState::from_bytes_zero_copy(&network_state_bytes)
        .expect("Failed to deserialize NetworkState");

    // Read all zero-copy components
    let header_bytes = sp1_zkvm::io::read_vec();
    let bridge_exits_bytes = sp1_zkvm::io::read_vec();
    let imported_bridge_exits_bytes = sp1_zkvm::io::read_vec();
    let nullifier_paths_bytes = sp1_zkvm::io::read_vec();
    let balances_proofs_bytes = sp1_zkvm::io::read_vec();
    let balance_merkle_paths_bytes = sp1_zkvm::io::read_vec();

    // Read aggchain_proof separately using bincode (since zero-copy truncates it)
    let aggchain_proof =
        sp1_zkvm::io::read::<pessimistic_proof_core::aggchain_proof::AggchainData>();

    // Reconstruct the MultiBatchHeaderRef from zero-copy components using the
    // helper function
    let batch_header_ref = MultiBatchHeader::<Keccak256Hasher>::from_zero_copy_components(
        &header_bytes,
        &bridge_exits_bytes,
        &imported_bridge_exits_bytes,
        &nullifier_paths_bytes,
        &balances_proofs_bytes,
        &balance_merkle_paths_bytes,
        aggchain_proof,
    )
    .expect("Failed to reconstruct MultiBatchHeaderRef");

    // Convert to owned MultiBatchHeader for the proof generation
    let batch_header = batch_header_ref
        .to_owned_keccak()
        .expect("Failed to convert MultiBatchHeaderRef to owned");

    let (outputs, _targets) = generate_pessimistic_proof(initial_state, &batch_header)
        .expect("Failed to generate pessimistic proof");

    let pp_inputs = PessimisticProofOutput::bincode_codec()
        .serialize(&outputs)
        .expect("Failed to serialize proof outputs");

    sp1_zkvm::io::commit_slice(&pp_inputs);
}
