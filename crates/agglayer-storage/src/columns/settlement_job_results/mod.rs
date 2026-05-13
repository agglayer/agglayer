use crate::{columns::SETTLEMENT_JOB_RESULTS_CF, schema::ColumnSchema};

#[cfg(test)]
mod tests;

/// Column family containing terminal settlement job results.
pub(crate) struct SettlementJobResultsColumn;

impl ColumnSchema for SettlementJobResultsColumn {
    type Key = crate::types::settlement::job_result::Key;
    type Value = crate::types::settlement::job_result::Value;

    const COLUMN_FAMILY_NAME: &'static str = SETTLEMENT_JOB_RESULTS_CF;
}
