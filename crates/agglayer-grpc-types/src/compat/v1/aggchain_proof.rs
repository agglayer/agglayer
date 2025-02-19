use agglayer_types::{AggchainProof, AggchainProofSP1};
use bincode::Options as _;
use prost::bytes::Bytes;

use super::Error;
use crate::protocol::types::v1;

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

impl TryFrom<AggchainProofSP1> for v1::AggchainProofSp1v4 {
    type Error = Error;

    fn try_from(value: AggchainProofSP1) -> Result<Self, Self::Error> {
        Ok(v1::AggchainProofSp1v4 {
            aggchain_params: Some(value.aggchain_params.into()),
            stark_proof: sp1v4_bincode_options()
                .serialize(&value.stark_proof)
                .map_err(Error::SerializingSp1v4Proof)?
                .into(),
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

impl TryFrom<AggchainProof> for v1::AggchainProof {
    type Error = Error;

    fn try_from(value: AggchainProof) -> Result<Self, Self::Error> {
        Ok(match value {
            AggchainProof::ECDSA { signature } => v1::AggchainProof {
                proof: Some(v1::aggchain_proof::Proof::Signature(v1::FixedBytes65 {
                    value: Bytes::copy_from_slice(&signature.as_bytes()),
                })),
            },
            AggchainProof::SP1 { aggchain_proof } => v1::AggchainProof {
                proof: Some(v1::aggchain_proof::Proof::Sp1StarkV4(
                    aggchain_proof.try_into()?,
                )),
            },
        })
    }
}
