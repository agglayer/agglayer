use agglayer_types::{L1InfoTreeLeaf, L1InfoTreeLeafInner};

use super::Error;
use crate::protocol::types::v1;

impl TryFrom<v1::L1InfoTreeLeaf> for L1InfoTreeLeafInner {
    type Error = Error;

    fn try_from(value: v1::L1InfoTreeLeaf) -> Result<Self, Self::Error> {
        Ok(L1InfoTreeLeafInner {
            global_exit_root: required_field!(value, global_exit_root),
            block_hash: required_field!(value, block_hash),
            timestamp: value.timestamp,
        })
    }
}

impl From<L1InfoTreeLeafInner> for v1::L1InfoTreeLeaf {
    fn from(value: L1InfoTreeLeafInner) -> Self {
        v1::L1InfoTreeLeaf {
            global_exit_root: Some(value.global_exit_root.into()),
            block_hash: Some(value.block_hash.into()),
            timestamp: value.timestamp,
        }
    }
}

impl TryFrom<v1::L1InfoTreeLeafWithContext> for L1InfoTreeLeaf {
    type Error = Error;

    fn try_from(value: v1::L1InfoTreeLeafWithContext) -> Result<Self, Self::Error> {
        Ok(L1InfoTreeLeaf {
            l1_info_tree_index: value.l1_info_tree_index,
            rer: required_field!(value, rer),
            mer: required_field!(value, mer),
            inner: required_field!(value, inner),
        })
    }
}

impl From<L1InfoTreeLeaf> for v1::L1InfoTreeLeafWithContext {
    fn from(value: L1InfoTreeLeaf) -> Self {
        v1::L1InfoTreeLeafWithContext {
            l1_info_tree_index: value.l1_info_tree_index,
            rer: Some(value.rer.into()),
            mer: Some(value.mer.into()),
            inner: Some(value.inner.into()),
        }
    }
}
