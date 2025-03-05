use super::Error;
use crate::protocol::types::v1;

impl TryFrom<v1::FixedBytes20> for [u8; 20] {
    type Error = Error;

    fn try_from(value: v1::FixedBytes20) -> Result<Self, Self::Error> {
        (&*value.value).try_into().map_err(|_| {
            Error::invalid_data(format!("expected 20 bytes, got {}", value.value.len()))
        })
    }
}

impl TryFrom<v1::FixedBytes32> for [u8; 32] {
    type Error = Error;

    fn try_from(value: v1::FixedBytes32) -> Result<Self, Self::Error> {
        (&*value.value).try_into().map_err(|_| {
            Error::invalid_data(format!("expected 32 bytes, got {}", value.value.len()))
        })
    }
}
