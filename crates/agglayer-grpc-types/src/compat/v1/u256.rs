use agglayer_types::U256;

use crate::protocol::types::v1;

use super::Error;

impl TryFrom<v1::FixedBytes32> for U256 {
    type Error = Error;

    fn try_from(value: v1::FixedBytes32) -> Result<Self, Self::Error> {
        Ok(U256::from_be_bytes(
            <[u8; 32]>::try_from(&*value.value).map_err(|_| Error::WrongBytesLength {
                expected: 32,
                actual: value.value.len(),
            })?,
        ))
    }
}
