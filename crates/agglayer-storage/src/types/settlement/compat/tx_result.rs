use agglayer_types::{SettlementAttemptResult, SettlementJobResult};

use super::Error;
use crate::types::generated::agglayer::storage::v0;

enum SettlementTxResultKind {
    ClientError(agglayer_types::ClientError),
    ContractCall(agglayer_types::ContractCallResult),
}

impl TryFrom<v0::tx_result::TxResult> for SettlementTxResultKind {
    type Error = Error;

    fn try_from(value: v0::tx_result::TxResult) -> Result<Self, Self::Error> {
        match value {
            v0::tx_result::TxResult::ClientError(client_error) => {
                Ok(Self::ClientError(client_error.try_into()?))
            }
            v0::tx_result::TxResult::ContractCallResult(contract_call_result) => {
                Ok(Self::ContractCall(contract_call_result.try_into()?))
            }
        }
    }
}

impl TryFrom<v0::TxResult> for SettlementTxResultKind {
    type Error = Error;

    fn try_from(value: v0::TxResult) -> Result<Self, Self::Error> {
        value
            .tx_result
            .ok_or_else(|| Error::missing_field("tx_result"))?
            .try_into()
    }
}

impl From<SettlementTxResultKind> for SettlementJobResult {
    fn from(value: SettlementTxResultKind) -> Self {
        match value {
            SettlementTxResultKind::ClientError(client_error) => Self::ClientError(client_error),
            SettlementTxResultKind::ContractCall(contract_call_result) => {
                Self::ContractCall(contract_call_result)
            }
        }
    }
}

impl From<SettlementTxResultKind> for SettlementAttemptResult {
    fn from(value: SettlementTxResultKind) -> Self {
        match value {
            SettlementTxResultKind::ClientError(client_error) => Self::ClientError(client_error),
            SettlementTxResultKind::ContractCall(contract_call_result) => {
                Self::ContractCall(contract_call_result)
            }
        }
    }
}

impl From<&SettlementJobResult> for v0::TxResult {
    fn from(value: &SettlementJobResult) -> Self {
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

impl From<&SettlementAttemptResult> for v0::TxResult {
    fn from(value: &SettlementAttemptResult) -> Self {
        let tx_result = match value {
            SettlementAttemptResult::ClientError(client_error) => {
                v0::tx_result::TxResult::ClientError(client_error.into())
            }
            SettlementAttemptResult::ContractCall(contract_call_result) => {
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
        Ok(SettlementTxResultKind::try_from(value)?.into())
    }
}

impl TryFrom<v0::TxResult> for SettlementAttemptResult {
    type Error = Error;

    fn try_from(value: v0::TxResult) -> Result<Self, Self::Error> {
        Ok(SettlementTxResultKind::try_from(value)?.into())
    }
}

#[cfg(test)]
mod tests {
    use agglayer_types::{
        ClientError, ContractCallOutcome, ContractCallResult, Digest, Nonce, SettlementTxHash, B256,
    };

    use super::*;

    fn sample_client_error() -> ClientError {
        ClientError::nonce_already_used(
            agglayer_types::Address::from([1_u8; 20]),
            Nonce(7),
            SettlementTxHash::new(Digest::from([2_u8; 32])),
        )
    }

    fn sample_contract_call_result() -> ContractCallResult {
        ContractCallResult {
            outcome: ContractCallOutcome::Success,
            metadata: vec![3, 4, 5].into(),
            block_hash: B256::from([6_u8; 32]),
            block_number: 42,
            tx_hash: SettlementTxHash::new(Digest::from([7_u8; 32])),
        }
    }

    #[test]
    fn missing_oneof_fails() {
        let proto = v0::TxResult { tx_result: None };

        assert!(SettlementJobResult::try_from(proto).is_err());
    }

    #[test]
    fn missing_oneof_fails_for_attempt_result() {
        let proto = v0::TxResult { tx_result: None };

        assert!(SettlementAttemptResult::try_from(proto).is_err());
    }

    #[test]
    fn job_and_attempt_contract_call_encode_same_proto() {
        let contract_call = sample_contract_call_result();

        let from_job: v0::TxResult =
            (&SettlementJobResult::ContractCall(contract_call.clone())).into();
        let from_attempt: v0::TxResult =
            (&SettlementAttemptResult::ContractCall(contract_call)).into();

        assert_eq!(from_job, from_attempt);
    }

    #[test]
    fn job_and_attempt_client_error_encode_same_proto() {
        let client_error = sample_client_error();

        let from_job: v0::TxResult =
            (&SettlementJobResult::ClientError(client_error.clone())).into();
        let from_attempt: v0::TxResult =
            (&SettlementAttemptResult::ClientError(client_error)).into();

        assert_eq!(from_job, from_attempt);
    }

    #[test]
    fn round_trip_contract_call_for_both_result_types() {
        let attempt_result = SettlementAttemptResult::ContractCall(sample_contract_call_result());
        let proto: v0::TxResult = (&attempt_result).into();

        let job_result = SettlementJobResult::try_from(proto.clone()).unwrap();
        let attempt_result = SettlementAttemptResult::try_from(proto).unwrap();

        assert_eq!(
            job_result,
            SettlementJobResult::ContractCall(sample_contract_call_result())
        );
        assert_eq!(
            attempt_result,
            SettlementAttemptResult::ContractCall(sample_contract_call_result())
        );
    }

    #[test]
    fn round_trip_client_error_for_both_result_types() {
        let attempt_result = SettlementAttemptResult::ClientError(sample_client_error());
        let proto: v0::TxResult = (&attempt_result).into();

        let job_result = SettlementJobResult::try_from(proto.clone()).unwrap();
        let attempt_result = SettlementAttemptResult::try_from(proto).unwrap();

        assert_eq!(
            job_result,
            SettlementJobResult::ClientError(sample_client_error())
        );
        assert_eq!(
            attempt_result,
            SettlementAttemptResult::ClientError(sample_client_error())
        );
    }
}
