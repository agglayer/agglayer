use crate::{
    columns::{
        latest_pending_certificate_per_network::LatestPendingCertificatePerNetworkColumn,
        latest_proven_certificate_per_network::LatestProvenCertificatePerNetworkColumn,
        pending_queue::{PendingQueueColumn, PendingQueueProtoColumn},
        proof_per_certificate::{ProofPerCertificateColumn, ProofPerCertificateProtoColumn},
    },
    schema::ColumnDescriptor,
};

/// Legacy definitions for the pending queue storage.
pub const PENDING_DB_V0: &[ColumnDescriptor] = &[
    ColumnDescriptor::new::<LatestProvenCertificatePerNetworkColumn>(),
    ColumnDescriptor::new::<LatestPendingCertificatePerNetworkColumn>(),
    ColumnDescriptor::new::<PendingQueueColumn>(),
    ColumnDescriptor::new::<ProofPerCertificateColumn>(),
];

/// Definitions for the pending queue storage.
pub const PENDING_DB: &[ColumnDescriptor] = &[
    ColumnDescriptor::new::<LatestProvenCertificatePerNetworkColumn>(),
    ColumnDescriptor::new::<LatestPendingCertificatePerNetworkColumn>(),
    ColumnDescriptor::new::<PendingQueueColumn>(),
    ColumnDescriptor::new::<PendingQueueProtoColumn>(),
    ColumnDescriptor::new::<ProofPerCertificateColumn>(),
    ColumnDescriptor::new::<ProofPerCertificateProtoColumn>(),
];
