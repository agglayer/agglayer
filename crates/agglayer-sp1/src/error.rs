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
