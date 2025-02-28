use std::fmt::Display;

use crate::{compat::v1::ErrorKind, node::v1::GetLatestCertificateHeaderErrorKind};

impl Display for GetLatestCertificateHeaderErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetLatestCertificateHeaderErrorKind::Unspecified => write!(f, "Unspecified error"),
            GetLatestCertificateHeaderErrorKind::MissingField => write!(f, "Missing field"),
            GetLatestCertificateHeaderErrorKind::InvalidData => write!(f, "Invalid data"),
            GetLatestCertificateHeaderErrorKind::NotFound => write!(f, "Certificate not found"),
        }
    }
}

impl From<GetLatestCertificateHeaderErrorKind> for String {
    fn from(kind: GetLatestCertificateHeaderErrorKind) -> Self {
        kind.to_string()
    }
}

impl From<ErrorKind> for GetLatestCertificateHeaderErrorKind {
    fn from(value: ErrorKind) -> Self {
        match value {
            ErrorKind::InvalidData => GetLatestCertificateHeaderErrorKind::InvalidData,
            ErrorKind::MissingField => GetLatestCertificateHeaderErrorKind::MissingField,
        }
    }
}
