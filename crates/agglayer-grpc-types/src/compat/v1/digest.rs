use agglayer_types::Digest;

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
