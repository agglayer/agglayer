use agglayer_types::{Certificate, NetworkId};

use super::Error;
use crate::protocol::types::v1;

impl TryFrom<v1::Certificate> for Certificate {
    type Error = Error;

    fn try_from(value: v1::Certificate) -> Result<Self, Self::Error> {
        Ok(Certificate {
            network_id: NetworkId::new(value.network_id),
            height: value.height,
            prev_local_exit_root: required_field!(value, prev_local_exit_root),
            new_local_exit_root: required_field!(value, new_local_exit_root),
            bridge_exits: value
                .bridge_exits
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()
                .map_err(|e| Error::ParsingField("bridge_exits", Box::new(e)))?,
            imported_bridge_exits: value
                .imported_bridge_exits
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()
                .map_err(|e| Error::ParsingField("imported_bridge_exits", Box::new(e)))?,
            aggchain_data: required_field!(value, aggchain_data),
            metadata: required_field!(value, metadata),
        })
    }
}

impl TryFrom<Certificate> for v1::Certificate {
    type Error = Error;

    fn try_from(value: Certificate) -> Result<Self, Self::Error> {
        Ok(v1::Certificate {
            network_id: value.network_id.into(),
            height: value.height,
            prev_local_exit_root: Some(value.prev_local_exit_root.into()),
            new_local_exit_root: Some(value.new_local_exit_root.into()),
            bridge_exits: value.bridge_exits.into_iter().map(Into::into).collect(),
            imported_bridge_exits: value
                .imported_bridge_exits
                .into_iter()
                .map(Into::into)
                .collect(),
            aggchain_data: Some(value.aggchain_data.try_into()?),
            metadata: Some(value.metadata.into()),
            custom_chain_data: Default::default(), // TODO: should be added to Certificate
        })
    }
}
