#![no_main]

use pessimistic_proof::local_exit_tree::hasher::Keccak256Hasher;
use pessimistic_proof::multi_batch_header::MultiBatchHeader;
use pessimistic_proof::{generate_leaf_proof, LocalNetworkState};

sp1_zkvm::entrypoint!(main);

pub fn main() {
    let initial_state = sp1_zkvm::io::read::<LocalNetworkState>();
    let batch_header = sp1_zkvm::io::read::<MultiBatchHeader<Keccak256Hasher>>();

    let prev_roots = initial_state.roots();
    let new_roots = generate_leaf_proof(initial_state, &batch_header).unwrap();

    //TODO: only necessary to expose a commitment to the imported_lers as a public
    // input, though maybe SP1 does that for us
    for (network, ler) in &batch_header.imported_local_exit_roots {
        sp1_zkvm::io::commit(&(network, ler));
    }

    sp1_zkvm::io::commit(&batch_header.imported_exits_root);
    sp1_zkvm::io::commit(&prev_roots);
    sp1_zkvm::io::commit(&new_roots);
}
