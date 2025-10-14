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
    pub settlement_tx_hashes: Vec<SettlementTxHash>,
}
