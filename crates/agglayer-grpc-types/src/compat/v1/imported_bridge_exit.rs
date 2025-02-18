use agglayer_types::ImportedBridgeExit;

use crate::protocol::types::v1;

use super::Error;

impl TryFrom<v1::ImportedBridgeExit> for ImportedBridgeExit {
    type Error = Error;

    fn try_from(value: v1::ImportedBridgeExit) -> Result<Self, Self::Error> {
        Ok(ImportedBridgeExit {
            bridge_exit: required_field!(value, bridge_exit),
            claim_data: required_field!(value, claim),
            global_index: required_field!(value, global_index),
        })
    }
}
