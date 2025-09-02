use agglayer_types::{aggchain_proof::AggchainData, Certificate, Height, Metadata, NetworkId};

use super::Error;
use crate::node::types::v1;

impl TryFrom<v1::Certificate> for Certificate {
    type Error = Error;

    fn try_from(value: v1::Certificate) -> Result<Self, Self::Error> {
        let aggchain_data: AggchainData = required_field!(value, aggchain_data);

        // whether it involves a multisig
        let has_multisig = matches!(
            aggchain_data,
            AggchainData::Generic { .. } // 1-of-1
                | AggchainData::MultisigOnly(_)
                | AggchainData::MultisigAndAggchainProof { .. }
        );

        // forbidden case
        if has_multisig && value.metadata.is_some() {
            return Err(Error::invalid_data(
                "metadata provided with multisig".to_owned(),
            ));
        }

        let certificate = Certificate {
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
        };

        Ok(certificate)
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
            aggchain_data: Some(value.aggchain_data.try_into()?),
            metadata: Some((*value.metadata).into()),
            custom_chain_data: value.custom_chain_data.into(),
            l1_info_tree_leaf_count: value.l1_info_tree_leaf_count,
        })
    }
}
