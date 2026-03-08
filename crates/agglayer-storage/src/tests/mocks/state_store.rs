use agglayer_types::{
    primitives::Digest, Certificate, CertificateHeader, CertificateId, CertificateStatus,
    EpochNumber, Height, LocalNetworkStateData, NetworkId, SettlementTxHash,
};
use mockall::mock;

use crate::{
    columns::latest_settled_certificate_per_network::SettledCertificate,
    error::Error,
    stores::{
        MetadataReader, MetadataWriter, NetworkInfoReader, SettlementReader, SettlementWriter,
        StateReader, StateWriter, UpdateEvenIfAlreadyPresent, UpdateStatusToCandidate,
    },
    types::generated::agglayer::storage::v0::{
        SettlementAttempt as StoredSettlementAttempt, SettlementJob as StoredSettlementJob,
        TxResult as StoredTxResult,
    },
};
mock! {
    pub StateStore {}
    impl NetworkInfoReader for StateStore {
        fn get_network_info(&self, network_id: NetworkId) -> Result<agglayer_types::NetworkInfo, Error>;

        fn get_latest_pending_height(&self, network_id: NetworkId) -> Result<Option<Height>, Error>;

        fn get_latest_settled_certificate_id(
            &self,
            network_id: NetworkId,
        ) -> Result<Option<CertificateId>, Error>;
    }

    impl MetadataReader for StateStore {
        fn get_latest_settled_epoch(&self) -> Result<Option<EpochNumber>, Error>;
    }

    impl MetadataWriter for StateStore {
        fn set_latest_settled_epoch(&self, value: EpochNumber) -> Result<(), Error>;
    }

    impl StateWriter for StateStore {
        fn update_settlement_tx_hash(
            &self,
            certificate_id: &CertificateId,
            tx_hash: SettlementTxHash,
            force: UpdateEvenIfAlreadyPresent,
            set_status: UpdateStatusToCandidate,
        ) -> Result<(), Error>;

        fn remove_settlement_tx_hash(
            &self,
            certificate_id: &CertificateId,
        ) -> Result<(), Error>;

        fn assign_certificate_to_epoch(
            &self,
            certificate_id: &CertificateId,
            epoch_number: &EpochNumber,
            certificate_index: &agglayer_types::CertificateIndex,
        ) -> Result<(), Error>;

        fn insert_certificate_header(
            &self,
            certificate: &Certificate,
            status: CertificateStatus,
        ) -> Result<(), Error>;

        fn update_certificate_header_status(
            &self,
            certificate_id: &CertificateId,
            status: &CertificateStatus,
        ) -> Result<(), Error>;

        fn set_latest_settled_certificate_for_network(
            &self,
            network_id: &NetworkId,
            height: &Height,
            certificate_id: &CertificateId,
            epoch_number: &EpochNumber,
            certificate_index: &agglayer_types::CertificateIndex
        ) -> Result<(), Error>;

        fn write_local_network_state(
            &self,
            network_id: &NetworkId,
            new_state: &LocalNetworkStateData,
            new_leaves: &[Digest],
        ) -> Result<(), Error>;

        fn disable_network(
            &self,
            network_id: &NetworkId,
            disabled_by: agglayer_types::network_info::DisabledBy,
        ) -> Result<(), Error>;
        fn enable_network(&self, network_id: &NetworkId) -> Result<(), Error>;
    }

    impl StateReader for StateStore {
        fn get_disabled_networks(&self) -> Result<Vec<NetworkId>, Error>;
        fn is_network_disabled(&self, network_id: &NetworkId) -> Result<bool, Error>;
        fn get_active_networks(&self) -> Result<Vec<NetworkId>, Error>;

        fn get_latest_settled_certificate_per_network(
            &self,
            network_id: &NetworkId,
        ) -> Result<Option<(NetworkId, SettledCertificate)>, Error>;

        fn get_certificate_header(
            &self,
            certificate_id: &CertificateId,
        ) -> Result<Option<CertificateHeader>, Error>;

        fn get_certificate_header_by_cursor(
            &self,
            network_id: NetworkId,
            height: Height,
        ) -> Result<Option<CertificateHeader>, Error>;
        fn get_current_settled_height(&self) -> Result<Vec<(NetworkId, SettledCertificate)>, Error>;

        fn read_local_network_state(
            &self,
            network_id: NetworkId,
        ) -> Result<Option<LocalNetworkStateData>, Error>;
    }

    impl SettlementReader for StateStore {
        fn get_settlement_job(
            &self,
            settlement_job_id: &ulid::Ulid,
        ) -> Result<Option<StoredSettlementJob>, Error>;

        fn get_settlement_attempt(
            &self,
            settlement_job_id: &ulid::Ulid,
            attempt_sequence_number: u64,
        ) -> Result<Option<StoredSettlementAttempt>, Error>;

        fn get_settlement_attempt_result(
            &self,
            settlement_job_id: &ulid::Ulid,
            attempt_sequence_number: u64,
        ) -> Result<Option<StoredTxResult>, Error>;

        fn list_settlement_attempts(
            &self,
            settlement_job_id: &ulid::Ulid,
        ) -> Result<Vec<(u64, StoredSettlementAttempt)>, Error>;

        fn list_settlement_attempt_results(
            &self,
            settlement_job_id: &ulid::Ulid,
        ) -> Result<Vec<(u64, StoredTxResult)>, Error>;
    }

    impl SettlementWriter for StateStore {
        fn insert_settlement_job(
            &self,
            settlement_job_id: &ulid::Ulid,
            settlement_job: &StoredSettlementJob,
        ) -> Result<(), Error>;

        fn insert_settlement_attempt(
            &self,
            settlement_job_id: &ulid::Ulid,
            attempt_sequence_number: u64,
            settlement_attempt: &StoredSettlementAttempt,
        ) -> Result<(), Error>;

        fn insert_settlement_attempt_result(
            &self,
            settlement_job_id: &ulid::Ulid,
            attempt_sequence_number: u64,
            tx_result: &StoredTxResult,
        ) -> Result<(), Error>;
    }
}
