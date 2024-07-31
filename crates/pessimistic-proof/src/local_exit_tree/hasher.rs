use serde::{Deserialize, Serialize};

use crate::keccak::{keccak256_combine, Digest as KeccakDigest};

/// A hasher used in constructing a [`super::LocalExitTree`].
pub trait Hasher {
    type Digest;

    /// Hashes two digests into one.
    fn merge(left: &Self::Digest, right: &Self::Digest) -> Self::Digest;
}

/// A Keccak hasher with a 256-bit security level.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Keccak256Hasher;

impl Hasher for Keccak256Hasher {
    type Digest = KeccakDigest;

    fn merge(left: &KeccakDigest, right: &KeccakDigest) -> KeccakDigest {
        keccak256_combine([left.as_ref(), right.as_ref()])
    }
}
