use crate::{
    columns::epochs::{
        certificates::{CertificatePerIndexColumn, CertificatePerIndexProtoColumn},
        end_checkpoint::EndCheckpointColumn,
        metadata::PerEpochMetadataColumn,
        proofs::ProofPerIndexColumn,
        start_checkpoint::StartCheckpointColumn,
    },
    schema::ColumnDescriptor,
};

/// Legacy definitions for the column families in the epochs storage.
pub const EPOCHS_DB_V0: &[ColumnDescriptor] = &[
    ColumnDescriptor::new::<CertificatePerIndexColumn>(),
    ColumnDescriptor::new::<PerEpochMetadataColumn>(),
    ColumnDescriptor::new::<ProofPerIndexColumn>(),
    ColumnDescriptor::new::<StartCheckpointColumn>(),
    ColumnDescriptor::new::<EndCheckpointColumn>(),
];

/// Definitions for the column families in the epochs storage.
pub const EPOCHS_DB: &[ColumnDescriptor] = &[
    ColumnDescriptor::new::<CertificatePerIndexColumn>(),
    ColumnDescriptor::new::<CertificatePerIndexProtoColumn>(),
    ColumnDescriptor::new::<PerEpochMetadataColumn>(),
    ColumnDescriptor::new::<ProofPerIndexColumn>(),
    ColumnDescriptor::new::<StartCheckpointColumn>(),
    ColumnDescriptor::new::<EndCheckpointColumn>(),
];
