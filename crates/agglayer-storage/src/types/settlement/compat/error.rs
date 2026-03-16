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
