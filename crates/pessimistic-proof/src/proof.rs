pub use pessimistic_proof_core::PessimisticProofOutput;
#[cfg(any(test, feature = "testutils"))]
use pessimistic_proof_core::{
    local_exit_tree::hasher::Keccak256Hasher, multi_batch_header::MultiBatchHeader, NetworkState,
};
use serde::{Deserialize, Serialize};
#[cfg(any(test, feature = "testutils"))]
use sp1_sdk::{Prover, ProverClient, SP1Stdin};
use sp1_sdk::{SP1Proof, SP1ProofWithPublicValues, SP1PublicValues};

#[cfg(any(test, feature = "testutils"))]
use crate::ELF;

pub trait DisplayToHex {
    fn display_to_hex(&self) -> String;
}

impl DisplayToHex for PessimisticProofOutput {
    fn display_to_hex(&self) -> String {
        format!(
            "prev_local_exit_root: {}, prev_pessimistic_root: {}, l1_info_root: {}, \
             origin_network: {}, aggchain_hash: {}, new_local_exit_root: {}, \
             new_pessimistic_root: {}",
            self.prev_local_exit_root,
            self.prev_pessimistic_root,
            self.l1_info_root,
            self.origin_network,
            self.aggchain_hash,
            self.new_local_exit_root,
            self.new_pessimistic_root,
        )
    }
}

/// Proof is a wrapper around all the different types of proofs that can be
/// generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Proof {
    SP1(SP1ProofWithPublicValues),
}

impl Proof {
    pub fn dummy() -> Self {
        Self::SP1(SP1ProofWithPublicValues {
            proof: SP1Proof::Core(vec![]),
            public_values: SP1PublicValues::new(),
            sp1_version: "".to_string(),
        })
    }

    #[cfg(any(test, feature = "testutils"))]
    pub fn new_for_test(
        state: &NetworkState,
        multi_batch_header: &MultiBatchHeader<Keccak256Hasher>,
    ) -> Self {
        let mock = ProverClient::builder().mock().build();
        let (p, _v) = mock.setup(ELF);

        let mut stdin = SP1Stdin::new();
        stdin.write(state);
        stdin.write(multi_batch_header);

        let proof = mock.prove(&p, &stdin).plonk().run().unwrap();

        Proof::SP1(proof)
    }
}

#[cfg(test)]
mod tests {
    use pessimistic_proof_core::proof::EMPTY_LER;

    use crate::local_state::LocalNetworkState;

    #[test]
    fn empty_tree_roots() {
        let empty_state = LocalNetworkState::default();

        let ler = empty_state.exit_tree.get_root();

        assert_eq!(EMPTY_LER, ler);
    }
}
