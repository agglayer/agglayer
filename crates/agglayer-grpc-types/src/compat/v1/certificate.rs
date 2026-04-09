use std::collections::HashMap;

use agglayer_types::{
    aggchain_proof::{
        AggchainData, AggchainProof, MultisigPayload, Proof, SP1StarkWithContext,
        SP1StarkWithContextExt as _,
    },
    bincode, Certificate, Height, Metadata, NetworkId,
};
use prost::bytes::Bytes;

use super::Error;
use crate::node::types::v1;

/// Maximum number of signers allowed in a multisig payload.
const MAX_SIGNERS: usize = 1024;

fn parse_sp1_proof(value: agglayer_interop::grpc::v1::Sp1StarkProof) -> Result<Proof, Error> {
    let proof = SP1StarkWithContext {
        version: value.version,
        proof: value.proof.to_vec(),
        vkey: value.vkey.to_vec(),
    };
    proof
        .ensure_readable()
        .map_err(|err| Error::invalid_data(err.to_string()))?;
    Ok(Proof::SP1Stark(proof))
}

fn parse_proof(value: agglayer_interop::grpc::v1::aggchain_proof::Proof) -> Result<Proof, Error> {
    match value {
        agglayer_interop::grpc::v1::aggchain_proof::Proof::Sp1Stark(proof) => {
            parse_sp1_proof(proof)
        }
    }
}

fn parse_public_values(
    context: &HashMap<String, Bytes>,
) -> Result<Option<Box<agglayer_types::aggchain_proof::AggchainProofPublicValues>>, Error> {
    context
        .get("public_values")
        .map(|bytes| {
            std::panic::catch_unwind(|| bincode::default().deserialize(bytes.as_ref()))
                .map_err(|_| {
                    Error::deserializing_aggchain_proof_public_values(Box::new(
                        bincode::ErrorKind::Custom(String::from("panic")),
                    ))
                })?
                .map_err(Error::deserializing_aggchain_proof_public_values)
                .map(Box::new)
        })
        .transpose()
}

fn parse_multisig(
    multisig: agglayer_interop::grpc::v1::Multisig,
) -> Result<MultisigPayload, Error> {
    match multisig.data {
        Some(agglayer_interop::grpc::v1::multisig::Data::Ecdsa(
            agglayer_interop::grpc::v1::EcdsaMultisig { signatures },
        )) => {
            if signatures.is_empty() {
                return Err(Error::invalid_data(
                    "Multisig ECDSA doesn't have any signature".to_string(),
                ));
            }

            let required_len = signatures
                .iter()
                .map(|entry| entry.index + 1)
                .max()
                .unwrap_or(0);

            if required_len as usize > MAX_SIGNERS {
                return Err(Error::invalid_data(format!(
                    "Multisig ECDSA has too many signers: {required_len} (max {MAX_SIGNERS})",
                )));
            }

            let mut result: Vec<Option<_>> = vec![None; required_len as usize];

            for entry in signatures {
                let index = entry.index as usize;
                if let Some(fixed_bytes) = entry.signature {
                    let signature = (&*fixed_bytes.value)
                        .try_into()
                        .map_err(Error::parsing_signature)?;

                    if result[index].is_some() {
                        return Err(Error::invalid_data(format!(
                            "Duplicate signature at index {index}",
                        )));
                    }

                    result[index] = Some(signature);
                }
            }

            Ok(MultisigPayload(result))
        }
        None => Err(Error::missing_field("data")),
    }
}

fn parse_aggchain_proof(
    value: agglayer_interop::grpc::v1::AggchainProof,
) -> Result<AggchainProof, Error> {
    Ok(AggchainProof {
        proof: parse_proof(value.proof.ok_or(Error::missing_field("proof"))?)
            .map_err(|e| e.inside_field("proof"))?,
        aggchain_params: value
            .aggchain_params
            .ok_or(Error::missing_field("aggchain_params"))?
            .try_into()
            .map_err(|e: Error| e.inside_field("aggchain_params"))?,
        public_values: parse_public_values(&value.context)?,
    })
}

