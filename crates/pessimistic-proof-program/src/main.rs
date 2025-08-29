#![no_main]

use pessimistic_proof_core::{
    generate_pessimistic_proof, multi_batch_header::MultiBatchHeader, NetworkState,
};

sp1_zkvm::entrypoint!(main);
pub fn main() {
    // Read NetworkState (zero-copy)
    let network_state_bytes = sp1_zkvm::io::read_vec();
    let initial_state = NetworkState::try_from(network_state_bytes.as_slice())
        .expect("Failed to deserialize NetworkState");

    // Read all zero-copy components
    let header_bytes = sp1_zkvm::io::read_vec();
    let bridge_exits_bytes = sp1_zkvm::io::read_vec();
    let imported_bridge_exits_bytes = sp1_zkvm::io::read_vec();
    let nullifier_paths_bytes = sp1_zkvm::io::read_vec();
    let balances_proofs_bytes = sp1_zkvm::io::read_vec();
    let balance_merkle_paths_bytes = sp1_zkvm::io::read_vec();
    let multisig_signatures_bytes = sp1_zkvm::io::read_vec();
    let multisig_expected_signers_bytes = sp1_zkvm::io::read_vec();

    // Reconstruct the MultiBatchHeaderRef from zero-copy components using the
    // helper function
    let batch_header_ref = MultiBatchHeader::from_zero_copy_components(
        &header_bytes,
        &bridge_exits_bytes,
        &imported_bridge_exits_bytes,
        &nullifier_paths_bytes,
        &balances_proofs_bytes,
        &balance_merkle_paths_bytes,
        &multisig_signatures_bytes,
        &multisig_expected_signers_bytes,
    )
    .expect("Failed to reconstruct MultiBatchHeaderRef");

    // Convert to owned MultiBatchHeader for the proof generation
    let batch_header = batch_header_ref
        .to_owned()
        .expect("Failed to convert MultiBatchHeaderRef to owned");

    let (outputs, _targets) = generate_pessimistic_proof(initial_state, &batch_header)
        .expect("Failed to generate pessimistic proof");

    let pp_inputs = outputs.to_bytes_zero_copy();

    sp1_zkvm::io::commit_slice(&pp_inputs);
}
