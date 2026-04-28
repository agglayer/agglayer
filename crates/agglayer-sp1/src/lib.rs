//! Thin facade for typed SP1 SDK usage used by the Agglayer node.
//!
//! All crates outside of the pessimistic-proof family and this one
//! should avoid direct `sp1-*` deps. Go through these helpers instead.

mod error;
mod ext;
mod mode;
mod policy;
#[cfg(any(test, feature = "testutils"))]
pub mod testutils;
mod version;

pub use error::ProofError;
pub use ext::{
    current_sp1_stark_with_context, CurrentSp1StarkProof, CurrentSp1StarkWithContext, ProofExt,
};
pub use mode::ProofMode;
pub use policy::AcceptancePolicy;
pub use version::{version_kind, Sp1ProofVersion};
