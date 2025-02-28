use std::fmt::Display;

use crate::node::v1::GetLatestCertificateHeaderErrorKind;

impl Display for GetLatestCertificateHeaderErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetLatestCertificateHeaderErrorKind::Internal => write!(f, "Internal error"),
            GetLatestCertificateHeaderErrorKind::InvalidRequestType => {
                write!(f, "Invalid request type")
            }
            GetLatestCertificateHeaderErrorKind::MalformedNetworkId => {
                write!(f, "Malformed network ID")
            }
            GetLatestCertificateHeaderErrorKind::MissingNetworkId => {
                write!(f, "Missing network ID")
            }
            GetLatestCertificateHeaderErrorKind::Unspecified => write!(f, "Unspecified error"),
        }
    }
}

impl From<GetLatestCertificateHeaderErrorKind> for String {
    fn from(kind: GetLatestCertificateHeaderErrorKind) -> Self {
        kind.to_string()
    }
}
