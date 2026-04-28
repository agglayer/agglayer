#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("missing required field `{field}`")]
    MissingField { field: &'static str },

    #[error("invalid data: {message}")]
    InvalidData { message: String },

    #[error("invalid `{field}`: {source}")]
    Field {
        field: &'static str,
        #[source]
        source: Box<Error>,
    },
}

impl From<crate::schema::CodecError> for Error {
    fn from(value: crate::schema::CodecError) -> Self {
        Self::invalid_data(value.to_string())
    }
}

impl From<prost_types::TimestampError> for Error {
    fn from(value: prost_types::TimestampError) -> Self {
        Self::invalid_data(format!("invalid timestamp: {value}"))
    }
}

impl Error {
    pub(crate) fn missing_field(field: &'static str) -> Self {
        Self::MissingField { field }
    }

    pub(crate) fn invalid_data(message: impl Into<String>) -> Self {
        Self::InvalidData {
            message: message.into(),
        }
    }

    pub(crate) fn inside_field(self, field: &'static str) -> Self {
        Self::Field {
            field,
            source: Box::new(self),
        }
    }
}
