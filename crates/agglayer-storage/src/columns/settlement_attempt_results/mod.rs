use crate::{
    columns::{SETTLEMENT_ATTEMPT_RESULTS_CF, SETTLEMENT_ATTEMPT_RESULTS_COLUMN_OPTIONS},
    schema::ColumnSchema,
};

#[cfg(test)]
mod tests;

/// Column family containing the settlement attempt results.
pub(crate) struct SettlementAttemptResultsColumn;

impl ColumnSchema for SettlementAttemptResultsColumn {
    type Key = crate::types::settlement::attempt_result::Key;
    type Value = crate::types::settlement::attempt_result::Value;

    const COLUMN_FAMILY_NAME: &'static str = SETTLEMENT_ATTEMPT_RESULTS_CF;
    const COLUMN_OPTIONS: crate::schema::options::ColumnOptions =
        SETTLEMENT_ATTEMPT_RESULTS_COLUMN_OPTIONS;
}
