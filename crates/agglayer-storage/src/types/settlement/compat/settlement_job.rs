use agglayer_types::SettlementJob;

use super::Error;
use crate::types::generated::agglayer::storage::v0;

impl TryFrom<v0::SettlementJob> for SettlementJob {
    type Error = Error;

    fn try_from(value: v0::SettlementJob) -> Result<Self, Self::Error> {
        Ok(Self {
            contract_address: required_field!(value, contract_address =>
                try_into::<agglayer_types::Address>
            ),
            calldata: required_field!(value, calldata => into::<Vec<u8>>).into(),
            eth_value: required_field!(value, eth_value => try_into::<agglayer_types::U256>),
            gas_limit: required_field!(value, gas_limit => try_into::<u128>),
        })
    }
}

impl From<&SettlementJob> for v0::SettlementJob {
    fn from(value: &SettlementJob) -> Self {
        Self {
            contract_address: Some(value.contract_address.into()),
            calldata: Some(v0::Calldata {
                data: value.calldata.to_vec().into(),
            }),
            eth_value: Some(value.eth_value.into()),
            gas_limit: Some(value.gas_limit.into()),
        }
    }
}

impl From<SettlementJob> for v0::SettlementJob {
    fn from(value: SettlementJob) -> Self {
        (&value).into()
    }
}

#[cfg(test)]
mod tests {
    use agglayer_types::Address;
    use alloy_primitives::Bytes;

    use super::*;

    fn sample_job() -> SettlementJob {
        SettlementJob {
            contract_address: Address::from([1_u8; 20]),
            calldata: Bytes::from(vec![1, 2, 3]),
            eth_value: agglayer_types::U256::from(3_u64),
            gas_limit: 10,
        }
    }

    #[test]
    fn settlement_job_round_trip() {
        let job = sample_job();

        let proto: v0::SettlementJob = (&job).into();
        let decoded = SettlementJob::try_from(proto).unwrap();

        assert_eq!(decoded, job);
    }

    #[test]
    fn settlement_job_from_proto_rejects_missing_required_field() {
        let mut proto: v0::SettlementJob = sample_job().into();
        proto.contract_address = None;

        let result = SettlementJob::try_from(proto);

        assert!(result.is_err());
    }
}
