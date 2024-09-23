pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("generated/agglayer.prover.bin");

#[path = "generated/agglayer.prover.v1.rs"]
#[rustfmt::skip]
#[allow(warnings)]
pub mod v1;
