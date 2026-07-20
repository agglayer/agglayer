use agglayer_types::{SettlementAttemptResult, SettlementJobResult};

use super::Error;
use crate::types::generated::agglayer::storage::v0;

impl TryFrom<v0::settlement_attempt_result::Result> for SettlementAttemptResult {
    type Error = Error;

    fn try_from(value: v0::settlement_attempt_result::Result) -> Result<Self, Self::Error> {
        match value {
            v0::settlement_attempt_result::Result::ClientError(client_error) => {
                Ok(Self::ClientError(client_error.try_into()?))
            }
            v0::settlement_attempt_result::Result::ContractCallResult(contract_call_result) => {
                Ok(Self::ContractCall(contract_call_result.try_into()?))
            }
        }
    }
}

impl From<&SettlementAttemptResult> for v0::SettlementAttemptResult {
    fn from(value: &SettlementAttemptResult) -> Self {
        let result = match value {
            SettlementAttemptResult::ClientError(client_error) => {
                v0::settlement_attempt_result::Result::ClientError(client_error.into())
            }
            SettlementAttemptResult::ContractCall(contract_call_result) => {
                v0::settlement_attempt_result::Result::ContractCallResult(
                    contract_call_result.into(),
                )
            }
        };

        Self {
            result: Some(result),
        }
    }
}

impl TryFrom<v0::SettlementAttemptResult> for SettlementAttemptResult {
    type Error = Error;

    fn try_from(value: v0::SettlementAttemptResult) -> Result<Self, Self::Error> {
        value
            .result
            .ok_or_else(|| Error::missing_field("result"))?
            .try_into()
    }
}

impl From<&SettlementJobResult> for v0::SettlementJobResult {
    fn from(value: &SettlementJobResult) -> Self {
        Self {
            wallet: Some(value.wallet.into()),
            nonce: Some(value.nonce.into()),
            attempt_number: Some(value.attempt_number.into()),
            contract_call_result: Some((&value.contract_call_result).into()),
        }
    }
}

impl TryFrom<v0::SettlementJobResult> for SettlementJobResult {
    type Error = Error;

    fn try_from(value: v0::SettlementJobResult) -> Result<Self, Self::Error> {
        Ok(SettlementJobResult {
            wallet: required_field!(value, wallet => try_into::<agglayer_types::Address>),
            nonce: required_field!(value, nonce => into::<agglayer_types::Nonce>),
            attempt_number: required_field!(value, attempt_number =>
                into::<agglayer_types::SettlementAttemptNumber>
            ),
            contract_call_result: required_field!(value, contract_call_result =>
                try_into::<agglayer_types::ContractCallResult>
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use agglayer_types::{
        ClientError, ContractCallOutcome, ContractCallResult, Digest, Nonce,
        SettlementAttemptNumber, SettlementTxHash, B256,
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
    fn missing_oneof_fails_for_attempt_result() {
        let proto = v0::SettlementAttemptResult { result: None };

        assert!(SettlementAttemptResult::try_from(proto).is_err());
    }

    #[test]
    fn attempt_result_round_trip_contract_call() {
        let attempt_result = SettlementAttemptResult::ContractCall(sample_contract_call_result());
        let proto: v0::SettlementAttemptResult = (&attempt_result).into();

        let decoded = SettlementAttemptResult::try_from(proto).unwrap();

        assert_eq!(
            decoded,
            SettlementAttemptResult::ContractCall(sample_contract_call_result())
        );
    }

    #[test]
    fn attempt_result_round_trip_client_error() {
        let attempt_result = SettlementAttemptResult::ClientError(sample_client_error());
        let proto: v0::SettlementAttemptResult = (&attempt_result).into();

        let decoded = SettlementAttemptResult::try_from(proto).unwrap();

        assert_eq!(
            decoded,
            SettlementAttemptResult::ClientError(sample_client_error())
        );
    }

    #[test]
    fn missing_contract_call_fails_for_job_result() {
        let proto = v0::SettlementJobResult {
            wallet: Some(agglayer_types::Address::from([1_u8; 20]).into()),
            nonce: Some(Nonce(7).into()),
            attempt_number: Some(SettlementAttemptNumber(8).into()),
            contract_call_result: None,
        };

        let result: Result<SettlementJobResult, _> = proto.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn job_result_round_trip_contract_call() {
        let job_result = SettlementJobResult {
            wallet: agglayer_types::Address::from([9_u8; 20]),
            nonce: Nonce(7),
            attempt_number: SettlementAttemptNumber(8),
            contract_call_result: sample_contract_call_result(),
        };
        let proto: v0::SettlementJobResult = (&job_result).into();

        let decoded = SettlementJobResult::try_from(proto).unwrap();

        assert_eq!(decoded, job_result);
    }
}
