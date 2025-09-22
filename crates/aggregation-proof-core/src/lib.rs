use std::collections::BTreeMap;

use agglayer_bincode as bincode;
use agglayer_primitives::Digest;
use pessimistic_proof_core::PessimisticProofOutput;
use serde::{Deserialize, Serialize};
use sha2::{Digest as ShaDigest, Sha256};
use unified_bridge::{MerkleProof, NetworkId};

// /// LUT on the subinclusion of each preconfirmed LERs
// #[derive(Deserialize, Serialize, Default, Debug)]
// pub struct LutPreconfirmedLERs {
//     pub sublet: BTreeMap<NetworkId, Vec<Digest>>,
// }

/// Witness for the aggregation proof.
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct AggregationWitness {
    /// Public values for each proofs.
    pub public_values: Vec<PessimisticProofOutput>,
    /// Pessimistic proof vkey
    pub pp_vkey: [u32; 8],
    // Sub inclusions of preconfirmed LERs
    //   pub sublet: BTreeMap<NetworkId, Vec<MerkleProof>>,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct AggregationPublicValues {
    /// Hash chain on all the pp inputs
    pub hash_chain_pp_inputs: Digest,
    /// Pessimistic proof vkey
    pub pp_vkey: [u32; 8],
}

impl AggregationPublicValues {
    pub fn bincode_codec() -> bincode::Codec<impl bincode::Options> {
        bincode::contracts()
    }
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
        AggregationPublicValues {
            hash_chain_pp_inputs: self.hash_chain_pub_values(),
            pp_vkey: self.pp_vkey,
        }
    }

    /// Hash chain on the PP public values.
    pub fn hash_chain_pub_values(&self) -> Digest {
        Digest::default() // todo
    }
}
