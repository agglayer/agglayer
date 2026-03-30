use crate::{columns::SETTLEMENT_JOB_ID_PER_CERTIFICATE_CF, schema::ColumnSchema};

#[cfg(test)]
mod tests;

/// Column family containing the settlement job id per certificate.
pub(crate) struct SettlementJobIdPerCertificateColumn;

impl ColumnSchema for SettlementJobIdPerCertificateColumn {
    type Key = crate::types::settlement::job_id_per_certificate::Key;
    type Value = crate::types::settlement::job_id_per_certificate::Value;

    const COLUMN_FAMILY_NAME: &'static str = SETTLEMENT_JOB_ID_PER_CERTIFICATE_CF;
}
