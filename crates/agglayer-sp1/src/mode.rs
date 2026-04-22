use sp1_sdk::SP1Proof;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ProofMode {
    Compressed,
    Plonk,
    Groth16,
}

pub fn proof_mode(proof: &SP1Proof) -> ProofMode {
    match proof {
        SP1Proof::Compressed(_) => ProofMode::Compressed,
        SP1Proof::Plonk(_) => ProofMode::Plonk,
        SP1Proof::Groth16(_) => ProofMode::Groth16,
        // Core proofs aren't used by agglayer today; bucket them with
        // Compressed for envelope purposes and let policy gating catch them.
        SP1Proof::Core(_) => ProofMode::Compressed,
    }
}
