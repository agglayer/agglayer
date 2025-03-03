use std::fmt::Display;

use crate::{
    compat::v1::{error::ErrorKind, Error},
    node::v1::GetCertificateHeaderErrorKind,
};

impl Display for GetCertificateHeaderErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetCertificateHeaderErrorKind::MissingCertificateId => {
                write!(f, "Missing certificate ID")
            }
            GetCertificateHeaderErrorKind::MalformedCertificateId => {
                write!(f, "Malformed certificate ID")
            }
            GetCertificateHeaderErrorKind::Unspecified => {
                write!(f, "Unspecified error")
            }
            GetCertificateHeaderErrorKind::Internal => {
                write!(f, "Internal error")
            }
        }
    }
}

impl From<GetCertificateHeaderErrorKind> for String {
    fn from(kind: GetCertificateHeaderErrorKind) -> String {
        kind.to_string()
    }
}

impl From<&Error> for GetCertificateHeaderErrorKind {
    fn from(value: &Error) -> Self {
        if value.field() == ["certificate_id"] {
            match value.kind() {
                ErrorKind::MissingField => GetCertificateHeaderErrorKind::MissingCertificateId,
                ErrorKind::InvalidData => GetCertificateHeaderErrorKind::MalformedCertificateId,
            }
        } else {
            GetCertificateHeaderErrorKind::Unspecified
        }
    }
}
