mod codec;
mod column;
pub mod options;

pub use codec::{bincode_codec, Codec, CodecError};
pub(crate) use codec::{impl_codec_using_bincode_for, impl_codec_using_protobuf_for};
pub use column::{ColumnDescriptor, ColumnSchema};
