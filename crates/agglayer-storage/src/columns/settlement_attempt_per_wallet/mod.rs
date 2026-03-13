use crate::{
    columns::{SETTLEMENT_ATTEMPT_PER_WALLET_CF, SETTLEMENT_ATTEMPT_PER_WALLET_COLUMN_OPTIONS},
    schema::ColumnSchema,
};

/// Column family containing the settlement attempt per wallet.
pub(crate) struct SettlementAttemptPerWalletColumn;

impl ColumnSchema for SettlementAttemptPerWalletColumn {
    type Key = crate::types::settlement::attempt_per_wallet::Key;
    type Value = crate::types::settlement::attempt_per_wallet::Value;

    const COLUMN_FAMILY_NAME: &'static str = SETTLEMENT_ATTEMPT_PER_WALLET_CF;
    const COLUMN_OPTIONS: crate::schema::options::ColumnOptions =
        SETTLEMENT_ATTEMPT_PER_WALLET_COLUMN_OPTIONS;
}
