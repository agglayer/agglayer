use pessimistic_proof_core::keccak::digest::Digest;

pub mod empty_hash;
pub mod smt;

/// Trait for objects that can be hashed.
pub trait Hashable {
    /// Hashes the object to a [`Digest`].
    fn hash(&self) -> Digest;
}
