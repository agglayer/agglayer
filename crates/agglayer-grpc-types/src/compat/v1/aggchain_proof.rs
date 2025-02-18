use agglayer_types::{AggchainProof, AggchainProofSP1};
use bincode::Options as _;

use crate::protocol::types::v1;

use super::Error;

fn sp1v4_bincode_options() -> impl bincode::Options {
    bincode::DefaultOptions::new()
        .with_big_endian()
        .with_fixint_encoding()
}

impl TryFrom<v1::AggchainProofSp1v4> for AggchainProofSP1 {
    type Error = Error;

    fn try_from(value: v1::AggchainProofSp1v4) -> Result<Self, Self::Error> {
        Ok(AggchainProofSP1 {
            aggchain_params: required_field!(value, aggchain_params),
            stark_proof: sp1v4_bincode_options()
                .deserialize(&value.stark_proof)
                .map_err(Error::DeserializingSp1v4Proof)?,
        })
    }
}

impl TryFrom<v1::AggchainProof> for AggchainProof {
    type Error = Error;

    fn try_from(value: v1::AggchainProof) -> Result<Self, Self::Error> {
        Ok(match value.proof {
            Some(v1::aggchain_proof::Proof::Signature(signature)) => AggchainProof::ECDSA {
                signature: (&*signature.value)
                    .try_into()
                    .map_err(Error::ParsingSignature)?,
            },
            Some(v1::aggchain_proof::Proof::Sp1StarkV4(sp1_stark_v4)) => AggchainProof::SP1 {
                aggchain_proof: sp1_stark_v4
                    .try_into()
                    .map_err(|e| Error::ParsingField("aggchain_proof", Box::new(e)))?,
            },
            None => return Err(Error::MissingField("proof")),
        })
    }
}
