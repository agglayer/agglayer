use std::sync::Arc;

use agglayer_config::settlement_service::SettlementTransactionConfig;
use agglayer_storage::types::generated::agglayer::storage::v0;

use super::{
    primitives::{
        multiplier_from_percents, multiplier_to_percents, parse_address, parse_calldata,
        parse_eth_value, parse_uint128, to_proto_address, to_proto_calldata, to_proto_eth_value,
        to_proto_uint128,
    },
    Error,
};
use crate::settlement_task::SettlementJob;

impl SettlementJob {
    pub(crate) fn from_proto(
        value: v0::SettlementJob,
        num_confirmations: u32,
        settlement_config: Arc<SettlementTransactionConfig>,
    ) -> Result<Self, Error> {
        Ok(Self {
            contract_address: required_field!(value, contract_address => parse_address),
            calldata: required_field!(value, calldata => parse_calldata),
            eth_value: required_field!(value, eth_value => parse_eth_value),
            num_confirmations,
            gas_limit: required_field!(value, gas_limit => parse_uint128),
            max_fee_per_gas_ceiling: required_field!(
                value,
                max_fee_per_gas_ceiling => parse_uint128
            ),
            max_fee_per_gas_floor: required_field!(value, max_fee_per_gas_floor => parse_uint128),
            max_fee_per_gas_multiplier: multiplier_from_percents(
                value.max_fee_per_gas_increase_percents,
            ),
            max_priority_fee_per_gas_ceiling: required_field!(
                value,
                max_priority_fee_per_gas_ceiling => parse_uint128
            ),
            max_priority_fee_per_gas_floor: required_field!(
                value,
                max_priority_fee_per_gas_floor => parse_uint128
            ),
            max_priority_fee_per_gas_multiplier: multiplier_from_percents(
                value.max_priority_fee_per_gas_increase_percents,
            ),
            settlement_config,
        })
    }
}

impl TryFrom<&SettlementJob> for v0::SettlementJob {
    type Error = Error;

    fn try_from(value: &SettlementJob) -> Result<Self, Self::Error> {
        Ok(Self {
            contract_address: Some(to_proto_address(value.contract_address)),
            calldata: Some(to_proto_calldata(&value.calldata)),
            eth_value: Some(to_proto_eth_value(value.eth_value)),
            gas_limit: Some(to_proto_uint128(value.gas_limit)),
            max_fee_per_gas_ceiling: Some(to_proto_uint128(value.max_fee_per_gas_ceiling)),
            max_fee_per_gas_floor: Some(to_proto_uint128(value.max_fee_per_gas_floor)),
            max_fee_per_gas_increase_percents: multiplier_to_percents(
                value.max_fee_per_gas_multiplier,
            )?,
            max_priority_fee_per_gas_ceiling: Some(to_proto_uint128(
                value.max_priority_fee_per_gas_ceiling,
            )),
            max_priority_fee_per_gas_floor: Some(to_proto_uint128(
                value.max_priority_fee_per_gas_floor,
            )),
            max_priority_fee_per_gas_increase_percents: multiplier_to_percents(
                value.max_priority_fee_per_gas_multiplier,
            )?,
        })
    }
}

impl TryFrom<SettlementJob> for v0::SettlementJob {
    type Error = Error;

    fn try_from(value: SettlementJob) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

#[cfg(test)]
mod tests {
    use agglayer_config::Multiplier;
    use agglayer_types::Digest;
    use alloy::primitives::{Address, Bytes, U128, U256};

    use super::*;

    fn sample_job(multiplier: Multiplier) -> SettlementJob {
        SettlementJob {
            contract_address: Address::from([1_u8; 20]),
            calldata: Bytes::from(vec![1, 2, 3]),
            eth_value: U256::from(3_u64),
            num_confirmations: 9,
            gas_limit: U128::from(10_u64),
            max_fee_per_gas_ceiling: U128::from(20_u64),
            max_fee_per_gas_floor: U128::from(30_u64),
            max_fee_per_gas_multiplier: multiplier,
            max_priority_fee_per_gas_ceiling: U128::from(40_u64),
            max_priority_fee_per_gas_floor: U128::from(50_u64),
            max_priority_fee_per_gas_multiplier: multiplier,
            settlement_config: Arc::new(SettlementTransactionConfig::default()),
        }
    }

    #[test]
    fn settlement_job_round_trip() {
        let job = sample_job(Multiplier::from_u64_per_1000(1_250));

        let proto = v0::SettlementJob::try_from(&job).unwrap();
        let decoded =
            SettlementJob::from_proto(proto, job.num_confirmations, job.settlement_config.clone())
                .unwrap();

        assert_eq!(decoded, job);
    }

    #[test]
    fn settlement_job_to_proto_rejects_non_representable_multiplier() {
        let job = sample_job(Multiplier::from_u64_per_1000(1_001));

        assert!(v0::SettlementJob::try_from(&job).is_err());
    }

    #[test]
    fn settlement_job_from_proto_rejects_missing_required_field() {
        let job = sample_job(Multiplier::from_u64_per_1000(1_200));
        let mut proto = v0::SettlementJob::try_from(job).unwrap();
        proto.contract_address = None;

        let result =
            SettlementJob::from_proto(proto, 1, Arc::new(SettlementTransactionConfig::default()));

        assert!(result.is_err());
    }

    #[test]
    fn settlement_tx_hash_primitive_conversion_smoke() {
        let tx_hash = agglayer_types::SettlementTxHash::new(Digest::from([7_u8; 32]));
        let proto = super::super::primitives::to_proto_settlement_tx_hash(tx_hash);
        let decoded = super::super::primitives::parse_settlement_tx_hash(proto).unwrap();

        assert_eq!(decoded, tx_hash);
    }
}
