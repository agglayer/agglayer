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

    #[error(r#"Conversion error: {0}"#)]
    Conversion(String),

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

impl<const N: usize> Codec for [u8; N] {
    fn encode_into<W: io::Write>(&self, mut writer: W) -> Result<(), CodecError> {
        writer.write_all(self)?;

        Ok(())
    }

    fn decode(buf: &[u8]) -> Result<Self, CodecError> {
        crate::schema::fixed_bytes::<N>(buf, "fixed byte array")
    }
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

macro_rules! impl_codec_using_protobuf_for {
    ($($type:ty),* $(,)?) => {
        $(
            impl $crate::schema::Codec for $type {
                fn encode_into<W: ::std::io::Write>(
                    &self,
                    mut writer: W,
                ) -> Result<(), $crate::schema::CodecError> {
                    let mut buf = ::prost::bytes::BytesMut::with_capacity(
                        <Self as ::prost::Message>::encoded_len(self),
                    );

                    <Self as ::prost::Message>::encode(self, &mut buf)?;

                    writer.write_all(&buf)?;

                    Ok(())
                }

                fn decode(buf: &[u8]) -> Result<Self, $crate::schema::CodecError> {
                    <Self as ::prost::Message>::decode(buf).map_err(Into::into)
                }
            }
        )*
    };
    ($($type:ty => $proto:ty),* $(,)?) => {
        $(
            impl $crate::schema::Codec for $type {
                fn encode_into<W: ::std::io::Write>(
                    &self,
                    mut writer: W,
                ) -> Result<(), $crate::schema::CodecError> {
                    let proto: $proto = self.into();
                    let mut buf = ::prost::bytes::BytesMut::with_capacity(
                        <$proto as ::prost::Message>::encoded_len(&proto),
                    );

                    <$proto as ::prost::Message>::encode(&proto, &mut buf)?;

                    writer.write_all(&buf)?;

                    Ok(())
                }

                fn decode(buf: &[u8]) -> Result<Self, $crate::schema::CodecError> {
                    let proto = <$proto as ::prost::Message>::decode(buf)?;
                    Ok(proto.into())
                }
            }
        )*
    };
}

pub(crate) use impl_codec_using_protobuf_for;
