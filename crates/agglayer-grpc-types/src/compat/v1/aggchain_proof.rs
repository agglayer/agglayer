use std::collections::HashMap;

use agglayer_types::aggchain_proof::{AggchainData, Proof};
use bincode::Options as _;
use prost::bytes::Bytes;

use super::Error;
use crate::protocol::types::v1;

fn sp1v4_bincode_options() -> impl bincode::Options {
    bincode::DefaultOptions::new()
        .with_big_endian()
        .with_fixint_encoding()
}

impl TryFrom<v1::AggchainData> for AggchainData {
    type Error = Error;

    fn try_from(value: v1::AggchainData) -> Result<Self, Self::Error> {
        Ok(match value.data {
            Some(v1::aggchain_data::Data::Signature(signature)) => AggchainData::ECDSA {
                signature: (&*signature.value)
                    .try_into()
                    .map_err(Error::ParsingSignature)?,
            },
            Some(v1::aggchain_data::Data::Sp1v4(sp1_v4)) => AggchainData::Generic {
                aggchain_params: required_field!(sp1_v4, aggchain_params),
                proof: Proof::SP1Stark(Box::new(
                    sp1v4_bincode_options()
                        .deserialize(&sp1_v4.proof)
                        .map_err(Error::DeserializingSp1v4Proof)?,
                )),
            },
            None => return Err(Error::MissingField("data")),
        })
    }
}

impl TryFrom<AggchainData> for v1::AggchainData {
    type Error = Error;

    fn try_from(value: AggchainData) -> Result<Self, Self::Error> {
        Ok(v1::AggchainData {
            context: HashMap::new(),
            data: Some(match value {
                AggchainData::ECDSA { signature } => {
                    v1::aggchain_data::Data::Signature(v1::FixedBytes65 {
                        value: Bytes::copy_from_slice(&signature.as_bytes()),
                    })
                }
                AggchainData::Generic {
                    proof,
                    aggchain_params,
                } => v1::aggchain_data::Data::Sp1v4(v1::AggchainProofSp1v4 {
                    aggchain_params: Some(aggchain_params.into()),
                    proof: sp1v4_bincode_options()
                        .serialize(&proof)
                        .map_err(Error::SerializingSp1v4Proof)?
                        .into(),
                }),
            }),
        })
    }
}
