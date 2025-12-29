use std::io;

use agglayer_types::bincode;

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error(r#"Serialization error: {0}
        This is a critical bug that needs to be reported on `https://github.com/agglayer/agglayer/issues`"#)]
    Serialization(#[from] bincode::Error),

    #[error(r#"Certificate encoded to an empty byte sequence.
        This is a critical bug that needs to be reported on `https://github.com/agglayer/agglayer/issues`"#)]
    CertificateEmpty,

    #[error(r#"Unrecognized certificate storage format version {version}.
        This is a critical bug that needs to be reported on `https://github.com/agglayer/agglayer/issues`"#)]
    BadCertificateVersion { version: u8 },

    #[error(r#"Serialization error: {0}
        This is a critical bug that needs to be reported on `https://github.com/agglayer/agglayer/issues`"#)]
    ProtobufSerialization(#[from] prost::EncodeError),

    #[error(r#"Deserialization error: {0}
           This is a critical bug that needs to be reported on `https://github.com/agglayer/agglayer/issues`"#)]
    ProtobufDeserialization(#[from] prost::DecodeError),

    #[error(r#"Invalid enum variant {0}"#)]
    InvalidEnumVariant(String),

    #[error(r#"Unable to write encoded bytes: {0}"#)]
    UnableToWriteEncodedBytes(#[from] std::io::Error),
}

pub fn bincode_codec() -> bincode::Codec<impl bincode::Options> {
    bincode::default()
}

pub trait Codec: Sized {
    #[inline]
    fn encode(&self) -> Result<Vec<u8>, CodecError> {
        let mut buffer = Vec::new();
        self.encode_into(&mut buffer)?;
        Ok(buffer)
    }

    fn encode_into<W: io::Write>(&self, writer: W) -> Result<(), CodecError>;

    fn decode(buf: &[u8]) -> Result<Self, CodecError>;
}

macro_rules! impl_codec_using_bincode_for {
    ($($type:ty),* $(,)?) => {
        $(
            impl $crate::schema::Codec for $type {
                fn encode_into<W: ::std::io::Write>(
                    &self,
                    writer: W,
                ) -> Result<(), $crate::schema::CodecError> {
                    Ok($crate::schema::bincode_codec().serialize_into(writer, self)?)
                }

                fn decode(buf: &[u8]) -> Result<Self, $crate::schema::CodecError> {
                    Ok($crate::schema::bincode_codec().deserialize(buf)?)
                }
            }
        )*
    };
}

pub(crate) use impl_codec_using_bincode_for;
