use sp1_sdk::SP1Proof;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ProofMode {
    Core,
    Compressed,
    Plonk,
    Groth16,
}

pub fn proof_mode(proof: &SP1Proof) -> ProofMode {
    match proof {
        SP1Proof::Core(_) => ProofMode::Core,
        SP1Proof::Compressed(_) => ProofMode::Compressed,
        SP1Proof::Plonk(_) => ProofMode::Plonk,
        SP1Proof::Groth16(_) => ProofMode::Groth16,
    }
}
