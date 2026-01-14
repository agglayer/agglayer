use eyre::eyre;
use pessimistic_proof::NetworkState;
pub use pessimistic_proof::{multi_batch_header::MultiBatchHeader, PessimisticProofOutput};
pub use sp1_sdk::{ExecutionReport, SP1Proof};
use sp1_sdk::{SP1ProofWithPublicValues, SP1PublicValues, SP1Stdin, SP1VerifyingKey};

use crate::PESSIMISTIC_PROOF_ELF;

pub struct ProofOutput {}

fn write_component(stdin: &mut SP1Stdin, mut bytes: Vec<u8>) {
    if bytes.is_empty() {
        bytes.push(0);
    }
    stdin.write_vec(bytes);
}

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

        // Use zero-copy serialization for NetworkState
        let network_state_bytes = state.to_bytes_zero_copy();
        stdin.write_vec(network_state_bytes);

        let components = batch_header
            .to_zero_copy_components()
            .expect("zero-copy MultiBatchHeader");
        stdin.write_vec(components.header_bytes);
        write_component(&mut stdin, components.bridge_exits_bytes);
        write_component(&mut stdin, components.imported_bridge_exits_bytes);
        write_component(&mut stdin, components.nullifier_paths_bytes);
        write_component(&mut stdin, components.balances_proofs_bytes);
        write_component(&mut stdin, components.balance_merkle_paths_bytes);
        write_component(&mut stdin, components.multisig_signatures_bytes);
        write_component(&mut stdin, components.multisig_expected_signers_bytes);
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
            .execute(PESSIMISTIC_PROOF_ELF, &stdin)
            .run()
            .map_err(|e| eyre!(e))?;

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
    ) -> eyre::Result<(
        SP1ProofWithPublicValues,
        SP1VerifyingKey,
        PessimisticProofOutput,
    )> {
        let stdin = Self::prepare_stdin(state, batch_header);
        let (pk, vk) = self.client.setup(PESSIMISTIC_PROOF_ELF);

        let proof = self
            .client
            .prove(&pk, &stdin)
            .plonk()
            .run()
            .map_err(|e| eyre!(e))?;
        let output = Self::extract_output(proof.public_values.clone());

        Ok((proof, vk, output))
    }
}
