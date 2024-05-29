use tiny_keccak::{Hasher, Keccak};

/// The output type of Keccak hashing.
pub type Digest = [u8; 32];

/// Hashes the input data using a Keccak hasher with a 256-bit security level.
pub fn keccak256(data: &[u8]) -> Digest {
    let mut hasher = Keccak::v256();
    hasher.update(data);

    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    output
}

/// Hashes the input items using a Keccak hasher with a 256-bit security level.
pub fn keccak256_combine<'a, I>(items: I) -> Digest
where
    I: IntoIterator<Item = &'a [u8]>,
{
    let mut hasher = Keccak::v256();
    for data in items {
        hasher.update(data);
    }

    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    output
}
