use std::time::SystemTime;

use agglayer_config::Multiplier;
use agglayer_storage::{schema::CodecError, types::generated::agglayer::storage::v0};
use agglayer_types::SettlementTxHash;
use alloy::primitives::{Address, BlockHash, Bytes as AlloyBytes, U128, U256};
use prost::bytes::Bytes as ProstBytes;

use super::Error;

const PERCENTS_TO_MULTIPLIER_FACTOR: u64 = 10;

fn from_codec_error(error: CodecError) -> Error {
    Error::invalid_data(error.to_string())
}

fn bytes_to_fixed<const N: usize>(bytes: &[u8], field: &'static str) -> Result<[u8; N], Error> {
    bytes.try_into().map_err(|_| {
        Error::invalid_data(format!(
            "{field} must be {N} bytes long, got {}",
            bytes.len()
        ))
    })
}

pub(super) fn parse_address(value: v0::Address) -> Result<Address, Error> {
    let bytes = bytes_to_fixed::<20>(value.address.as_ref(), "address")?;
    Ok(Address::from(bytes))
}

pub(super) fn to_proto_address(value: Address) -> v0::Address {
    v0::Address {
        address: ProstBytes::copy_from_slice(value.as_slice()),
    }
}

pub(super) fn parse_calldata(value: v0::Calldata) -> Result<AlloyBytes, Error> {
    let bytes: Vec<u8> = value.into();
    Ok(AlloyBytes::from(bytes))
}

pub(super) fn to_proto_calldata(value: &AlloyBytes) -> v0::Calldata {
    value.to_vec().into()
}

pub(super) fn parse_uint128(value: v0::Uint128) -> Result<U128, Error> {
    parse_uint128_to_u128(value).map(U128::from)
}

pub(super) fn parse_uint128_to_u128(value: v0::Uint128) -> Result<u128, Error> {
    value.try_into().map_err(from_codec_error)
}

pub(super) fn to_proto_uint128(value: U128) -> v0::Uint128 {
    to_proto_uint128_from_u128(u128::from_be_bytes(value.to_be_bytes::<16>()))
}

pub(super) fn to_proto_uint128_from_u128(value: u128) -> v0::Uint128 {
    value.into()
}

pub(super) fn parse_uint256(value: v0::Uint256) -> Result<U256, Error> {
    value.try_into().map_err(from_codec_error)
}

pub(super) fn to_proto_uint256(value: U256) -> v0::Uint256 {
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

pub(super) fn parse_block_hash(value: v0::BlockHash) -> Result<BlockHash, Error> {
    value.try_into().map_err(from_codec_error)
}

pub(super) fn to_proto_block_hash(value: BlockHash) -> v0::BlockHash {
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

pub(super) fn multiplier_from_percents(value: u32) -> Multiplier {
    Multiplier::from_u64_per_1000(u64::from(value) * PERCENTS_TO_MULTIPLIER_FACTOR)
}

pub(super) fn multiplier_to_percents(value: Multiplier) -> Result<u32, Error> {
    let per_1000 = value.as_u64_per_1000();

    if !per_1000.is_multiple_of(PERCENTS_TO_MULTIPLIER_FACTOR) {
        return Err(Error::invalid_data(format!(
            "multiplier {value} cannot be represented in settlement proto percents"
        )));
    }

    let percents = per_1000 / PERCENTS_TO_MULTIPLIER_FACTOR;
    u32::try_from(percents).map_err(|_| {
        Error::invalid_data(format!(
            "multiplier {value} is too large for settlement proto percents"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_address_rejects_invalid_length() {
        let proto = v0::Address {
            address: ProstBytes::copy_from_slice(&[1_u8; 19]),
        };

        assert!(parse_address(proto).is_err());
    }

    #[test]
    fn multiplier_to_percents_rejects_precision_loss() {
        let multiplier = Multiplier::from_u64_per_1000(1_001);

        assert!(multiplier_to_percents(multiplier).is_err());
    }

    #[test]
    fn multiplier_round_trip() {
        let multiplier = Multiplier::from_u64_per_1000(1_250);

        let encoded = multiplier_to_percents(multiplier).unwrap();
        let decoded = multiplier_from_percents(encoded);

        assert_eq!(multiplier, decoded);
    }
}
