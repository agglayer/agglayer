use agglayer_types::Address;

use crate::protocol::types::v1;

use super::Error;

impl TryFrom<v1::FixedBytes20> for Address {
    type Error = Error;

    fn try_from(value: v1::FixedBytes20) -> Result<Self, Self::Error> {
        Address::try_from(&*value.value).map_err(|_| Error::WrongBytesLength {
            expected: 20,
            actual: value.value.len(),
        })
    }
}
