pub use pessimistic_proof_core::PessimisticProofOutput;

pub trait DisplayToHex {
    fn display_to_hex(&self) -> String;
}

impl DisplayToHex for PessimisticProofOutput {
    fn display_to_hex(&self) -> String {
        format!(
            "prev_local_exit_root: {}, prev_pessimistic_root: {}, l1_info_root: {}, \
             origin_network: {}, consensus_hash: {}, new_local_exit_root: {}, \
             new_pessimistic_root: {}",
            self.prev_local_exit_root,
            self.prev_pessimistic_root,
            self.l1_info_root,
            self.origin_network,
            self.consensus_hash,
            self.new_local_exit_root,
            self.new_pessimistic_root,
        )
    }
}

#[cfg(test)]
mod tests {
    use pessimistic_proof_core::{
        keccak::keccak256_combine,
        proof::{EMPTY_LER, EMPTY_PP_ROOT},
    };

    use crate::local_state::LocalNetworkState;

    #[test]
    fn empty_tree_roots() {
        let empty_state = LocalNetworkState::default();

        let ler = empty_state.exit_tree.get_root();
        let ppr = keccak256_combine([
            empty_state.balance_tree.root.as_slice(),
            empty_state.nullifier_tree.root.as_slice(),
            empty_state.exit_tree.leaf_count().to_le_bytes().as_slice(),
        ]);

        assert_eq!(EMPTY_LER, ler);
        assert_eq!(EMPTY_PP_ROOT, ppr);
    }
}
