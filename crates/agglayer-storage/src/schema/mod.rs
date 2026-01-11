mod codec;
mod column;
pub mod options;

pub(crate) use codec::impl_codec_using_bincode_for;
pub use codec::{bincode_codec, Codec, CodecError};
pub use column::{ColumnDescriptor, ColumnSchema};
