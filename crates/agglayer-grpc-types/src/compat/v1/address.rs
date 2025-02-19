use agglayer_types::Address;
use prost::bytes::Bytes;

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

impl From<Address> for v1::FixedBytes20 {
    fn from(value: Address) -> Self {
        v1::FixedBytes20 {
            value: Bytes::copy_from_slice(&value.0 .0),
        }
    }
}
