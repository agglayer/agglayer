use agglayer_types::NetworkId;

use crate::{columns::DISABLED_NETWORKS_CF, schema::ColumnSchema};

/// Column family containing the disabled networks.
///
/// ## Column definition
///
/// | key             | value            |
/// | --              | --               |
/// | `NetworkId`     | `DisabledNetwork`|
pub(crate) struct DisabledNetworksColumn;

impl ColumnSchema for DisabledNetworksColumn {
    type Key = NetworkId;
    type Value = crate::types::disabled_network::Value;

    const COLUMN_FAMILY_NAME: &'static str = DISABLED_NETWORKS_CF;
}
