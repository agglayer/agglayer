use agglayer_types::{Address, Digest, SettlementTxHash, B256, U256};
use prost::bytes::Bytes as ProstBytes;

use crate::{schema::CodecError, types::generated::agglayer::storage::v0};

fn invalid_data(message: impl Into<String>) -> CodecError {
    CodecError::InvalidEnumVariant(message.into())
}

fn bytes_to_fixed<const N: usize>(
    bytes: &[u8],
    field: &'static str,
) -> Result<[u8; N], CodecError> {
    bytes.try_into().map_err(|_| {
        invalid_data(format!(
            "{field} must be {N} bytes long, got {}",
            bytes.len()
        ))
    })
}

impl TryFrom<v0::Address> for Address {
    type Error = CodecError;

    fn try_from(value: v0::Address) -> Result<Self, Self::Error> {
        let bytes = bytes_to_fixed::<20>(value.address.as_ref(), "address")?;
        Ok(Address::from(bytes))
    }
}

impl From<Address> for v0::Address {
    fn from(value: Address) -> Self {
        Self {
            address: ProstBytes::copy_from_slice(value.as_slice()),
        }
    }
}

impl TryFrom<v0::Uint128> for u128 {
    type Error = CodecError;

    fn try_from(value: v0::Uint128) -> Result<Self, Self::Error> {
        let bytes = bytes_to_fixed::<16>(value.value.as_ref(), "uint128")?;
        Ok(u128::from_be_bytes(bytes))
    }
}

impl From<u128> for v0::Uint128 {
    fn from(value: u128) -> Self {
        Self {
            value: ProstBytes::copy_from_slice(&value.to_be_bytes()),
        }
    }
}

impl TryFrom<v0::Uint256> for U256 {
    type Error = CodecError;

    fn try_from(value: v0::Uint256) -> Result<Self, Self::Error> {
        let bytes = bytes_to_fixed::<32>(value.value.as_ref(), "uint256")?;
        Ok(U256::from_be_bytes(bytes))
    }
}

impl From<U256> for v0::Uint256 {
    fn from(value: U256) -> Self {
        Self {
            value: ProstBytes::copy_from_slice(&value.to_be_bytes::<32>()),
        }
    }
}

impl TryFrom<v0::EthValue> for U256 {
    type Error = CodecError;

    fn try_from(value: v0::EthValue) -> Result<Self, Self::Error> {
        let uint256 = value
            .value
            .ok_or_else(|| invalid_data("eth value is missing uint256 field"))?;
        uint256.try_into()
    }
}

impl From<U256> for v0::EthValue {
    fn from(value: U256) -> Self {
        Self {
            value: Some(value.into()),
        }
    }
}

impl TryFrom<v0::TxHash> for SettlementTxHash {
    type Error = CodecError;

    fn try_from(value: v0::TxHash) -> Result<Self, Self::Error> {
        let bytes = bytes_to_fixed::<32>(value.hash.as_ref(), "tx_hash")?;
        Ok(SettlementTxHash::new(Digest::from(bytes)))
    }
}

impl From<SettlementTxHash> for v0::TxHash {
    fn from(value: SettlementTxHash) -> Self {
        let hash: B256 = value.into();

        Self {
            hash: ProstBytes::copy_from_slice(hash.as_slice()),
        }
    }
}

impl TryFrom<v0::BlockHash> for B256 {
    type Error = CodecError;

    fn try_from(value: v0::BlockHash) -> Result<Self, Self::Error> {
        let bytes = bytes_to_fixed::<32>(value.hash.as_ref(), "block_hash")?;
        Ok(B256::from(bytes))
    }
}

impl From<B256> for v0::BlockHash {
    fn from(value: B256) -> Self {
        Self {
            hash: ProstBytes::copy_from_slice(value.as_slice()),
        }
    }
}

impl From<v0::Calldata> for Vec<u8> {
    fn from(value: v0::Calldata) -> Self {
        value.data.to_vec()
    }
}

impl From<Vec<u8>> for v0::Calldata {
    fn from(value: Vec<u8>) -> Self {
        Self {
            data: ProstBytes::from(value),
        }
    }
}

impl From<v0::BlockNumber> for u64 {
    fn from(value: v0::BlockNumber) -> Self {
        value.number
    }
}

impl From<u64> for v0::BlockNumber {
    fn from(value: u64) -> Self {
        Self { number: value }
    }
}

impl From<v0::Nonce> for u64 {
    fn from(value: v0::Nonce) -> Self {
        value.nonce
    }
}

impl From<u64> for v0::Nonce {
    fn from(value: u64) -> Self {
        Self { nonce: value }
    }
}

impl From<v0::AttemptSequenceNumber> for u64 {
    fn from(value: v0::AttemptSequenceNumber) -> Self {
        value.number
    }
}

impl From<u64> for v0::AttemptSequenceNumber {
    fn from(value: u64) -> Self {
        Self { number: value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_round_trip() {
        let address = Address::from([1_u8; 20]);

        let proto: v0::Address = address.into();
        let decoded = Address::try_from(proto).unwrap();

        assert_eq!(decoded, address);
    }

    #[test]
    fn uint128_invalid_len_fails() {
        let proto = v0::Uint128 {
            value: ProstBytes::copy_from_slice(&[1_u8; 15]),
        };

        assert!(u128::try_from(proto).is_err());
    }

    #[test]
    fn settlement_tx_hash_round_trip() {
        let tx_hash = SettlementTxHash::new(Digest::from([9_u8; 32]));

        let proto: v0::TxHash = tx_hash.into();
        let decoded = SettlementTxHash::try_from(proto).unwrap();

        assert_eq!(decoded, tx_hash);
    }

    #[test]
    fn eth_value_missing_inner_value_fails() {
        let proto = v0::EthValue { value: None };

        assert!(U256::try_from(proto).is_err());
    }

    #[test]
    fn block_hash_round_trip() {
        let block_hash = B256::from([7_u8; 32]);

        let proto: v0::BlockHash = block_hash.into();
        let decoded = B256::try_from(proto).unwrap();

        assert_eq!(decoded, block_hash);
    }
}
