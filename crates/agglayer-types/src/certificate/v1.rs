use alloy::primitives::hex;

/// Fields specific to v1 certificate.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FieldsV1 {}

impl FieldsV1 {
    // The original position of network ID and height plugged with
    // all ones, followed by two-byte version,
    pub const ID_PREIMAGE_PREFIX: &[u8] = &hex!("FFFFFFFF FFFFFFFFFFFFFFFF 0001");
}
