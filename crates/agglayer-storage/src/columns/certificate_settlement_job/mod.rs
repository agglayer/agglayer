use agglayer_types::{CertificateId, SettlementJobId};

use crate::{columns::CERTIFICATE_SETTLEMENT_JOB_CF, schema::ColumnSchema};

/// Column family mapping certificates to their settlement jobs.
///
/// ## Column definition
///
/// | key             | value             |
/// | --              | --                |
/// | `CertificateId` | `SettlementJobId` |
pub(crate) struct CertificateSettlementJobColumn;

impl ColumnSchema for CertificateSettlementJobColumn {
    type Key = CertificateId;
    type Value = SettlementJobId;

    const COLUMN_FAMILY_NAME: &'static str = CERTIFICATE_SETTLEMENT_JOB_CF;
}
