use bytemuck;
pub use pessimistic_proof::PessimisticProofOutput;
use pessimistic_proof::{
    keccak::{Hasher, Keccak256Hasher},
    NetworkState,
};
pub use sp1_sdk::{ExecutionReport, SP1Proof};
use sp1_sdk::{SP1ProofWithPublicValues, SP1PublicValues, SP1Stdin, SP1VerifyingKey};

use crate::PESSIMISTIC_PROOF_ELF;

pub type KeccakHasher = Keccak256Hasher;
pub type Digest = <KeccakHasher as Hasher>::Digest;
pub type MultiBatchHeader = pessimistic_proof::multi_batch_header::MultiBatchHeader<KeccakHasher>;

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

    /// Helper function to serialize zero-copy data and write to stdin
    fn write_zero_copy_data<T: bytemuck::Pod>(stdin: &mut SP1Stdin, data: &[T], name: &str) {
        let bytes = bytemuck::cast_slice(data);
        println!("Writing {}: {} bytes", name, bytes.len());
        stdin.write_vec(bytes.to_vec());
    }

    /// Convert inputs to stdin.
    pub fn prepare_stdin(state: &NetworkState, batch_header: &MultiBatchHeader) -> SP1Stdin {
        let mut stdin = SP1Stdin::new();

        // Use zero-copy for NetworkState
        let zero_copy_bytes = state.to_bytes_zero_copy();
        println!(
            "Writing NetworkState bytes: {} bytes",
            zero_copy_bytes.len()
        );
        stdin.write_vec(zero_copy_bytes);

        // Use zero-copy for MultiBatchHeader header
        let header_zero_copy = batch_header.to_zero_copy();
        let header_bytes = header_zero_copy.to_bytes();
        println!(
            "Writing MultiBatchHeader header: {} bytes",
            header_bytes.len()
        );
        stdin.write_vec(header_bytes);

        // Use zero-copy for bridge_exits
        let bridge_exits_zero_copy: Vec<pessimistic_proof::multi_batch_header::BridgeExitZeroCopy> =
            batch_header
                .bridge_exits
                .iter()
                .map(|be| {
                    pessimistic_proof::multi_batch_header::BridgeExitZeroCopy::from_bridge_exit(be)
                })
                .collect();
        Self::write_zero_copy_data(&mut stdin, &bridge_exits_zero_copy, "bridge_exits");

        // Use zero-copy for imported_bridge_exits
        let imported_bridge_exits_zero_copy: Vec<
            pessimistic_proof::multi_batch_header::ImportedBridgeExitZeroCopy,
        > = batch_header
            .imported_bridge_exits
            .iter()
            .map(|(ibe, _)| {
                pessimistic_proof::multi_batch_header::ImportedBridgeExitZeroCopy::from_imported_bridge_exit(ibe)
            })
            .collect();
        Self::write_zero_copy_data(
            &mut stdin,
            &imported_bridge_exits_zero_copy,
            "imported_bridge_exits",
        );

        // Use zero-copy for imported bridge exit nullifier paths
        let nullifier_paths_zero_copy: Vec<
            pessimistic_proof::multi_batch_header::SmtNonInclusionProofZeroCopy,
        > = batch_header
            .imported_bridge_exits
            .iter()
            .map(|(_, path)| {
                pessimistic_proof::multi_batch_header::SmtNonInclusionProofZeroCopy::from_smt_non_inclusion_proof(path)
            })
            .collect();
        Self::write_zero_copy_data(&mut stdin, &nullifier_paths_zero_copy, "nullifier_paths");

        // Use zero-copy for balances_proofs (TokenInfo + balance amount)
        let balances_proofs_zero_copy: Vec<
            pessimistic_proof::multi_batch_header::BalanceProofEntryZeroCopy,
        > = batch_header
            .balances_proofs
            .iter()
            .map(|(ti, (balance, _))| {
                pessimistic_proof::multi_batch_header::BalanceProofEntryZeroCopy {
                    token_info:
                        pessimistic_proof::multi_batch_header::TokenInfoZeroCopy::from_token_info(
                            ti,
                        ),
                    balance: balance.to_be_bytes(),
                    _padding: [0; 8],
                }
            })
            .collect();
        Self::write_zero_copy_data(&mut stdin, &balances_proofs_zero_copy, "balances_proofs");

        // Use zero-copy for balance Merkle paths (zero-copy)
        let balance_merkle_paths_zero_copy: Vec<
            pessimistic_proof::multi_batch_header::SmtMerkleProofZeroCopy,
        > = batch_header
            .balances_proofs
            .iter()
            .map(|(_, (_, path))| {
                pessimistic_proof::multi_batch_header::SmtMerkleProofZeroCopy::from_smt_merkle_proof(path)
            })
            .collect();
        Self::write_zero_copy_data(
            &mut stdin,
            &balance_merkle_paths_zero_copy,
            "balance_merkle_paths",
        );

        // Write aggchain_proof separately using bincode (since zero-copy truncates it)
        println!("Writing aggchain_proof using bincode");
        stdin.write(&batch_header.aggchain_proof);

        println!("Stdin prepared");
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
