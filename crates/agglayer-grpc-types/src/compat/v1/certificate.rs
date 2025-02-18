use agglayer_types::{Certificate, NetworkId};

use crate::protocol::types::v1;

use super::Error;

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
            aggchain_proof: required_field!(value, aggchain_proof),
            metadata: required_field!(value, metadata),
        })
    }
}
