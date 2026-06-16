use agglayer_types::{Nonce, SettlementAttempt, SettlementAttemptNumber};

use super::Error;
use crate::types::generated::agglayer::storage::v0;

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
            sender_wallet: Some(value.sender_wallet.into()),
            nonce: Some(value.nonce.into()),
            tx_hash: Some(value.hash.into()),
            submission_time: Some(prost_types::Timestamp::from(value.submission_time)),
            max_fee_per_gas: Some(value.max_fee_per_gas.into()),
            max_priority_fee_per_gas: Some(value.max_priority_fee_per_gas.into()),
        }
    }
}

impl TryFrom<v0::SettlementAttempt> for SettlementAttempt {
    type Error = Error;

    fn try_from(value: v0::SettlementAttempt) -> Result<Self, Self::Error> {
        Ok(Self {
            sender_wallet: required_field!(value, sender_wallet =>
                try_into::<agglayer_types::Address>
            ),
            nonce: required_field!(value, nonce => into::<Nonce>),
            hash: required_field!(value, tx_hash => try_into::<agglayer_types::SettlementTxHash>),
            submission_time: required_field!(value, submission_time =>
                try_into::<std::time::SystemTime>
            ),
            max_fee_per_gas: required_field!(value, max_fee_per_gas => try_into::<u128>),
            max_priority_fee_per_gas: required_field!(value, max_priority_fee_per_gas =>
                try_into::<u128>
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use agglayer_types::{Address, Digest, SettlementTxHash};

    use super::*;

    #[test]
    fn settlement_attempt_round_trip() {
        let attempt = SettlementAttempt {
            sender_wallet: Address::from([1_u8; 20]),
            nonce: Nonce(7),
            hash: SettlementTxHash::new(Digest::from([2_u8; 32])),
            submission_time: SystemTime::UNIX_EPOCH,
            max_fee_per_gas: 30_000_000_000,
            max_priority_fee_per_gas: 1_000_000_000,
        };

        let proto: v0::SettlementAttempt = (&attempt).into();
        let decoded = SettlementAttempt::try_from(proto).unwrap();

        assert_eq!(decoded.sender_wallet, attempt.sender_wallet);
        assert_eq!(decoded.nonce, attempt.nonce);
        assert_eq!(decoded.hash, attempt.hash);
        assert_eq!(decoded.submission_time, attempt.submission_time);
        assert_eq!(decoded.max_fee_per_gas, attempt.max_fee_per_gas);
        assert_eq!(
            decoded.max_priority_fee_per_gas,
            attempt.max_priority_fee_per_gas
        );
    }
}
