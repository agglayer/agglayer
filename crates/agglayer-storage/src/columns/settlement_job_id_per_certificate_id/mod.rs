use agglayer_types::{CertificateId, SettlementJobId};

use crate::{columns::SETTLEMENT_JOB_ID_PER_CERTIFICATE_ID_CF, schema::ColumnSchema};

/// Column family mapping a certificate to its settlement job.
///
/// ## Column definition
///
/// | key             | value             |
/// | --              | --                |
/// | `CertificateId` | `SettlementJobId` |
pub(crate) struct SettlementJobIdPerCertificateIdColumn;

impl ColumnSchema for SettlementJobIdPerCertificateIdColumn {
    type Key = CertificateId;
    type Value = SettlementJobId;

    const COLUMN_FAMILY_NAME: &'static str = SETTLEMENT_JOB_ID_PER_CERTIFICATE_ID_CF;
}
