use agglayer_types::{CertificateId, SettlementJobId};

use crate::{columns::CERTIFICATE_ID_PER_SETTLEMENT_JOB_ID_CF, schema::ColumnSchema};

/// Reverse column family mapping a settlement job to its certificate.
///
/// ## Column definition
///
/// | key               | value           |
/// | --                | --              |
/// | `SettlementJobId` | `CertificateId` |
pub(crate) struct CertificateIdPerSettlementJobIdColumn;

impl ColumnSchema for CertificateIdPerSettlementJobIdColumn {
    type Key = SettlementJobId;
    type Value = CertificateId;

    const COLUMN_FAMILY_NAME: &'static str = CERTIFICATE_ID_PER_SETTLEMENT_JOB_ID_CF;
}
