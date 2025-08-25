use crate::certificate::Metadata;

/// Fields specific to v0 certificate.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FieldsV0 {
    pub metadata: Metadata,
}

impl FieldsV0 {
    // Empty for backwards compatibility
    pub const ID_PREIMAGE_PREFIX: &[u8] = &[];
}
