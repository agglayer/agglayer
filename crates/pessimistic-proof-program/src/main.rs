#![no_main]

use pessimistic_proof_core::{
    generate_pessimistic_proof, keccak::Keccak256Hasher, multi_batch_header::{
        MultiBatchHeader, MultiBatchHeaderZeroCopy, BridgeExitZeroCopy
    },
    NetworkState, PessimisticProofOutput,
};
use unified_bridge;
use agglayer_primitives;
use agglayer_tries;
sp1_zkvm::entrypoint!(main);
pub fn main() {
    let raw_data = sp1_zkvm::io::read_vec();
    let initial_state = NetworkState::from_bytes_zero_copy(&raw_data).unwrap();
    let batch_header = sp1_zkvm::io::read::<MultiBatchHeader<Keccak256Hasher>>();
    


    let (outputs, _targets) = generate_pessimistic_proof(initial_state, &batch_header).unwrap();

    let pp_inputs = PessimisticProofOutput::bincode_codec()
        .serialize(&outputs)
        .unwrap();

    sp1_zkvm::io::commit_slice(&pp_inputs);
}
