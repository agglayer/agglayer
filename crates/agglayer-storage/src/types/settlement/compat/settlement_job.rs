use agglayer_types::SettlementJob;

use super::{
    primitives::{
        parse_address, parse_calldata, parse_eth_value, parse_uint128_to_u128, to_proto_address,
        to_proto_calldata, to_proto_eth_value, to_proto_uint128_from_u128,
    },
    Error,
};
use crate::types::generated::agglayer::storage::v0;

impl TryFrom<v0::SettlementJob> for SettlementJob {
    type Error = Error;

    fn try_from(value: v0::SettlementJob) -> Result<Self, Self::Error> {
        Ok(Self {
            contract_address: required_field!(value, contract_address => parse_address),
            calldata: required_field!(value, calldata => parse_calldata).into(),
            eth_value: required_field!(value, eth_value => parse_eth_value),
            gas_limit: required_field!(value, gas_limit => parse_uint128_to_u128),
            max_fee_per_gas_ceiling: required_field!(
                value,
                max_fee_per_gas_ceiling => parse_uint128_to_u128
            ),
            max_fee_per_gas_floor: required_field!(value, max_fee_per_gas_floor => parse_uint128_to_u128),
            max_fee_per_gas_increase_percents: value.max_fee_per_gas_increase_percents,
            max_priority_fee_per_gas_ceiling: required_field!(
                value,
                max_priority_fee_per_gas_ceiling => parse_uint128_to_u128
            ),
            max_priority_fee_per_gas_floor: required_field!(
                value,
                max_priority_fee_per_gas_floor => parse_uint128_to_u128
            ),
            max_priority_fee_per_gas_increase_percents: value
                .max_priority_fee_per_gas_increase_percents,
        })
    }
}

impl TryFrom<&SettlementJob> for v0::SettlementJob {
    type Error = Error;

    fn try_from(value: &SettlementJob) -> Result<Self, Self::Error> {
        Ok(Self {
            contract_address: Some(to_proto_address(value.contract_address)),
            calldata: Some(to_proto_calldata(value.calldata.as_ref())),
            eth_value: Some(to_proto_eth_value(value.eth_value)),
            gas_limit: Some(to_proto_uint128_from_u128(value.gas_limit)),
            max_fee_per_gas_ceiling: Some(to_proto_uint128_from_u128(
                value.max_fee_per_gas_ceiling,
            )),
            max_fee_per_gas_floor: Some(to_proto_uint128_from_u128(value.max_fee_per_gas_floor)),
            max_fee_per_gas_increase_percents: value.max_fee_per_gas_increase_percents,
            max_priority_fee_per_gas_ceiling: Some(to_proto_uint128_from_u128(
                value.max_priority_fee_per_gas_ceiling,
            )),
            max_priority_fee_per_gas_floor: Some(to_proto_uint128_from_u128(
                value.max_priority_fee_per_gas_floor,
            )),
            max_priority_fee_per_gas_increase_percents: value
                .max_priority_fee_per_gas_increase_percents,
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
    use agglayer_types::{Address, Digest};
    use alloy_primitives::Bytes;

    use super::*;

    fn sample_job() -> SettlementJob {
        SettlementJob {
            contract_address: Address::from([1_u8; 20]),
            calldata: Bytes::from(vec![1, 2, 3]),
            eth_value: agglayer_types::U256::from(3_u64),
            gas_limit: 10,
            max_fee_per_gas_ceiling: 20,
            max_fee_per_gas_floor: 30,
            max_fee_per_gas_increase_percents: 125,
            max_priority_fee_per_gas_ceiling: 40,
            max_priority_fee_per_gas_floor: 50,
            max_priority_fee_per_gas_increase_percents: 125,
        }
    }

    #[test]
    fn settlement_job_round_trip() {
        let job = sample_job();

        let proto = v0::SettlementJob::try_from(&job).unwrap();
        let decoded = SettlementJob::try_from(proto).unwrap();

        assert_eq!(decoded, job);
    }

    #[test]
    fn settlement_job_from_proto_rejects_missing_required_field() {
        let mut proto = v0::SettlementJob::try_from(sample_job()).unwrap();
        proto.contract_address = None;

        let result = SettlementJob::try_from(proto);

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
