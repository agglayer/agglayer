#![no_main]

use pessimistic_proof::{batch_header::BatchHeader, generate_leaf_proof, LocalNetworkState};

sp1_zkvm::entrypoint!(main);

pub fn main() {
    let initial_state = sp1_zkvm::io::read::<LocalNetworkState>();
    let batch_header = sp1_zkvm::io::read::<BatchHeader>();

    let new_roots = generate_leaf_proof(initial_state, &batch_header).unwrap();

    if let Some(imported_roots) = &batch_header.imported_lers_root {
        sp1_zkvm::io::commit(imported_roots);
    }else {
        sp1_zkvm::io::commit(None);
    }

    if let Some(imported_global_root) = &batch_header.imported_global_exit_root {
        sp1_zkvm::io::commit(imported_global_root);
    }else {
        sp1_zkvm::io::commit(None);
    }

    if let Some(imported_exits_root) = &batch_header.imported_exits_root {
        sp1_zkvm::io::commit(imported_exits_root);
    }else {
        sp1_zkvm::io::commit(None);
    }

    sp1_zkvm::io::commit(&new_roots);
}
