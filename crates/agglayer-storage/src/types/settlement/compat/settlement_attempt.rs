use agglayer_types::{Nonce, SettlementAttempt, SettlementAttemptNumber};

use super::{
    primitives::{
        parse_address, parse_settlement_tx_hash, parse_timestamp, parse_uint128_to_u128,
        to_proto_address, to_proto_settlement_tx_hash, to_proto_timestamp,
        to_proto_uint128_from_u128,
    },
    Error,
};
use crate::types::generated::agglayer::storage::v0;

fn parse_nonce(value: v0::Nonce) -> Result<Nonce, Error> {
    Ok(value.into())
}

impl From<SettlementAttemptNumber> for v0::AttemptSequenceNumber {
    fn from(value: SettlementAttemptNumber) -> Self {
        Self { number: value.0 }
    }
}

impl From<v0::AttemptSequenceNumber> for SettlementAttemptNumber {
    fn from(value: v0::AttemptSequenceNumber) -> Self {
        Self(value.number)
    }
}

impl From<Nonce> for v0::Nonce {
    fn from(value: Nonce) -> Self {
        Self { nonce: value.0 }
    }
}

impl From<v0::Nonce> for Nonce {
    fn from(value: v0::Nonce) -> Self {
        Self(value.nonce)
    }
}

impl From<&SettlementAttempt> for v0::SettlementAttempt {
    fn from(value: &SettlementAttempt) -> Self {
        Self {
            sender_wallet: Some(to_proto_address(value.sender_wallet)),
            nonce: Some(value.nonce.into()),
            max_fee_per_gas: Some(to_proto_uint128_from_u128(value.max_fee_per_gas)),
            max_priority_fee_per_gas: Some(to_proto_uint128_from_u128(
                value.max_priority_fee_per_gas,
            )),
            tx_hash: Some(to_proto_settlement_tx_hash(value.hash)),
            submission_time: Some(to_proto_timestamp(value.submission_time)),
        }
    }
}

impl From<SettlementAttempt> for v0::SettlementAttempt {
    fn from(value: SettlementAttempt) -> Self {
        (&value).into()
    }
}

impl TryFrom<v0::SettlementAttempt> for SettlementAttempt {
    type Error = Error;

    fn try_from(value: v0::SettlementAttempt) -> Result<Self, Self::Error> {
        Ok(Self {
            sender_wallet: required_field!(value, sender_wallet => parse_address),
            nonce: required_field!(value, nonce => parse_nonce),
            max_fee_per_gas: required_field!(value, max_fee_per_gas => parse_uint128_to_u128),
            max_priority_fee_per_gas: required_field!(
                value,
                max_priority_fee_per_gas => parse_uint128_to_u128
            ),
            hash: required_field!(value, tx_hash => parse_settlement_tx_hash),
            submission_time: required_field!(value, submission_time => parse_timestamp),
            result: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use agglayer_types::{
        Address, ClientError, ClientErrorType, Digest, SettlementJobResult, SettlementTxHash,
    };

    use super::*;

    #[test]
    fn settlement_attempt_round_trip_ignores_result() {
        let attempt = SettlementAttempt {
            sender_wallet: Address::from([1_u8; 20]),
            nonce: Nonce(7),
            max_fee_per_gas: 10,
            max_priority_fee_per_gas: 20,
            hash: SettlementTxHash::new(Digest::from([2_u8; 32])),
            submission_time: SystemTime::UNIX_EPOCH,
            result: Some(SettlementJobResult::ClientError(ClientError {
                kind: ClientErrorType::Unknown,
                message: "ignored".to_string(),
            })),
        };

        let proto: v0::SettlementAttempt = (&attempt).into();
        let decoded = SettlementAttempt::try_from(proto).unwrap();

        assert_eq!(decoded.sender_wallet, attempt.sender_wallet);
        assert_eq!(decoded.nonce, attempt.nonce);
        assert_eq!(decoded.max_fee_per_gas, attempt.max_fee_per_gas);
        assert_eq!(
            decoded.max_priority_fee_per_gas,
            attempt.max_priority_fee_per_gas
        );
        assert_eq!(decoded.hash, attempt.hash);
        assert_eq!(decoded.submission_time, attempt.submission_time);
        assert_eq!(decoded.result, None);
    }
}
