use agglayer_types::{BridgeExit, LeafType, NetworkId};

use crate::protocol::types::v1;

use super::Error;

impl TryFrom<v1::BridgeExit> for BridgeExit {
    type Error = Error;

    fn try_from(value: v1::BridgeExit) -> Result<Self, Self::Error> {
        Ok(BridgeExit {
            leaf_type: match value.leaf_type() {
                v1::LeafType::Transfer => LeafType::Transfer,
                v1::LeafType::Message => LeafType::Message,
                _ => return Err(Error::InvalidLeafType(value.leaf_type)),
            },
            token_info: required_field!(value, token_info),
            dest_network: NetworkId::new(value.dest_network),
            dest_address: required_field!(value, dest_address),
            amount: required_field!(value, amount),
            metadata: value
                .metadata
                .map(TryInto::try_into)
                .transpose()
                .map_err(|e| Error::ParsingField("metadata", Box::new(e)))?,
        })
    }
}
