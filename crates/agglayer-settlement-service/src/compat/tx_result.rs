use agglayer_storage::types::generated::agglayer::storage::v0;

use super::Error;
use crate::settlement_task::SettlementJobResult;

impl From<SettlementJobResult> for v0::TxResult {
    fn from(value: SettlementJobResult) -> Self {
        let tx_result = match value {
            SettlementJobResult::ClientError(client_error) => {
                v0::tx_result::TxResult::ClientError(client_error.into())
            }
            SettlementJobResult::ContractCall(contract_call_result) => {
                v0::tx_result::TxResult::ContractCallResult(contract_call_result.into())
            }
        };

        Self {
            tx_result: Some(tx_result),
        }
    }
}

impl TryFrom<v0::TxResult> for SettlementJobResult {
    type Error = Error;

    fn try_from(value: v0::TxResult) -> Result<Self, Self::Error> {
        match value
            .tx_result
            .ok_or_else(|| Error::missing_field("tx_result"))?
        {
            v0::tx_result::TxResult::ClientError(client_error) => {
                Ok(SettlementJobResult::ClientError(client_error.try_into()?))
            }
            v0::tx_result::TxResult::ContractCallResult(contract_call_result) => Ok(
                SettlementJobResult::ContractCall(contract_call_result.try_into()?),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_oneof_fails() {
        let proto = v0::TxResult { tx_result: None };

        assert!(SettlementJobResult::try_from(proto).is_err());
    }
}
