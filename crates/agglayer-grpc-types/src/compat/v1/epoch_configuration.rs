use agglayer_types::EpochConfiguration;

use crate::protocol::types::v1;

impl From<v1::EpochConfiguration> for EpochConfiguration {
    fn from(value: v1::EpochConfiguration) -> Self {
        EpochConfiguration {
            genesis_block: value.genesis_block,
            epoch_duration: value.epoch_duration,
        }
    }
}

impl From<EpochConfiguration> for v1::EpochConfiguration {
    fn from(value: EpochConfiguration) -> Self {
        v1::EpochConfiguration {
            genesis_block: value.genesis_block,
            epoch_duration: value.epoch_duration,
        }
    }
}
