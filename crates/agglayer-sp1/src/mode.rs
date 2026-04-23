use sp1_sdk::SP1Proof;

/// The mode of an SP1 proof. Mirrors the variants of `sp1_sdk::SP1Proof`.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ProofMode {
    Core,
    Compressed,
    Plonk,
    Groth16,
}

impl From<&SP1Proof> for ProofMode {
    fn from(proof: &SP1Proof) -> Self {
        match proof {
            SP1Proof::Core(_) => ProofMode::Core,
            SP1Proof::Compressed(_) => ProofMode::Compressed,
            SP1Proof::Plonk(_) => ProofMode::Plonk,
            SP1Proof::Groth16(_) => ProofMode::Groth16,
        }
    }
}