fn parse_aggchain_data(
    value: agglayer_interop::grpc::v1::AggchainData,
) -> Result<AggchainData, Error> {
    Ok(match value.data {
        Some(agglayer_interop::grpc::v1::aggchain_data::Data::Signature(signature)) => {
            AggchainData::ECDSA {
                signature: (&*signature.value)
                    .try_into()
                    .map_err(Error::parsing_signature)?,
            }
        }
        Some(agglayer_interop::grpc::v1::aggchain_data::Data::Generic(aggchain_proof)) => {
            let signature = aggchain_proof
                .signature
                .as_ref()
                .map(|signature| {
                    (&*signature.value)
                        .try_into()
                        .map_err(Error::parsing_signature)
                })
                .transpose()?
                .map(Box::new);

            let AggchainProof {
                proof,
                aggchain_params,
                public_values,
            } = parse_aggchain_proof(aggchain_proof)?;

            AggchainData::Generic {
                proof,
                aggchain_params,
                signature,
                public_values,
            }
        }
        Some(agglayer_interop::grpc::v1::aggchain_data::Data::Multisig(multisig)) => {
            AggchainData::MultisigOnly {
                multisig: parse_multisig(multisig)?,
            }
        }
        Some(agglayer_interop::grpc::v1::aggchain_data::Data::MultisigAndAggchainProof(
            multisig_and_aggchain_proof,
        )) => AggchainData::MultisigAndAggchainProof {
            multisig: parse_multisig(
                multisig_and_aggchain_proof
                    .multisig
                    .ok_or(Error::missing_field("multisig"))?,
            )
            .map_err(|e| e.inside_field("multisig"))?,
            aggchain_proof: parse_aggchain_proof(
                multisig_and_aggchain_proof
                    .aggchain_proof
                    .ok_or(Error::missing_field("aggchain_proof"))?,
            )
            .map_err(|e| e.inside_field("aggchain_proof"))?,
        },
        None => return Err(Error::missing_field("data")),
    })
}

fn encode_public_values(
    public_values: Option<Box<agglayer_types::aggchain_proof::AggchainProofPublicValues>>,
) -> Result<HashMap<String, Bytes>, Error> {
    Ok(match public_values {
        Some(public_values) => HashMap::from([(
            "public_values".to_owned(),
            Bytes::from(
                std::panic::catch_unwind(|| bincode::default().serialize(&*public_values))
                    .map_err(|_| {
                        Error::serializing_aggchain_proof_public_values(Box::new(
                            bincode::ErrorKind::Custom(String::from("panic")),
                        ))
                    })?
                    .map_err(Error::serializing_context)?,
            ),
        )]),
        None => HashMap::new(),
    })
}

fn encode_proof(value: Proof) -> Result<agglayer_interop::grpc::v1::aggchain_proof::Proof, Error> {
    match value {
        Proof::SP1Stark(proof) => Ok(agglayer_interop::grpc::v1::aggchain_proof::Proof::Sp1Stark(
            agglayer_interop::grpc::v1::Sp1StarkProof {
                version: proof.version,
                proof: proof.proof.into(),
                vkey: proof.vkey.into(),
            },
        )),
    }
}

fn encode_aggchain_proof(
    value: AggchainProof,
) -> Result<agglayer_interop::grpc::v1::AggchainProof, Error> {
    Ok(agglayer_interop::grpc::v1::AggchainProof {
        proof: Some(encode_proof(value.proof)?),
        aggchain_params: Some(value.aggchain_params.into()),
        signature: None,
        context: encode_public_values(value.public_values)?,
    })
}

fn encode_multisig(multisig: MultisigPayload) -> agglayer_interop::grpc::v1::Multisig {
    agglayer_interop::grpc::v1::Multisig {
        data: Some(agglayer_interop::grpc::v1::multisig::Data::Ecdsa(
            agglayer_interop::grpc::v1::EcdsaMultisig {
                signatures: multisig
                    .0
                    .iter()
                    .enumerate()
                    .map(|(index, sig)| {
                        agglayer_interop::grpc::v1::ecdsa_multisig::EcdsaMultisigEntry {
                            index: index.try_into().unwrap_or(u32::MAX),
                            signature: sig.map(|sig| agglayer_interop::grpc::v1::FixedBytes65 {
                                value: Bytes::copy_from_slice(&sig.as_bytes()),
                            }),
                        }
                    })
                    .collect(),
            },
        )),
    }
}

