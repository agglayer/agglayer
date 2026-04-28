use crate::{
    columns::debug_certificates::{DebugCertificatesColumn, DebugCertificatesProtoColumn},
    schema::ColumnDescriptor,
};

/// Legacy definitions for the column families in the debug storage.
pub const DEBUG_DB_V0: &[ColumnDescriptor] = &[ColumnDescriptor::new::<DebugCertificatesColumn>()];

/// Definitions for the column families in the debug storage.
pub const DEBUG_DB: &[ColumnDescriptor] = &[
    ColumnDescriptor::new::<DebugCertificatesColumn>(),
    ColumnDescriptor::new::<DebugCertificatesProtoColumn>(),
];

pub const DEBUG_CERTIFICATE_PROTO_CFS: &[ColumnDescriptor] =
    &[ColumnDescriptor::new::<DebugCertificatesProtoColumn>()];
