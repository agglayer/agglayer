use agglayer_storage::types::generated::agglayer::storage::v0;
use alloy::primitives::Bytes as AlloyBytes;
use prost::bytes::Bytes as ProstBytes;

use super::{
    primitives::{
        parse_block_hash, parse_block_number, parse_settlement_tx_hash, to_proto_block_hash,
        to_proto_block_number, to_proto_settlement_tx_hash,
    },
    Error,
};
use crate::settlement_task::{ContractCallOutcome, ContractCallResult};

fn parse_contract_call_metadata(value: v0::ContractCallMetadata) -> Result<AlloyBytes, Error> {
    Ok(AlloyBytes::from(value.metadata.to_vec()))
}

impl From<ContractCallOutcome> for v0::ContractCallOutcome {
    fn from(value: ContractCallOutcome) -> Self {
        match value {
            ContractCallOutcome::Success => v0::ContractCallOutcome::Success,
            ContractCallOutcome::Revert => v0::ContractCallOutcome::Reverted,
        }
    }
}

impl TryFrom<v0::ContractCallOutcome> for ContractCallOutcome {
    type Error = Error;

    fn try_from(value: v0::ContractCallOutcome) -> Result<Self, Self::Error> {
        match value {
            v0::ContractCallOutcome::Unspecified => Err(Error::invalid_data(
                "contract call outcome must be specified",
            )),
            v0::ContractCallOutcome::Success => Ok(ContractCallOutcome::Success),
            v0::ContractCallOutcome::Reverted => Ok(ContractCallOutcome::Revert),
        }
    }
}

impl From<ContractCallResult> for v0::ContractCallResult {
    fn from(value: ContractCallResult) -> Self {
        Self {
            outcome: v0::ContractCallOutcome::from(value.outcome) as i32,
            metadata: Some(v0::ContractCallMetadata {
                metadata: ProstBytes::copy_from_slice(value.metadata.as_ref()),
            }),
            block_hash: Some(to_proto_block_hash(value.block_hash)),
            block_number: Some(to_proto_block_number(value.block_number)),
            tx_hash: Some(to_proto_settlement_tx_hash(value.tx_hash)),
        }
    }
}

impl TryFrom<v0::ContractCallResult> for ContractCallResult {
    type Error = Error;

    fn try_from(value: v0::ContractCallResult) -> Result<Self, Self::Error> {
        let outcome = v0::ContractCallOutcome::try_from(value.outcome)
            .map_err(|_| {
                Error::invalid_data(format!(
                    "unknown contract_call_result.outcome value {}",
                    value.outcome
                ))
            })?
            .try_into()
            .map_err(|error: Error| error.inside_field("outcome"))?;

        Ok(Self {
            outcome,
            metadata: required_field!(value, metadata => parse_contract_call_metadata),
            block_hash: required_field!(value, block_hash => parse_block_hash),
            block_number: required_field!(value, block_number => parse_block_number),
            tx_hash: required_field!(value, tx_hash => parse_settlement_tx_hash),
        })
    }
}

#[cfg(test)]
mod tests {
    use agglayer_types::{Digest, SettlementTxHash};
    use alloy::primitives::{BlockHash, Bytes};

    use super::*;

    #[test]
    fn contract_call_result_round_trip() {
        let result = ContractCallResult {
            outcome: ContractCallOutcome::Success,
            metadata: Bytes::from(vec![1, 2, 3]),
            block_hash: BlockHash::from([8_u8; 32]),
            block_number: 42,
            tx_hash: SettlementTxHash::new(Digest::from([9_u8; 32])),
        };

        let proto: v0::ContractCallResult = result.clone().into();
        let decoded = ContractCallResult::try_from(proto).unwrap();

        assert_eq!(decoded, result);
    }

    #[test]
    fn unspecified_outcome_fails() {
        let proto = v0::ContractCallResult {
            outcome: v0::ContractCallOutcome::Unspecified as i32,
            metadata: Some(v0::ContractCallMetadata {
                metadata: ProstBytes::copy_from_slice(&[1]),
            }),
            block_hash: Some(v0::BlockHash {
                hash: ProstBytes::copy_from_slice(&[2_u8; 32]),
            }),
            block_number: Some(v0::BlockNumber { number: 1 }),
            tx_hash: Some(v0::TxHash {
                hash: ProstBytes::copy_from_slice(&[3_u8; 32]),
            }),
        };

        assert!(ContractCallResult::try_from(proto).is_err());
    }
}
