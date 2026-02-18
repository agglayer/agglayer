use crate::{columns::SETTLEMENT_ATTEMPT_PER_WALLET_CF, schema::ColumnSchema};

/// Column family containing the settlement attempt per wallet.
pub(crate) struct SettlementAttemptPerWalletColumn;

impl ColumnSchema for SettlementAttemptPerWalletColumn {
    type Key = crate::types::settlement::attempt_result::Key;
    type Value = crate::types::settlement::attempt_result::Value;

    const COLUMN_FAMILY_NAME: &'static str = SETTLEMENT_ATTEMPT_PER_WALLET_CF;
}
