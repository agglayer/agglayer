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
    #[serde(flatten, with = "settlement_tx_hashes_serde")]
    pub settlement_tx_hashes: Vec<SettlementTxHash>,
}

// Backwards compatible serde serialization for settlement tx hashes.
// Only works with self-describing formats.
mod settlement_tx_hashes_serde {
    use std::borrow::Cow;

    use serde::{Deserialize, Serialize};

    use super::SettlementTxHash;

    #[derive(Debug, Serialize, Deserialize)]
    struct Format<'a> {
        #[serde(default)]
        settlement_tx_hash: Option<SettlementTxHash>,
        #[serde(default)]
        settlement_tx_hashes: Option<Cow<'a, [SettlementTxHash]>>,
    }

    pub fn serialize<S: serde::Serializer>(
        hashes: &[SettlementTxHash],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let format = Format {
            settlement_tx_hash: hashes.last().copied(),
            settlement_tx_hashes: Some(Cow::Borrowed(hashes)),
        };
        format.serialize(serializer)
    }

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<SettlementTxHash>, D::Error> {
        let format = Format::deserialize(deserializer)?;
        // If both "settlement_tx_hash" and "settlement_tx_hashes" are present,
        // the new "_hashes" format takes precedence.
        Ok(format.settlement_tx_hashes.map_or_else(
            || format.settlement_tx_hash.into_iter().collect(),
            Cow::into_owned,
        ))
    }
}
