use agglayer_types::{CertificateId, SettlementTxRecord};

use crate::columns::{ColumnSchema, SETTLEMENT_TX_HASHES_PER_CERTIFICATE_CF};

pub struct SettlementTxHashesPerCertificateColumn;

pub type Key = CertificateId;
pub type Value = SettlementTxRecord;

impl ColumnSchema for SettlementTxHashesPerCertificateColumn {
    type Key = Key;
    type Value = Value;

    const COLUMN_FAMILY_NAME: &'static str = SETTLEMENT_TX_HASHES_PER_CERTIFICATE_CF;
}
