use agglayer_types::ImportedBridgeExit;

use super::Error;
use crate::protocol::types::v1;

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

impl From<ImportedBridgeExit> for v1::ImportedBridgeExit {
    fn from(value: ImportedBridgeExit) -> Self {
        v1::ImportedBridgeExit {
            bridge_exit: Some(value.bridge_exit.into()),
            claim: Some(value.claim_data.into()),
            global_index: Some(value.global_index.into()),
        }
    }
}
