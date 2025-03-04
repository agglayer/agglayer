use std::fmt::Display;

use crate::{compat::v1::error::ErrorKind, node::v1::GetCertificateHeaderErrorKind};

impl Display for GetCertificateHeaderErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetCertificateHeaderErrorKind::Unspecified => write!(f, "Unspecified error"),
            GetCertificateHeaderErrorKind::MissingField => write!(f, "Missing field"),
            GetCertificateHeaderErrorKind::InvalidData => write!(f, "Invalid data"),
            GetCertificateHeaderErrorKind::NotFound => write!(f, "Certificate not found"),
        }
    }
}

impl From<GetCertificateHeaderErrorKind> for String {
    fn from(kind: GetCertificateHeaderErrorKind) -> String {
        kind.to_string()
    }
}

impl From<ErrorKind> for GetCertificateHeaderErrorKind {
    fn from(value: ErrorKind) -> Self {
        match value {
            ErrorKind::MissingField => GetCertificateHeaderErrorKind::MissingField,
            ErrorKind::InvalidData => GetCertificateHeaderErrorKind::InvalidData,
        }
    }
}