fn encode_aggchain_data(
    value: AggchainData,
) -> Result<agglayer_interop::grpc::v1::AggchainData, Error> {
    Ok(agglayer_interop::grpc::v1::AggchainData {
        data: Some(match value {
            AggchainData::ECDSA { signature } => {
                agglayer_interop::grpc::v1::aggchain_data::Data::Signature(
                    agglayer_interop::grpc::v1::FixedBytes65 {
                        value: Bytes::copy_from_slice(&signature.as_bytes()),
                    },
                )
            }
            AggchainData::Generic {
                proof,
                signature,
                aggchain_params,
                public_values,
            } => agglayer_interop::grpc::v1::aggchain_data::Data::Generic(
                agglayer_interop::grpc::v1::AggchainProof {
                    context: encode_public_values(public_values)?,
                    aggchain_params: Some(aggchain_params.into()),
                    signature: signature.map(|signature| {
                        agglayer_interop::grpc::v1::FixedBytes65 {
                            value: Bytes::copy_from_slice(&signature.as_bytes()),
                        }
                    }),
                    proof: Some(encode_proof(proof)?),
                },
            ),
            AggchainData::MultisigOnly { multisig } => {
                agglayer_interop::grpc::v1::aggchain_data::Data::Multisig(encode_multisig(multisig))
            }
            AggchainData::MultisigAndAggchainProof {
                multisig,
                aggchain_proof,
            } => agglayer_interop::grpc::v1::aggchain_data::Data::MultisigAndAggchainProof(
                agglayer_interop::grpc::v1::AggchainProofWithMultisig {
                    multisig: Some(encode_multisig(multisig)),
                    aggchain_proof: Some(encode_aggchain_proof(aggchain_proof)?),
                },
            ),
        }),
    })
}

impl TryFrom<v1::Certificate> for Certificate {
    type Error = Error;

    fn try_from(value: v1::Certificate) -> Result<Self, Self::Error> {
        let aggchain_data = value
            .aggchain_data
            .ok_or(Error::missing_field("aggchain_data"))?;
        let aggchain_data =
            parse_aggchain_data(aggchain_data).map_err(|e| e.inside_field("aggchain_data"))?;

        let has_multisig = matches!(
            aggchain_data,
            AggchainData::Generic { .. }
                | AggchainData::MultisigOnly { .. }
                | AggchainData::MultisigAndAggchainProof { .. }
        );

        if has_multisig && value.metadata.is_some() {
            return Err(Error::invalid_data(
                "metadata provided with multisig".to_owned(),
            ));
        }

        Ok(Certificate {
            network_id: NetworkId::new(value.network_id),
            height: Height::new(value.height),
            prev_local_exit_root: required_field!(value, prev_local_exit_root),
            new_local_exit_root: required_field!(value, new_local_exit_root),
            bridge_exits: value
                .bridge_exits
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()
                .map_err(|e: Error| e.inside_field("bridge_exits"))?,
            imported_bridge_exits: value
                .imported_bridge_exits
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()
                .map_err(|e: Error| e.inside_field("imported_bridge_exits"))?,
            aggchain_data,
            metadata: if let Some(metadata) = value.metadata {
                Metadata::new(metadata.try_into()?)
            } else {
                Metadata::default()
            },
            custom_chain_data: value.custom_chain_data.to_vec(),
            l1_info_tree_leaf_count: value.l1_info_tree_leaf_count,
        })
    }
}

impl TryFrom<Certificate> for v1::Certificate {
    type Error = Error;

    fn try_from(value: Certificate) -> Result<Self, Self::Error> {
        Ok(v1::Certificate {
            network_id: value.network_id.into(),
            height: value.height.as_u64(),
            prev_local_exit_root: Some(value.prev_local_exit_root.into()),
            new_local_exit_root: Some(value.new_local_exit_root.into()),
            bridge_exits: value.bridge_exits.into_iter().map(Into::into).collect(),
            imported_bridge_exits: value
                .imported_bridge_exits
                .into_iter()
                .map(Into::into)
                .collect(),
            aggchain_data: Some(encode_aggchain_data(value.aggchain_data)?),
            metadata: Some((*value.metadata).into()),
            custom_chain_data: value.custom_chain_data.into(),
            l1_info_tree_leaf_count: value.l1_info_tree_leaf_count,
        })
    }
}
