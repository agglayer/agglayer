#![no_main]
ziskos::entrypoint!(main);

use bincode::Options;
use pessimistic_proof::local_exit_tree::hasher::Keccak256Hasher;
use pessimistic_proof::multi_batch_header::MultiBatchHeader;
use pessimistic_proof::{generate_pessimistic_proof, LocalNetworkState, PessimisticProofOutput, PessimisticProofInput};
use ziskos::{read_input, write_output};

pub fn main() { 
    // Get the input slice from ziskos
    let input  = read_input();
    let input = bincode::deserialize::<PessimisticProofInput>(&input).unwrap();

    // let initial_state = sp1_zkvm::io::read::<LocalNetworkState>();
    // let batch_header = sp1_zkvm::io::read::<MultiBatchHeader<Keccak256Hasher>>();

    let outputs = generate_pessimistic_proof(input.initial_state, &input.batch_header).unwrap();

    let pp_inputs = PessimisticProofOutput::bincode_options()
        .serialize(&outputs)
        .unwrap();

    //write_output(&pp_inputs);
}
