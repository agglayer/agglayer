use std::time::SystemTime;

use agglayer_types::{Address, SettlementTxHash, B256, U256};

use super::Error;
use crate::{schema::CodecError, types::generated::agglayer::storage::v0};

fn from_codec_error(error: CodecError) -> Error {
    Error::invalid_data(error.to_string())
}

pub(super) fn parse_address(value: v0::Address) -> Result<Address, Error> {
    value.try_into().map_err(from_codec_error)
}

pub(super) fn to_proto_address(value: Address) -> v0::Address {
    value.into()
}

pub(super) fn parse_calldata(value: v0::Calldata) -> Result<Vec<u8>, Error> {
    Ok(value.into())
}

pub(super) fn to_proto_calldata(value: &[u8]) -> v0::Calldata {
    value.to_vec().into()
}

pub(super) fn parse_uint128_to_u128(value: v0::Uint128) -> Result<u128, Error> {
    value.try_into().map_err(from_codec_error)
}

pub(super) fn to_proto_uint128_from_u128(value: u128) -> v0::Uint128 {
    value.into()
}

pub(super) fn parse_eth_value(value: v0::EthValue) -> Result<U256, Error> {
    value.try_into().map_err(from_codec_error)
}

pub(super) fn to_proto_eth_value(value: U256) -> v0::EthValue {
    value.into()
}

pub(super) fn parse_settlement_tx_hash(value: v0::TxHash) -> Result<SettlementTxHash, Error> {
    value.try_into().map_err(from_codec_error)
}

pub(super) fn to_proto_settlement_tx_hash(value: SettlementTxHash) -> v0::TxHash {
    value.into()
}

pub(super) fn parse_block_hash(value: v0::BlockHash) -> Result<B256, Error> {
    value.try_into().map_err(from_codec_error)
}

pub(super) fn to_proto_block_hash(value: B256) -> v0::BlockHash {
    value.into()
}

pub(super) fn parse_block_number(value: v0::BlockNumber) -> Result<u64, Error> {
    Ok(value.into())
}

pub(super) fn to_proto_block_number(value: u64) -> v0::BlockNumber {
    value.into()
}

pub(super) fn parse_timestamp(value: prost_types::Timestamp) -> Result<SystemTime, Error> {
    value
        .try_into()
        .map_err(|error| Error::invalid_data(format!("invalid timestamp: {error}")))
}

pub(super) fn to_proto_timestamp(value: SystemTime) -> prost_types::Timestamp {
    prost_types::Timestamp::from(value)
}

#[cfg(test)]
mod tests {
    use prost::bytes::Bytes as ProstBytes;

    use super::*;

    #[test]
    fn parse_address_rejects_invalid_length() {
        let proto = v0::Address {
            address: ProstBytes::copy_from_slice(&[1_u8; 19]),
        };

        assert!(parse_address(proto).is_err());
    }

    #[test]
    fn settlement_tx_hash_round_trip() {
        let tx_hash = SettlementTxHash::new(agglayer_types::Digest::from([7_u8; 32]));

        let proto = to_proto_settlement_tx_hash(tx_hash);
        let decoded = parse_settlement_tx_hash(proto).unwrap();

        assert_eq!(decoded, tx_hash);
    }
}
