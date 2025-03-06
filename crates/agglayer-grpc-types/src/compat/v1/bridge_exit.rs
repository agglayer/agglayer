use agglayer_types::{BridgeExit, LeafType, NetworkId};

use super::Error;
use crate::protocol::types::v1;

impl TryFrom<v1::BridgeExit> for BridgeExit {
    type Error = Error;

    fn try_from(value: v1::BridgeExit) -> Result<Self, Self::Error> {
        Ok(BridgeExit {
            leaf_type: match value.leaf_type() {
                v1::LeafType::Transfer => LeafType::Transfer,
                v1::LeafType::Message => LeafType::Message,
                _ => {
                    let t = value.leaf_type;
                    return Err(Error::invalid_data(format!("invalid leaf type: {t}")));
                }
            },
            token_info: required_field!(value, token_info),
            dest_network: NetworkId::new(value.dest_network),
            dest_address: required_field!(value, dest_address),
            amount: required_field!(value, amount),
            metadata: value
                .metadata
                .map(TryInto::try_into)
                .transpose()
                .map_err(|e: Error| e.inside_field("metadata"))?,
        })
    }
}

impl From<BridgeExit> for v1::BridgeExit {
    fn from(value: BridgeExit) -> Self {
        v1::BridgeExit {
            leaf_type: match value.leaf_type {
                LeafType::Transfer => v1::LeafType::Transfer,
                LeafType::Message => v1::LeafType::Message,
            }
            .into(),
            token_info: Some(value.token_info.into()),
            dest_network: value.dest_network.into(),
            dest_address: Some(value.dest_address.into()),
            amount: Some(value.amount.into()),
            metadata: value.metadata.map(Into::into),
        }
    }
}
