use agglayer_interop::grpc::v1::FixedBytes32;
use agglayer_types::{CertificateId, Digest};
use prost::bytes::Bytes;

use super::Error;
use crate::node::types::v1;

impl TryFrom<v1::CertificateId> for CertificateId {
    type Error = Error;

    fn try_from(value: v1::CertificateId) -> Result<Self, Self::Error> {
        let value = value.value.ok_or(Error::missing_field("value"))?;
        Ok(CertificateId::new(Digest::from(<[u8; 32]>::try_from(
            value,
        )?)))
    }
}

impl From<CertificateId> for v1::CertificateId {
    fn from(value: CertificateId) -> Self {
        v1::CertificateId {
            value: Some(FixedBytes32 {
                value: Bytes::copy_from_slice(&value.as_slice()),
            }),
        }
    }
}
