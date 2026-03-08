use agglayer_storage::types::generated::agglayer::storage::v0;

use super::Error;
use crate::settlement_task::{ClientError, ClientErrorType};

impl From<ClientErrorType> for v0::ClientErrorType {
    fn from(value: ClientErrorType) -> Self {
        match value {
            ClientErrorType::Unknown => v0::ClientErrorType::Unspecified,
            ClientErrorType::NonceAlreadyUsed => v0::ClientErrorType::NonceAlreadyUsed,
        }
    }
}

impl From<v0::ClientErrorType> for ClientErrorType {
    fn from(value: v0::ClientErrorType) -> Self {
        match value {
            v0::ClientErrorType::Unspecified => ClientErrorType::Unknown,
            v0::ClientErrorType::NonceAlreadyUsed => ClientErrorType::NonceAlreadyUsed,
        }
    }
}

impl From<ClientError> for v0::ClientError {
    fn from(value: ClientError) -> Self {
        Self {
            error_type: v0::ClientErrorType::from(value.kind) as i32,
            error_message: value.message,
        }
    }
}

impl TryFrom<v0::ClientError> for ClientError {
    type Error = Error;

    fn try_from(value: v0::ClientError) -> Result<Self, Self::Error> {
        let error_type = v0::ClientErrorType::try_from(value.error_type).map_err(|_| {
            Error::invalid_data(format!(
                "unknown client_error.error_type value {}",
                value.error_type
            ))
        })?;

        Ok(Self {
            kind: error_type.into(),
            message: value.error_message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_error_round_trip() {
        let error = ClientError {
            kind: ClientErrorType::NonceAlreadyUsed,
            message: "nonce already used".to_string(),
        };

        let proto: v0::ClientError = error.clone().into();
        let decoded = ClientError::try_from(proto).unwrap();

        assert_eq!(decoded, error);
    }

    #[test]
    fn invalid_client_error_type_fails() {
        let proto = v0::ClientError {
            error_type: 999,
            error_message: "oops".to_string(),
        };

        assert!(ClientError::try_from(proto).is_err());
    }
}
