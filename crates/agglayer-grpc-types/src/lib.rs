#[allow(clippy::needless_lifetimes)]
mod generated;

pub use crate::generated::agglayer::*;

#[cfg(feature = "compat")]
pub mod compat;
