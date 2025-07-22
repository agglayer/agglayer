use agglayer_primitives::Digest;
use pessimistic_proof_core::PessimisticProofOutput;
use serde::{Deserialize, Serialize};
use sha2::{Digest as ShaDigest, Sha256};

/// Witness for the aggregation proof.
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct AggregationWitness {
    /// Public values for each proofs.
    pub public_values: Vec<PessimisticProofOutput>,
    /// Pessimistic proof vkey
    pub pp_vkey: [u32; 8],
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct AggregationPublicValues {
    /// Hash chain on all the pp inputs
    pub hash_chain_pp_inputs: Digest,
    /// Pessimistic proof vkey
    pub pp_vkey: [u32; 8],
}

impl AggregationWitness {
    /// Verify the aggregation of all the pessimistic proofs.
    pub fn verify_aggregation(&self) -> Result<AggregationPublicValues, ()> {
        for pv in &self.public_values {
            let pv_serialized = PessimisticProofOutput::bincode_codec()
                .serialize(&pv)
                .unwrap();
            let pv_digest = Sha256::digest(pv_serialized);
            sp1_zkvm::lib::verify::verify_sp1_proof(&self.pp_vkey, &pv_digest.into());
        }

        Ok(self.public_values())
    }

    /// Computes and returns the public values.
    pub fn public_values(&self) -> AggregationPublicValues {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert!(true);
    }
}
