use std::fmt;

use agglayer_types::primitives::SignatureError;
use tonic_types::FieldViolation;

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error(transparent)]
    Bincode(#[from] bincode::Error),

    #[error(transparent)]
    Signature(#[from] SignatureError),
}

#[derive(Clone, Copy, Debug)]
pub enum ErrorKind {
    InvalidData,
    MissingField,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::InvalidData => {
                write!(f, "Invalid data")
            }
            ErrorKind::MissingField => {
                write!(f, "Missing field")
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub struct Error {
    kind: ErrorKind,
    message: String,
    field: Vec<&'static str>,
    #[source]
    source: Option<SourceError>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.field.is_empty() {
            write!(f, "{}: ", self.field_str())?;
        }
        write!(f, "{}", self.message)
    }
}

impl Error {
    pub fn missing_field(f: &'static str) -> Self {
        Error {
            kind: ErrorKind::MissingField,
            message: "required field is missing".to_string(),
            field: vec![f],
            source: None,
        }
    }

    pub fn invalid_data(m: String) -> Self {
        Error {
            kind: ErrorKind::InvalidData,
            message: m,
            field: vec![],
            source: None,
        }
    }

    pub fn inside_field(mut self, f: &'static str) -> Self {
        self.field.push(f);
        self.field.rotate_right(1);
        self
    }

    pub fn serializing_proof(e: bincode::Error) -> Self {
        Error {
            kind: ErrorKind::InvalidData,
            message: "failed to serialize proof".to_string(),
            field: vec![],
            source: Some(SourceError::Bincode(e)),
        }
    }

    pub fn deserializing_proof(e: bincode::Error) -> Self {
        Error {
            kind: ErrorKind::InvalidData,
            message: "failed to deserialize proof".to_string(),
            field: vec![],
            source: Some(SourceError::Bincode(e)),
        }
    }

    pub fn parsing_signature(e: SignatureError) -> Self {
        Error {
            kind: ErrorKind::InvalidData,
            message: "failed to parse signature".to_string(),
            field: vec![],
            source: Some(SourceError::Signature(e)),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn field(&self) -> &[&'static str] {
        &self.field
    }

    pub fn field_str(&self) -> String {
        if self.field.is_empty() {
            ".".to_string()
        } else {
            self.field.join(".")
        }
    }
}

impl From<&Error> for Vec<FieldViolation> {
    fn from(value: &Error) -> Self {
        vec![FieldViolation::new(
            value.field_str(),
            value.message.clone(),
        )]
    }
}
