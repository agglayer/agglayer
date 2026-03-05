use crate::{columns::SETTLEMENT_JOBS_CF, schema::ColumnSchema};

#[cfg(test)]
mod tests;

/// Column family containing the settlement jobs.
pub(crate) struct SettlementJobsColumn;

impl ColumnSchema for SettlementJobsColumn {
    type Key = crate::types::settlement::job::Key;
    type Value = crate::types::settlement::job::Value;

    const COLUMN_FAMILY_NAME: &'static str = SETTLEMENT_JOBS_CF;
}
