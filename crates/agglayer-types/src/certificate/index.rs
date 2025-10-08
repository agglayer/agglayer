/// Index of the certificate inside its epoch
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    derive_more::Display,
    serde::Deserialize,
    serde::Serialize,
)]
#[cfg_attr(feature = "testutils", derive(arbitrary::Arbitrary))]
#[serde(transparent)]
pub struct CertificateIndex(u64);

impl CertificateIndex {
    pub const ZERO: CertificateIndex = CertificateIndex(0);

    pub const fn new(index: u64) -> CertificateIndex {
        CertificateIndex(index)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}
