use agglayer_types::CertificateId;
use prost::bytes::Bytes;

use crate::protocol::types::v1;

use super::Error;

impl TryFrom<v1::CertificateId> for CertificateId {
    type Error = Error;

    fn try_from(value: v1::CertificateId) -> Result<Self, Self::Error> {
        let value = value
            .certificate_id
            .ok_or(Error::MissingField("certificate_id"))?;
        CertificateId::try_from(&*value.value).map_err(|_| Error::WrongBytesLength {
            expected: 32,
            actual: value.value.len(),
        })
    }
}

impl From<CertificateId> for v1::CertificateId {
    fn from(value: CertificateId) -> Self {
        v1::CertificateId {
            certificate_id: Some(v1::FixedBytes32 {
                value: Bytes::copy_from_slice(&value.0),
            }),
        }
    }
}
