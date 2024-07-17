pub use pessimistic_proof::{certificate::Certificate, LeafProofOutput, LocalNetworkState};
pub use sp1_sdk::SP1Proof;
use sp1_sdk::SP1Stdin;

/// ELF bytecode binary of the pessimistic proof program
pub const PESSIMISTIC_PROOF_ELF: &[u8] =
    include_bytes!("../../pessimistic-proof-program/elf/riscv32im-succinct-zkvm-elf");

/// A convenient interface to the pessimistic proof
pub struct Client {
    client: sp1_sdk::ProverClient,
    proving_key: sp1_sdk::SP1ProvingKey,
    verifying_key: sp1_sdk::SP1VerifyingKey,
}

impl Client {
    /// A new pessimistic proof client
    pub fn new() -> Self {
        let client = sp1_sdk::ProverClient::new();
        Self::from_client(client)
    }

    /// A new pessimistic proof client from a custom generic client
    pub fn from_client(client: sp1_sdk::ProverClient) -> Self {
        let (proving_key, verifying_key) = client.setup(PESSIMISTIC_PROOF_ELF);

        Self {
            client,
            proving_key,
            verifying_key,
        }
    }

    /// Verify given proof
    pub fn verify(&self, proof: &SP1Proof) -> Result<(), sp1_sdk::SP1VerificationError> {
        self.client.verify(proof, &self.verifying_key)
    }

    /// Create a proof for given inputs
    pub fn prove(
        &self,
        state: &LocalNetworkState,
        cert: &Certificate,
    ) -> anyhow::Result<(SP1Proof, LeafProofOutput)> {
        let stdin = Self::prepare_stdin(state, cert);
        let mut proof = self.client.prove(&self.proving_key, stdin)?;
        self.verify(&proof)?;
        let output = proof.public_values.read();
        Ok((proof, output))
    }

    /// Convert inputs to stdin
    pub fn prepare_stdin(state: &LocalNetworkState, cert: &Certificate) -> SP1Stdin {
        let mut stdin = SP1Stdin::new();
        stdin.write(state);
        stdin.write(cert);
        stdin
    }

    /// Execute the ELF with given inputs
    pub fn execute(
        &self,
        state: &LocalNetworkState,
        cert: &Certificate,
    ) -> anyhow::Result<(LeafProofOutput, ExecutionStats)> {
        let stdin = Self::prepare_stdin(state, cert);
        let (mut public_vals, report) = self.client.execute(PESSIMISTIC_PROOF_ELF, stdin)?;

        let output = public_vals.read();
        let stats = ExecutionStats {
            instructions: report.total_instruction_count(),
            syscalls: report.total_syscall_count(),
        };

        Ok((output, stats))
    }
}

/// Handy statistics about a VM execution
#[derive(PartialEq, Eq, Debug)]
pub struct ExecutionStats {
    /// Number of executed instructions (cycles)
    pub instructions: u64,
    /// Number of executed syscalls
    pub syscalls: u64,
}

impl std::fmt::Display for ExecutionStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            instructions,
            syscalls,
        } = self;
        write!(f, "instructions={instructions}, syscalls={syscalls}")
    }
}
