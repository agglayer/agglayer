use crate::{
    columns::{
        balance_tree_per_network::BalanceTreePerNetworkColumn,
        certificate_header::CertificateHeaderColumn,
        certificate_per_network::CertificatePerNetworkColumn,
        disabled_networks::DisabledNetworksColumn,
        latest_settled_certificate_per_network::LatestSettledCertificatePerNetworkColumn,
        local_exit_tree_per_network::LocalExitTreePerNetworkColumn, metadata::MetadataColumn,
        network_info::NetworkInfoColumn, nullifier_tree_per_network::NullifierTreePerNetworkColumn,
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
];
