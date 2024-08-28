use reth_primitives::U256;
use tiny_keccak::{Hasher, Keccak};

use crate::{local_balance_tree::FromU256, nullifier_tree::FromBool};

/// The output type of Keccak hashing.
pub type Digest = [u8; 32];

impl FromBool for Digest {
    fn from_bool(b: bool) -> Self {
        if b {
            [
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ]
        } else {
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ]
        }
    }
}
impl FromU256 for Digest {
    fn from_u256(u: U256) -> Self {
        u.to_be_bytes()
    }
}

/// Hashes the input data using a Keccak hasher with a 256-bit security level.
pub fn keccak256(data: &[u8]) -> Digest {
    let mut hasher = Keccak::v256();
    hasher.update(data);

    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    output
}

/// Hashes the input items using a Keccak hasher with a 256-bit security level.
/// Safety: This function should only be called with fixed-size items to avoid collisions.
pub fn keccak256_combine<I, T>(items: I) -> Digest
where
    I: IntoIterator<Item = T>,
    T: AsRef<[u8]>,
{
    let mut hasher = Keccak::v256();
    for data in items {
        hasher.update(data.as_ref());
    }

    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    output
}
