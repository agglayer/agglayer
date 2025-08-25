macro_rules! impl_codec_using_bincode_for {
    ($($type:ty),* $(,)?) => {
        $(
            impl $crate::columns::Codec for $type {
                fn encode(&self) -> Result<Vec<u8>, $crate::columns::CodecError> {
                    Ok($crate::columns::bincode_codec().serialize(self)?)
                }

                fn decode(buf: &[u8]) -> Result<Self, $crate::columns::CodecError> {
                    Ok($crate::columns::bincode_codec().deserialize(buf)?)
                }
            }
        )*
    };
}

// Physical storage
#[rustfmt::skip]
pub mod storage;
// Logical store
#[rustfmt::skip]
pub mod stores;

#[macro_use]
#[rustfmt::skip]
pub mod columns;
#[rustfmt::skip]
pub mod error;

#[rustfmt::skip]
pub mod types;

#[cfg(any(test, feature = "testutils"))]
pub mod tests;
