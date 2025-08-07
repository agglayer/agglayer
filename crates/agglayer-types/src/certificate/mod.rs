mod common;
mod header;
mod height;
mod id;
mod index;
mod metadata;
#[cfg(feature = "testutils")]
mod testutils;
mod v0;
mod v1;

pub use common::Fields;
pub use header::{CertificateHeader, CertificateStatus, SettlementTxHash};
pub use height::Height;
pub use id::CertificateId;
pub use index::CertificateIndex;
pub use metadata::Metadata;
#[cfg(feature = "testutils")]
pub use testutils::compute_signature_info;
pub use v0::CertificateV0;
pub use v1::CertificateV1;

// Make V0 as the current certificate for now.
// Should be eventually replaced with CertificateVx.
pub type Certificate = CertificateV0;

/// Certificate of any supported version
#[derive(Clone, Debug, derive_more::From)]
pub enum CertificateVx {
    V0(CertificateV0),
    V1(CertificateV1),
}

impl CertificateVx {
    pub fn version(&self) -> Version {
        match self {
            Self::V0(_) => Version::V0,
            Self::V1(_) => Version::V1,
        }
    }

    pub fn hash(&self) -> CertificateId {
        match self {
            Self::V0(cert) => cert.hash(),
            Self::V1(cert) => cert.hash(),
        }
    }

    // TODO re-expose various certificate methods here in fashion similar to
    // [Self::hash].
    //
    // It could also be considered to abstract these methods into a trait.
    // Some code duplication could be avoided by:
    // 1. Factoring out common parts of implementation,
    // 2. Introducing a trait with common "core" methods and an extension trait that
    //    exposes methods implemented in terms of the core ones.
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Version {
    V0,
    V1,
}

impl Version {
    pub fn as_u32(&self) -> u32 {
        match self {
            Self::V0 => 0,
            Self::V1 => 1,
        }
    }
}
