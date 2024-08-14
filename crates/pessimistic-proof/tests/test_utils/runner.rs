use pessimistic_proof::bridge_exit::NetworkId;
pub use pessimistic_proof::{LeafProofOutput, LocalNetworkState};
pub use sp1_core::runtime::ExecutionReport;
pub use sp1_sdk::SP1Proof;
use sp1_sdk::{SP1PublicValues, SP1Stdin};

use crate::test_utils::PESSIMISTIC_PROOF_ELF;

pub type Hasher = pessimistic_proof::local_exit_tree::hasher::Keccak256Hasher;
pub type Digest = <Hasher as pessimistic_proof::local_exit_tree::hasher::Hasher>::Digest;
pub type MultiBatchHeader = pessimistic_proof::multi_batch_header::MultiBatchHeader<Hasher>;

pub struct ProofOutput {}

/// A convenient interface to run the pessimistic proof ELF bytecode.
pub struct Runner {
    client: sp1_sdk::ProverClient,
}

impl Runner {
    /// Create a new pessimistic proof client.
    pub fn new() -> Self {
        Self::from_client(sp1_sdk::ProverClient::new())
    }

    /// Create a new pessimistic proof client from a custom generic client.
    pub fn from_client(client: sp1_sdk::ProverClient) -> Self {
        Self { client }
    }

    /// Convert inputs to stdin.
    pub fn prepare_stdin(state: &LocalNetworkState, batch_header: &MultiBatchHeader) -> SP1Stdin {
        let mut stdin = SP1Stdin::new();
        stdin.write(state);
        stdin.write(batch_header);
        stdin
    }

    /// Extract outputs from the committed public values.
    pub fn extract_output(
        mut public_vals: SP1PublicValues,
        num_imported_local_exit_roots: usize,
    ) -> LeafProofOutput {
        // Ignore the first couple of committed values which are taken directly from inputs
        for _ in 0..num_imported_local_exit_roots {
            let _ = public_vals.read::<(NetworkId, Digest)>();
        }
        let _ = public_vals.read::<Option<Digest>>();
        let _ = public_vals.read::<LeafProofOutput>();

        public_vals.read::<LeafProofOutput>()
    }

    /// Execute the ELF with given inputs.
    pub fn execute(
        &self,
        state: &LocalNetworkState,
        batch_header: &MultiBatchHeader,
    ) -> anyhow::Result<(LeafProofOutput, ExecutionReport)> {
        let stdin = Self::prepare_stdin(state, batch_header);
        let (public_vals, report) = self.client.execute(PESSIMISTIC_PROOF_ELF, stdin).run()?;

        let output =
            Self::extract_output(public_vals, batch_header.imported_local_exit_roots.len());

        Ok((output, report))
    }
}
