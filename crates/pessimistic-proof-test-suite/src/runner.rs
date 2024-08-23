pub use pessimistic_proof::{LeafProofOutput, LocalNetworkState};
use reth_primitives::Address;
pub use sp1_sdk::{ExecutionReport, SP1Proof};
use sp1_sdk::{SP1PublicValues, SP1Stdin};

use crate::PESSIMISTIC_PROOF_ELF;

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
    pub fn extract_output(mut public_vals: SP1PublicValues) -> LeafProofOutput {
        // Ignore the first couple of committed values which are taken directly from
        // inputs
        let _exits_root = public_vals.read::<Option<Digest>>();
        let _signer = public_vals.read::<Address>();
        let _selected_ger = public_vals.read::<Digest>();
        let _prev_roots = public_vals.read::<LeafProofOutput>();

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

        let output = Self::extract_output(public_vals);

        Ok((output, report))
    }
}
