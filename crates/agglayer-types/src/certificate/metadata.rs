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
    derive_more::Display,
    derive_more::Deref,
    derive_more::From,
    serde::Deserialize,
    serde::Serialize,
)]
#[cfg_attr(feature = "testutils", derive(arbitrary::Arbitrary))]
#[serde(transparent)]
pub struct Metadata(Digest);

impl Metadata {
    pub const ZERO: Metadata = Metadata(Digest::ZERO);

    pub const fn new(metadata: Digest) -> Metadata {
        Metadata(metadata)
    }

    pub const fn as_digest(&self) -> &Digest {
        &self.0
    }
}
