use eyre::eyre;
use pessimistic_proof::NetworkState;
pub use pessimistic_proof::{multi_batch_header::MultiBatchHeader, PessimisticProofOutput};
use sp1_sdk::blocking::{EnvProver, ProveRequest, Prover, ProverClient};
use sp1_sdk::{
    Elf, ProvingKey, SP1ProofWithPublicValues, SP1PublicValues, SP1Stdin, SP1VerifyingKey,
};
pub use sp1_sdk::{ExecutionReport, SP1Proof};

use crate::PESSIMISTIC_PROOF_ELF;

pub struct ProofOutput {}

/// A convenient interface to run the pessimistic proof ELF bytecode.
pub struct Runner {
    client: EnvProver,
}

impl Default for Runner {
    fn default() -> Self {
        Self::new()
    }
}

impl Runner {
    /// Create a new pessimistic proof client.
    pub fn new() -> Self {
        Self::from_client(ProverClient::from_env())
    }

    /// Create a new pessimistic proof client from a custom generic client.
    pub fn from_client(client: EnvProver) -> Self {
        Self { client }
    }

    /// Convert inputs to stdin.
    pub fn prepare_stdin(state: &NetworkState, batch_header: &MultiBatchHeader) -> SP1Stdin {
        let mut stdin = SP1Stdin::new();
        stdin.write(state);
        stdin.write(batch_header);
        stdin
    }

    /// Extract outputs from the committed public values.
    pub fn extract_output(public_vals: SP1PublicValues) -> PessimisticProofOutput {
        PessimisticProofOutput::bincode_codec()
            .deserialize(public_vals.as_slice())
            .expect("deser")
    }

    /// Execute the ELF with given inputs.
    pub fn execute(
        &self,
        state: &NetworkState,
        batch_header: &MultiBatchHeader,
    ) -> eyre::Result<(PessimisticProofOutput, ExecutionReport)> {
        let stdin = Self::prepare_stdin(state, batch_header);
        let (public_vals, report) = self
            .client
            .execute(Elf::Static(PESSIMISTIC_PROOF_ELF), stdin)
            .run()
            .map_err(|e| eyre!(e))?;

        let output = Self::extract_output(public_vals);

        Ok((output, report))
    }

    pub fn get_vkey(&self) -> SP1VerifyingKey {
        self.client
            .setup(Elf::Static(PESSIMISTIC_PROOF_ELF))
            .unwrap()
            .verifying_key()
            .clone()
    }

    /// Generate one plonk proof.
    pub fn generate_plonk_proof(
        &self,
        state: &NetworkState,
        batch_header: &MultiBatchHeader,
    ) -> eyre::Result<(
        SP1ProofWithPublicValues,
        SP1VerifyingKey,
        PessimisticProofOutput,
    )> {
        let stdin = Self::prepare_stdin(state, batch_header);
        let pk = self
            .client
            .setup(Elf::Static(PESSIMISTIC_PROOF_ELF))
            .map_err(|e| eyre!(e))?;
        let vk = pk.verifying_key().clone();

        let proof = self
            .client
            .prove(&pk, stdin)
            .plonk()
            .run()
            .map_err(|e| eyre!(e))?;
        let output = Self::extract_output(proof.public_values.clone());

        Ok((proof, vk, output))
    }
}
