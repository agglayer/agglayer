use agglayer_types::Digest;
use prost::bytes::Bytes;

use crate::protocol::types::v1;

use super::Error;

impl TryFrom<v1::FixedBytes32> for Digest {
    type Error = Error;

    fn try_from(value: v1::FixedBytes32) -> Result<Self, Self::Error> {
        Digest::try_from(&*value.value).map_err(|_| Error::WrongBytesLength {
            expected: 32,
            actual: value.value.len(),
        })
    }
}

impl From<Digest> for v1::FixedBytes32 {
    fn from(value: Digest) -> Self {
        v1::FixedBytes32 {
            value: Bytes::copy_from_slice(&value.0),
        }
    }
}
