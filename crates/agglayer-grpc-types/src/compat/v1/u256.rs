use agglayer_types::U256;
use prost::bytes::Bytes;

use super::Error;
use crate::protocol::types::v1;

impl TryFrom<v1::FixedBytes32> for U256 {
    type Error = Error;

    fn try_from(value: v1::FixedBytes32) -> Result<Self, Self::Error> {
        Ok(U256::from_be_bytes(<[u8; 32]>::try_from(value)?))
    }
}

impl From<U256> for v1::FixedBytes32 {
    fn from(value: U256) -> Self {
        v1::FixedBytes32 {
            value: Bytes::copy_from_slice(&value.to_be_bytes::<32>()),
        }
    }
}
