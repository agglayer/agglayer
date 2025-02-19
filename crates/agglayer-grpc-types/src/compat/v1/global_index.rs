use agglayer_types::{GlobalIndex, U256};

use crate::protocol::types::v1;

use super::Error;

impl TryFrom<v1::FixedBytes32> for GlobalIndex {
    type Error = Error;

    fn try_from(value: v1::FixedBytes32) -> Result<Self, Self::Error> {
        Ok(GlobalIndex::from(U256::try_from(value)?))
    }
}

impl From<GlobalIndex> for v1::FixedBytes32 {
    fn from(value: GlobalIndex) -> Self {
        <U256 as From<GlobalIndex>>::from(value).into()
    }
}
