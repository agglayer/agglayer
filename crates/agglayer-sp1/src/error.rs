use agglayer_interop_types::bincode;

#[derive(Debug, thiserror::Error)]
pub enum ProofError {
    #[error("invalid SP1 proof version `{version}`")]
    InvalidSp1Version { version: String },

    #[error("unsupported SP1 proof version `{version}` for read")]
    UnsupportedReadableSp1Version { version: String },

    #[error("unsupported SP1 proof version `{version}`")]
    UnsupportedSp1VersionMajor { version: String },

    #[error("unsupported SP1 proof version `{version}` for execute")]
    UnsupportedExecutableSp1Version { version: String },

    #[error("unsupported SP1 proof version `{version}` for write")]
    UnsupportedWritableSp1Version { version: String },

    #[error("failed to deserialize SP1 proof bytes for version `{version}`: {source}")]
    DeserializeSp1Proof {
        version: String,
        #[source]
        source: bincode::Error,
    },

    #[error("failed to deserialize SP1 verifying key bytes for version `{version}`: {source}")]
    DeserializeSp1Vkey {
        version: String,
        #[source]
        source: bincode::Error,
    },
}

impl ProofError {
    #[must_use]
    pub fn invalid_version(&self) -> Option<&str> {
        match self {
            Self::InvalidSp1Version { version } => Some(version),
            Self::UnsupportedReadableSp1Version { .. }
            | Self::UnsupportedSp1VersionMajor { .. }
            | Self::UnsupportedExecutableSp1Version { .. }
            | Self::UnsupportedWritableSp1Version { .. }
            | Self::DeserializeSp1Proof { .. }
            | Self::DeserializeSp1Vkey { .. } => None,
        }
    }

    #[must_use]
    pub fn unsupported_version(&self) -> Option<&str> {
        match self {
            Self::UnsupportedReadableSp1Version { version }
            | Self::UnsupportedSp1VersionMajor { version }
            | Self::UnsupportedExecutableSp1Version { version }
            | Self::UnsupportedWritableSp1Version { version }
            | Self::DeserializeSp1Proof { version, .. }
            | Self::DeserializeSp1Vkey { version, .. } => Some(version),
            Self::InvalidSp1Version { .. } => None,
        }
    }
}
