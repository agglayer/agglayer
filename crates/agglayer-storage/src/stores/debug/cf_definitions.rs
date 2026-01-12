use crate::{columns::debug_certificates::DebugCertificatesColumn, schema::ColumnDescriptor};

/// Definitions for the column families in the debug storage.
pub const DEBUG_DB: &[ColumnDescriptor] = &[ColumnDescriptor::new::<DebugCertificatesColumn>()];
