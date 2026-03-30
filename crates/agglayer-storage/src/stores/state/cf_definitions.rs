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
        settlement_job_id_per_certificate::SettlementJobIdPerCertificateColumn,
        settlement_job_results::SettlementJobResultsColumn, settlement_jobs::SettlementJobsColumn,
    },
    schema::ColumnDescriptor,
};

/// Definitions for the column families in the state storage.
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
    ColumnDescriptor::new::<SettlementJobIdPerCertificateColumn>(),
    ColumnDescriptor::new::<SettlementJobsColumn>(),
    ColumnDescriptor::new::<SettlementJobResultsColumn>(),
    ColumnDescriptor::new::<SettlementAttemptsColumn>(),
    ColumnDescriptor::new::<SettlementAttemptResultsColumn>(),
    ColumnDescriptor::new::<SettlementAttemptPerWalletColumn>(),
];

/// Migration step 1: add certificate -> settlement job mapping.
pub const STATE_DB_MIGRATION_STEP_1_ADD_CFS: &[ColumnDescriptor] =
    &[ColumnDescriptor::new::<SettlementJobIdPerCertificateColumn>()];
