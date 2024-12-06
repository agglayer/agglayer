use serde::{Deserialize, Serialize};

use crate::keccak::{digest::NewDigest, new_keccak256_combine};

/// A hasher used in constructing a [`super::LocalExitTree`].
pub trait Hasher {
    type Digest;

    /// Hashes two digests into one.
    fn merge(left: &Self::Digest, right: &Self::Digest) -> Self::Digest;
}

// pub type NewKeccak256Hasher = Keccak256Hasher;
/// A Keccak hasher with a 256-bit security level.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct NewKeccak256Hasher;

impl Hasher for NewKeccak256Hasher {
    type Digest = NewDigest;

    fn merge(left: &Self::Digest, right: &Self::Digest) -> Self::Digest {
        new_keccak256_combine([left.as_ref(), right.as_ref()]).into()
    }
}
