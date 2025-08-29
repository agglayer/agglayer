use pessimistic_proof::NetworkState;
pub use pessimistic_proof::{multi_batch_header::MultiBatchHeader, PessimisticProofOutput};
pub use sp1_sdk::{ExecutionReport, SP1Proof};
use sp1_sdk::{SP1ProofWithPublicValues, SP1PublicValues, SP1Stdin, SP1VerifyingKey};

use crate::PESSIMISTIC_PROOF_ELF;

pub struct ProofOutput {}

/// A convenient interface to run the pessimistic proof ELF bytecode.
pub struct Runner {
    client: sp1_sdk::EnvProver,
}

impl Default for Runner {
    fn default() -> Self {
        Self::new()
    }
}

impl Runner {
    /// Create a new pessimistic proof client.
    pub fn new() -> Self {
        Self::from_client(sp1_sdk::ProverClient::from_env())
    }

    /// Create a new pessimistic proof client from a custom generic client.
    pub fn from_client(client: sp1_sdk::EnvProver) -> Self {
        Self { client }
    }

    /// Convert inputs to stdin.
    pub fn prepare_stdin(state: &NetworkState, batch_header: &MultiBatchHeader) -> SP1Stdin {
        let mut stdin = SP1Stdin::new();

        // Use zero-copy for NetworkState
        let zero_copy_bytes = state.to_bytes_zero_copy();
        stdin.write_vec(zero_copy_bytes);

        // Use the new helper function to get all zero-copy components
        let components = batch_header
            .to_zero_copy_components()
            .expect("Failed to convert to zero-copy components");

        // Write all components to stdin
        stdin.write_vec(components.header_bytes);

        // Write zero-copy components directly as bytes
        stdin.write_vec(components.bridge_exits_bytes);
        stdin.write_vec(components.imported_bridge_exits_bytes);
        stdin.write_vec(components.nullifier_paths_bytes);
        stdin.write_vec(components.balances_proofs_bytes);
        stdin.write_vec(components.balance_merkle_paths_bytes);
        stdin.write_vec(components.multisig_signatures_bytes);
        stdin.write_vec(components.multisig_expected_signers_bytes);
        stdin
    }

    /// Extract outputs from the committed public values.
    /// Updated to use zero-copy deserialization for better performance.
    pub fn extract_output(public_vals: SP1PublicValues) -> PessimisticProofOutput {
        PessimisticProofOutput::from_bytes_zero_copy(public_vals.as_slice())
            .expect("Failed to deserialize PessimisticProofOutput from zero-copy bytes")
    }

    /// Execute the ELF with given inputs.
    pub fn execute(
        &self,
        state: &NetworkState,
        batch_header: &MultiBatchHeader,
    ) -> anyhow::Result<(PessimisticProofOutput, ExecutionReport)> {
        let stdin = Self::prepare_stdin(state, batch_header);
        let (public_vals, report) = self.client.execute(PESSIMISTIC_PROOF_ELF, &stdin).run()?;

        let output = Self::extract_output(public_vals);

        Ok((output, report))
    }

    pub fn get_vkey(&self) -> SP1VerifyingKey {
        let (_pk, vk) = self.client.setup(PESSIMISTIC_PROOF_ELF);
        vk
    }

    /// Generate one plonk proof.
    pub fn generate_plonk_proof(
        &self,
        state: &NetworkState,
        batch_header: &MultiBatchHeader,
    ) -> anyhow::Result<(
        SP1ProofWithPublicValues,
        SP1VerifyingKey,
        PessimisticProofOutput,
    )> {
        let stdin = Self::prepare_stdin(state, batch_header);
        let (pk, vk) = self.client.setup(PESSIMISTIC_PROOF_ELF);

        let proof = self.client.prove(&pk, &stdin).plonk().run()?;
        let output = Self::extract_output(proof.public_values.clone());

        Ok((proof, vk, output))
    }
}
