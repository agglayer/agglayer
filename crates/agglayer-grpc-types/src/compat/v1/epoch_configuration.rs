use agglayer_types::EpochConfiguration;

use crate::node::v1;

impl From<v1::types::EpochConfiguration> for EpochConfiguration {
    fn from(value: v1::types::EpochConfiguration) -> Self {
        EpochConfiguration {
            genesis_block: value.genesis_block,
            epoch_duration: value.epoch_duration,
        }
    }
}

impl From<EpochConfiguration> for v1::types::EpochConfiguration {
    fn from(value: EpochConfiguration) -> Self {
        v1::types::EpochConfiguration {
            genesis_block: value.genesis_block,
            epoch_duration: value.epoch_duration,
        }
    }
}
