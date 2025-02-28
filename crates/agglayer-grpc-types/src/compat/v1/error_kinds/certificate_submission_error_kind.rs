use std::fmt::Display;

use crate::{compat::v1::error::ErrorKind, node::v1::SubmitCertificateErrorKind};

impl Display for SubmitCertificateErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubmitCertificateErrorKind::Unspecified => write!(f, "Unspecified error"),
            SubmitCertificateErrorKind::MissingField => write!(f, "Missing field"),
            SubmitCertificateErrorKind::InvalidData => write!(f, "Invalid data"),
            SubmitCertificateErrorKind::SignatureVerification => {
                write!(f, "Signature verification")
            }
            SubmitCertificateErrorKind::UnableToReplacePendingCertificate => {
                write!(f, "Unable to replace pending certificate")
            }
        }
    }
}

impl From<SubmitCertificateErrorKind> for String {
    fn from(kind: SubmitCertificateErrorKind) -> String {
        kind.to_string()
    }
}

impl From<ErrorKind> for SubmitCertificateErrorKind {
    fn from(value: ErrorKind) -> Self {
        match value {
            ErrorKind::InvalidData => SubmitCertificateErrorKind::InvalidData,
            ErrorKind::MissingField => SubmitCertificateErrorKind::MissingField,
        }
    }
}
