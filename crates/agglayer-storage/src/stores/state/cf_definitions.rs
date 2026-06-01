use crate::{
    columns::{
        balance_tree_per_network::BalanceTreePerNetworkColumn,
        certificate_header::CertificateHeaderColumn,
        certificate_per_network::CertificatePerNetworkColumn,
        disabled_networks::DisabledNetworksColumn,
        latest_settled_certificate_per_network::LatestSettledCertificatePerNetworkColumn,
        local_exit_tree_per_network::LocalExitTreePerNetworkColumn, metadata::MetadataColumn,
        network_info::NetworkInfoColumn, nullifier_tree_per_network::NullifierTreePerNetworkColumn,
        settlement_attempt_per_wallet::SettlementAttemptPerWalletColumn,
        settlement_attempt_results::SettlementAttemptResultsColumn,
        settlement_attempts::SettlementAttemptsColumn,
        settlement_job_results::SettlementJobResultsColumn, settlement_jobs::SettlementJobsColumn,
    },
    schema::ColumnDescriptor,
};

/// Original (V0) state-DB schema, predating the addition of
/// `disabled_networks_cf` and the settlement-related CFs. Used as the V0
/// declaration so that legacy production snapshots — which still have
/// just these eight CFs — pass the migration framework's schema gate
/// when opened by the current binary. The remaining CFs declared in
/// [`STATE_DB`] are brought in by an `ensure_cfs(STATE_DB)` step that
/// creates whatever is missing and is a no-op when everything is already
/// present.
pub const STATE_DB_V0: &[ColumnDescriptor] = &[
    ColumnDescriptor::new::<CertificateHeaderColumn>(),
    ColumnDescriptor::new::<CertificatePerNetworkColumn>(),
    ColumnDescriptor::new::<LatestSettledCertificatePerNetworkColumn>(),
    ColumnDescriptor::new::<MetadataColumn>(),
    ColumnDescriptor::new::<LocalExitTreePerNetworkColumn>(),
    ColumnDescriptor::new::<BalanceTreePerNetworkColumn>(),
    ColumnDescriptor::new::<NullifierTreePerNetworkColumn>(),
    ColumnDescriptor::new::<NetworkInfoColumn>(),
];

/// Definitions for the column families in the state storage. The
/// authoritative target schema: `init_db` ensures every CF listed here
/// exists on disk, regardless of whether the source was V0 or already at
/// the current schema.
pub const STATE_DB: &[ColumnDescriptor] = &[
    ColumnDescriptor::new::<CertificateHeaderColumn>(),
    ColumnDescriptor::new::<CertificatePerNetworkColumn>(),
    ColumnDescriptor::new::<LatestSettledCertificatePerNetworkColumn>(),
    ColumnDescriptor::new::<MetadataColumn>(),
    ColumnDescriptor::new::<LocalExitTreePerNetworkColumn>(),
    ColumnDescriptor::new::<BalanceTreePerNetworkColumn>(),
    ColumnDescriptor::new::<NullifierTreePerNetworkColumn>(),
    ColumnDescriptor::new::<NetworkInfoColumn>(),
    ColumnDescriptor::new::<DisabledNetworksColumn>(),
    // Settlement related CFs
    ColumnDescriptor::new::<SettlementJobsColumn>(),
    ColumnDescriptor::new::<SettlementJobResultsColumn>(),
    ColumnDescriptor::new::<SettlementAttemptsColumn>(),
    ColumnDescriptor::new::<SettlementAttemptResultsColumn>(),
    ColumnDescriptor::new::<SettlementAttemptPerWalletColumn>(),
];
