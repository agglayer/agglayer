use crate::{columns::SETTLEMENT_ATTEMPTS_CF, schema::ColumnSchema};

#[cfg(test)]
mod tests;

/// Column family containing the settlement attempts.
pub(crate) struct SettlementAttemptsColumn;

impl ColumnSchema for SettlementAttemptsColumn {
    type Key = crate::types::settlement::attempt::Key;
    type Value = crate::types::settlement::attempt::Value;

    const COLUMN_FAMILY_NAME: &'static str = SETTLEMENT_ATTEMPTS_CF;
}
