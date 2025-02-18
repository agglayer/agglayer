use agglayer_types::EpochConfiguration;

use crate::protocol::types::v1;

use super::Error;

impl TryFrom<v1::EpochConfiguration> for EpochConfiguration {
    type Error = Error;

    fn try_from(value: v1::EpochConfiguration) -> Result<Self, Self::Error> {
        Ok(EpochConfiguration {
            genesis_block: value.genesis_block,
            epoch_duration: value.epoch_duration,
        })
    }
}
