use crate::Digest;

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    derive_more::Deref,
    derive_more::Display,
    derive_more::From,
    derive_more::Into,
    serde::Deserialize,
    serde::Serialize,
)]
#[cfg_attr(feature = "testutils", derive(arbitrary::Arbitrary))]
#[serde(transparent)]
pub struct CertificateId(Digest);

impl CertificateId {
    pub const fn new(id: Digest) -> CertificateId {
        CertificateId(id)
    }

    /// Creates a deterministic certificate id for tests.
    #[cfg(feature = "testutils")]
    pub fn for_test(seed: u8) -> CertificateId {
        CertificateId::new(Digest::from([seed; 32]))
    }

    pub const fn as_digest(&self) -> &Digest {
        &self.0
    }
}
