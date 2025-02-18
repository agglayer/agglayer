use agglayer_types::{L1InfoTreeLeaf, L1InfoTreeLeafInner};

use crate::protocol::types::v1;

use super::Error;

impl TryFrom<v1::L1InfoTreeLeafInner> for L1InfoTreeLeafInner {
    type Error = Error;

    fn try_from(value: v1::L1InfoTreeLeafInner) -> Result<Self, Self::Error> {
        Ok(L1InfoTreeLeafInner {
            global_exit_root: required_field!(value, global_exit_root),
            block_hash: required_field!(value, block_hash),
            timestamp: value.timestamp,
        })
    }
}

impl TryFrom<v1::L1InfoTreeLeaf> for L1InfoTreeLeaf {
    type Error = Error;

    fn try_from(value: v1::L1InfoTreeLeaf) -> Result<Self, Self::Error> {
        Ok(L1InfoTreeLeaf {
            l1_info_tree_index: value.l1_info_tree_index,
            rer: required_field!(value, rer),
            mer: required_field!(value, mer),
            inner: required_field!(value, inner),
        })
    }
}
