use agglayer_types::{
    aggchain_proof::{AggchainData, AggchainProof, ProofExt as _},
    Certificate, Height, Metadata, NetworkId,
};
use prost::bytes::Bytes;

use super::Error;
use crate::node::types::v1;

impl TryFrom<v1::Certificate> for Certificate {
    type Error = Error;

    fn try_from(value: v1::Certificate) -> Result<Self, Self::Error> {
        let aggchain_data = match value
            .aggchain_data
            .ok_or(Error::missing_field("aggchain_data"))?
            .data
        {
            Some(agglayer_interop::grpc::v1::aggchain_data::Data::Signature(signature)) => {
                AggchainData::ECDSA {
                    signature: (&*signature.value)
                        .try_into()
                        .map_err(Error::parsing_signature)
                        .map_err(|e| e.inside_field("aggchain_data"))?,
                }
            }
            Some(agglayer_interop::grpc::v1::aggchain_data::Data::Generic(aggchain_proof)) => {
                let parsed: AggchainData = agglayer_interop::grpc::v1::AggchainData {
                    data: Some(agglayer_interop::grpc::v1::aggchain_data::Data::Generic(
                        aggchain_proof,
                    )),
                }
                .try_into()
                .map_err(|e: Error| e.inside_field("aggchain_data"))?;

                let AggchainData::Generic {
                    proof,
                    aggchain_params,
                    signature,
                    public_values,
                } = parsed
                else {
                    unreachable!("interop generic conversion returned a non-generic variant")
                };

                proof.ensure_deserializable().map_err(
                    |err: agglayer_types::aggchain_proof::ProofError| {
                        Error::invalid_data(err.to_string()).inside_field("aggchain_data")
                    },
                )?;

                AggchainData::Generic {
                    proof,
                    aggchain_params,
                    signature,
                    public_values,
                }
            }
            Some(agglayer_interop::grpc::v1::aggchain_data::Data::Multisig(multisig)) => {
                // Full multisig delegation is temporary until agglayer/interop#214
                // hardens signer-index overflow handling in the upstream compat decoder.
                AggchainData::MultisigOnly {
                    multisig: multisig
                        .try_into()
                        .map_err(|e: Error| e.inside_field("aggchain_data"))?,
                }
            }
            Some(agglayer_interop::grpc::v1::aggchain_data::Data::MultisigAndAggchainProof(
                multisig_and_aggchain_proof,
            )) => AggchainData::MultisigAndAggchainProof {
                multisig: multisig_and_aggchain_proof
                    .multisig
                    .ok_or(Error::missing_field("multisig"))
                    .and_then(|multisig| {
                        multisig
                            .try_into()
                            .map_err(|e: Error| e.inside_field("multisig"))
                    })
                    .map_err(|e| e.inside_field("aggchain_data"))?,
                aggchain_proof: {
                    let aggchain_proof = multisig_and_aggchain_proof
                        .aggchain_proof
                        .ok_or(Error::missing_field("aggchain_proof"))
                        .map_err(|e| e.inside_field("aggchain_data"))?;

                    let parsed: AggchainProof = aggchain_proof
                        .try_into()
                        .map_err(|e: Error| e.inside_field("aggchain_data"))?;

                    parsed.proof.ensure_deserializable().map_err(
                        |err: agglayer_types::aggchain_proof::ProofError| {
                            Error::invalid_data(err.to_string()).inside_field("aggchain_data")
                        },
                    )?;

                    parsed
                },
            },
            None => return Err(Error::missing_field("data").inside_field("aggchain_data")),
        };

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
        let aggchain_data = agglayer_interop::grpc::v1::AggchainData {
            data: Some(match value.aggchain_data {
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
                } => match agglayer_interop::grpc::v1::AggchainData::try_from(
                    AggchainData::Generic {
                        proof,
                        signature,
                        aggchain_params,
                        public_values,
                    },
                )?
                .data
                {
                    Some(agglayer_interop::grpc::v1::aggchain_data::Data::Generic(proof)) => {
                        agglayer_interop::grpc::v1::aggchain_data::Data::Generic(proof)
                    }
                    _ => unreachable!("interop generic conversion returned a non-generic variant"),
                },
                AggchainData::MultisigOnly { multisig } => {
                    agglayer_interop::grpc::v1::aggchain_data::Data::Multisig(multisig.into())
                }
                AggchainData::MultisigAndAggchainProof {
                    multisig,
                    aggchain_proof,
                } => agglayer_interop::grpc::v1::aggchain_data::Data::MultisigAndAggchainProof(
                    agglayer_interop::grpc::v1::AggchainProofWithMultisig {
                        multisig: Some(multisig.into()),
                        aggchain_proof: Some(aggchain_proof.try_into()?),
                    },
                ),
            }),
        };

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
            aggchain_data: Some(aggchain_data),
            metadata: Some((*value.metadata).into()),
            custom_chain_data: value.custom_chain_data.into(),
            l1_info_tree_leaf_count: value.l1_info_tree_leaf_count,
        })
    }
}
