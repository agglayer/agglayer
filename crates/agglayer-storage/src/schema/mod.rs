mod codec;
mod column;
pub mod options;

pub use codec::{bincode_codec, Codec, CodecError};
pub(crate) use codec::{impl_codec_using_bincode_for, impl_codec_using_protobuf_for};
pub use column::{ColumnDescriptor, ColumnSchema};

pub(crate) const RAW_ULID_LEN: usize = 16;
pub(crate) const U32_LEN: usize = std::mem::size_of::<u32>();
pub(crate) const U64_LEN: usize = std::mem::size_of::<u64>();

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

pub(crate) fn decode_u32_be(bytes: &[u8], field: &'static str) -> Result<u32, CodecError> {
    Ok(u32::from_be_bytes(fixed_bytes::<U32_LEN>(bytes, field)?))
}

pub(crate) fn decode_u64_be(bytes: &[u8], field: &'static str) -> Result<u64, CodecError> {
    Ok(u64::from_be_bytes(fixed_bytes::<U64_LEN>(bytes, field)?))
}

pub(crate) fn decode_raw_ulid(bytes: &[u8], field: &'static str) -> Result<ulid::Ulid, CodecError> {
    Ok(ulid::Ulid::from_bytes(fixed_bytes::<RAW_ULID_LEN>(
        bytes, field,
    )?))
}
