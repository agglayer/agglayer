use crate::{
    CertificateId, CertificateIndex, EpochNumber, Height, LocalExitRoot, Metadata, NetworkId,
};

mod settlement_tx_hash;
mod status;

pub use settlement_tx_hash::SettlementTxHash;
pub use status::CertificateStatus;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct CertificateHeader {
    pub network_id: NetworkId,
    pub height: Height,
    pub epoch_number: Option<EpochNumber>,
    pub certificate_index: Option<CertificateIndex>,
    pub certificate_id: CertificateId,
    pub prev_local_exit_root: LocalExitRoot,
    pub new_local_exit_root: LocalExitRoot,
    pub metadata: Metadata,
    pub status: CertificateStatus,
    pub settlement_tx_hash: Option<SettlementTxHash>,
}

#[cfg(feature = "testutils")]
impl CertificateHeader {
    /// Generate a CertificateHeader for testing using the provided seed.
    ///
    /// Rules:
    /// - `settlement_tx_hash` is only set if status is Candidate or Settled
    /// - `epoch_number` and `certificate_index` are only set if status is
    ///   Settled
    pub fn generate_for_test(
        seed: u64,
        network_id: NetworkId,
        height: Height,
        certificate_id: CertificateId,
        prev_local_exit_root: LocalExitRoot,
        new_local_exit_root: LocalExitRoot,
    ) -> Self {
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        // Generate random metadata
        let metadata = Metadata::new(crate::Digest(rng.random::<[u8; 32]>()));

        // Generate random status using generate_for_test
        let status = CertificateStatus::generate_for_test(seed);

        // Set settlement_tx_hash only for Candidate or Settled
        let settlement_tx_hash = match status {
            CertificateStatus::Candidate | CertificateStatus::Settled => Some(
                SettlementTxHash::new(crate::Digest(rng.random::<[u8; 32]>())),
            ),
            _ => None,
        };

        // Set epoch_number and certificate_index only for Settled
        let (epoch_number, certificate_index) = match status {
            CertificateStatus::Settled => {
                let epoch = EpochNumber::new(rng.random_range(0..100));
                let index = CertificateIndex::new(rng.random_range(0..10));
                (Some(epoch), Some(index))
            }
            _ => (None, None),
        };

        Self {
            network_id,
            height,
            epoch_number,
            certificate_index,
            certificate_id,
            prev_local_exit_root,
            new_local_exit_root,
            metadata,
            status,
            settlement_tx_hash,
        }
    }
}
