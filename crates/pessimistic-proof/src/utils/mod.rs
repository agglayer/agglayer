use pessimistic_proof_core::keccak::digest::Digest;

pub mod empty_hash;
pub mod smt;

pub use pessimistic_proof_core::utils::FromBool;
pub use pessimistic_proof_core::utils::FromU256;

/// Trait for objects that can be hashed.
pub trait Hashable {
    /// Hashes the object to a [`Digest`].
    fn hash(&self) -> Digest;
}
