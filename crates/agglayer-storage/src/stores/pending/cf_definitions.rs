use crate::{
    columns::{
        latest_pending_certificate_per_network::LatestPendingCertificatePerNetworkColumn,
        latest_proven_certificate_per_network::LatestProvenCertificatePerNetworkColumn,
        pending_queue::PendingQueueColumn, proof_per_certificate::ProofPerCertificateColumn,
    },
    schema::ColumnDescriptor,
};

/// Definitions for the column families in the pending queue storage.
pub const PENDING_DB: &[ColumnDescriptor] = &[
    ColumnDescriptor::new::<LatestProvenCertificatePerNetworkColumn>(),
    ColumnDescriptor::new::<LatestPendingCertificatePerNetworkColumn>(),
    ColumnDescriptor::new::<PendingQueueColumn>(),
    ColumnDescriptor::new::<ProofPerCertificateColumn>(),
];
