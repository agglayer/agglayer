use pessimistic_proof_core::keccak::digest::Digest;

pub mod empty_hash;
pub mod smt;

pub trait Hashable {
    fn hash(&self) -> Digest;
}
