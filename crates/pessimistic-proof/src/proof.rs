use std::collections::HashMap;

use crate::{
    bridge_exit::NetworkId,
    certificate::Certificate,
    keccak::Digest,
    local_balance_tree::{merge_balance_trees, BalanceTreeByNetwork},
};

/// Represents all errors that can occur while generating the proof.
#[derive(Debug)]
pub enum ProofError {
    InvalidLocalExitRoot { got: Digest, expected: Digest },
    NotEnoughBalance { debtors: Vec<NetworkId> },
}

pub type ExitRoot = Digest;
pub type BalanceRoot = Digest;
pub type FullProofOutput = (HashMap<NetworkId, ExitRoot>, HashMap<NetworkId, BalanceRoot>);

/// Returns the updated local balance and exit roots for each network.
pub fn generate_full_proof(certificates: &[Certificate]) -> Result<FullProofOutput, ProofError> {
    // Check the validity of the provided exit roots
    for certificate in certificates {
        let computed_root = certificate.prev_local_exit_tree.get_root();

        if computed_root != certificate.prev_local_exit_root {
            return Err(ProofError::InvalidLocalExitRoot {
                got: computed_root,
                expected: certificate.prev_local_exit_root,
            });
        }
    }

    // Compute the new exit root
    let exit_roots: HashMap<NetworkId, ExitRoot> = certificates
        .iter()
        .map(|certificate| (certificate.origin_network, certificate.compute_new_exit_root()))
        .collect();

    // Compute the new balance tree by network
    let balance_trees: HashMap<NetworkId, BalanceTreeByNetwork> = certificates
        .iter()
        .map(|certificate| (certificate.origin_network, certificate.compute_new_balance_tree()))
        .collect();

    // Merge the balance tree by network
    let balance_tree_by_network: BalanceTreeByNetwork = merge_balance_trees(&balance_trees);

    // Detect the debtors if any
    let debtors = balance_tree_by_network
        .iter()
        .filter_map(|(network, balance_tree)| balance_tree.has_debt().then(|| *network))
        .collect::<Vec<_>>();

    if !debtors.is_empty() {
        return Err(ProofError::NotEnoughBalance { debtors });
    }

    let balance_roots: HashMap<NetworkId, BalanceRoot> = balance_tree_by_network
        .iter()
        .map(|(network, balance_tree)| (*network, balance_tree.hash()))
        .collect();

    Ok((exit_roots, balance_roots))
}
