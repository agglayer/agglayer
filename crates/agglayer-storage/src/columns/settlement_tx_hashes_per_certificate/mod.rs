use agglayer_types::CertificateId;

use crate::{
    columns::{ColumnSchema, SETTLEMENT_TX_HASHSES_PER_CERTIFICATE_CF},
    types::SettlementTxHashRecord,
};

pub struct SettlementTxHashesPerCertificateColumn;

pub type Key = CertificateId;
pub type Value = SettlementTxHashRecord;

impl ColumnSchema for SettlementTxHashesPerCertificateColumn {
    type Key = Key;
    type Value = Value;

    const COLUMN_FAMILY_NAME: &'static str = SETTLEMENT_TX_HASHSES_PER_CERTIFICATE_CF;
}
