use crate::{
    CertificateId, CertificateIndex, EpochNumber, Height, LocalExitRoot, Metadata, NetworkId,
};

mod settlement_job_id;
mod settlement_tx_hash;
mod status;

pub use settlement_job_id::SettlementJobId;
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
    /// Settlement job ID used to track settlement until marked as Settled.
    /// Once settled, this is cleared and settlement_tx_hash is set.
    pub settlement_job_id: Option<SettlementJobId>,
    /// Settlement transaction hash. Only set when certificate is marked as Settled.
    pub settlement_tx_hash: Option<SettlementTxHash>,
}
