mod codec;
mod column;
pub mod options;

pub use codec::{bincode_codec, Codec, CodecError};
pub(crate) use codec::{impl_codec_using_bincode_for, impl_codec_using_protobuf_for};
pub use column::{ColumnDescriptor, ColumnSchema};

pub(crate) const U32_LEN: usize = std::mem::size_of::<u32>();
pub(crate) const U64_LEN: usize = std::mem::size_of::<u64>();
pub(crate) const U128_LEN: usize = std::mem::size_of::<u128>();

/// Converts a byte slice into an array with an exact, schema-defined length.
pub(crate) fn fixed_bytes<const N: usize>(
    bytes: &[u8],
    field: &'static str,
) -> Result<[u8; N], CodecError> {
    bytes.try_into().map_err(|_| {
        CodecError::Conversion(format!(
            "{field} must be {N} bytes long, got {}",
            bytes.len()
        ))
    })
}

/// Decodes a big-endian `u32` from a fixed-width storage field.
pub(crate) fn decode_u32_be(bytes: &[u8], field: &'static str) -> Result<u32, CodecError> {
    Ok(u32::from_be_bytes(fixed_bytes::<U32_LEN>(bytes, field)?))
}

/// Decodes a big-endian `u64` from a fixed-width storage field.
pub(crate) fn decode_u64_be(bytes: &[u8], field: &'static str) -> Result<u64, CodecError> {
    Ok(u64::from_be_bytes(fixed_bytes::<U64_LEN>(bytes, field)?))
}

/// Decodes a big-endian `u128` from a fixed-width storage field.
pub(crate) fn decode_u128_be(bytes: &[u8], field: &'static str) -> Result<u128, CodecError> {
    Ok(u128::from_be_bytes(fixed_bytes::<U128_LEN>(bytes, field)?))
}
