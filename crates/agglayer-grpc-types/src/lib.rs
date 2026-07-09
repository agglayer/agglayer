#[allow(clippy::needless_lifetimes)]
#[allow(clippy::useless_borrows_in_formatting)]
mod generated;

pub use crate::generated::agglayer::*;

#[cfg(feature = "compat")]
pub mod compat;
