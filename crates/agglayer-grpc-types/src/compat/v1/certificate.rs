use agglayer_sp1::{AcceptancePolicy, ProofError, ProofExt};
use agglayer_types::{aggchain_proof::AggchainData, Certificate, Height, Metadata, NetworkId};
use thiserror::Error as ThisError;

use super::Error;
use crate::node::types::v1;

const AGGCHAIN_PROOF_FIELD: &str = "aggchain_proof";

#[derive(Debug, ThisError)]
pub enum CertificateConversionError {
    #[error(transparent)]
    Compat(#[from] Error),

    #[error("aggchain_proof: unsupported proof version `{version}`")]
    UnsupportedProofVersion { version: String },

    #[error("aggchain_proof: invalid proof version `{version}`")]
    InvalidProofVersion { version: String },
}

impl CertificateConversionError {
    #[must_use]
    pub fn unsupported_proof_version(&self) -> Option<&str> {
        match self {
            Self::UnsupportedProofVersion { version } => Some(version),
            Self::Compat(_) | Self::InvalidProofVersion { .. } => None,
        }
    }

    #[must_use]
    pub fn invalid_proof_version(&self) -> Option<&str> {
        match self {
            Self::InvalidProofVersion { version } => Some(version),
            Self::Compat(_) | Self::UnsupportedProofVersion { .. } => None,
        }
    }

    #[must_use]
    pub fn kind(&self) -> super::ErrorKind {
        match self {
            Self::Compat(error) => error.kind(),
            Self::UnsupportedProofVersion { .. } | Self::InvalidProofVersion { .. } => {
                super::ErrorKind::InvalidData
            }
        }
    }
}

impl From<CertificateConversionError> for Error {
    fn from(value: CertificateConversionError) -> Self {
        match value {
            CertificateConversionError::Compat(error) => error,
            CertificateConversionError::UnsupportedProofVersion { version } => {
                Error::invalid_data(format!("unsupported proof version `{version}`"))
                    .inside_field(AGGCHAIN_PROOF_FIELD)
            }
            CertificateConversionError::InvalidProofVersion { version } => {
                Error::invalid_data(format!("invalid proof version `{version}`"))
                    .inside_field(AGGCHAIN_PROOF_FIELD)
            }
        }
    }
}

fn proof_version_error(error: ProofError) -> CertificateConversionError {
    if let Some(version) = error.unsupported_version() {
        CertificateConversionError::UnsupportedProofVersion {
            version: version.to_owned(),
        }
    } else if let Some(version) = error.invalid_version() {
        CertificateConversionError::InvalidProofVersion {
            version: version.to_owned(),
        }
    } else {
        CertificateConversionError::Compat(
            Error::invalid_data(error.to_string()).inside_field(AGGCHAIN_PROOF_FIELD),
        )
    }
}

fn ensure_writable_proof_version(
    aggchain_data: &AggchainData,
) -> Result<(), CertificateConversionError> {
    let proof = match aggchain_data {
        AggchainData::Generic { proof, .. } => proof,
        AggchainData::MultisigAndAggchainProof { aggchain_proof, .. } => &aggchain_proof.proof,
        AggchainData::ECDSA { .. } | AggchainData::MultisigOnly { .. } => return Ok(()),
    };

    proof
        .ensure_writable(&AcceptancePolicy::DEFAULT)
        .map_err(proof_version_error)
}

impl TryFrom<v1::Certificate> for Certificate {
    type Error = CertificateConversionError;

    fn try_from(value: v1::Certificate) -> Result<Self, Self::Error> {
        let aggchain_data: AggchainData = required_field!(value, aggchain_data);

        // whether it involves a multisig
        let has_multisig = matches!(
            aggchain_data,
            AggchainData::Generic { .. } // 1-of-1
                | AggchainData::MultisigOnly { .. }
                | AggchainData::MultisigAndAggchainProof { .. }
        );

        // forbidden case
        if has_multisig && value.metadata.is_some() {
            return Err(Error::invalid_data("metadata provided with multisig".to_owned()).into());
        }

        ensure_writable_proof_version(&aggchain_data)?;

        let certificate = Certificate {
            network_id: NetworkId::new(value.network_id),
            height: Height::new(value.height),
            prev_local_exit_root: required_field!(value, prev_local_exit_root),
            new_local_exit_root: required_field!(value, new_local_exit_root),
            bridge_exits: value
                .bridge_exits
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()
                .map_err(|e: Error| {
                    CertificateConversionError::Compat(e.inside_field("bridge_exits"))
                })?,
            imported_bridge_exits: value
                .imported_bridge_exits
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()
                .map_err(|e: Error| {
                    CertificateConversionError::Compat(e.inside_field("imported_bridge_exits"))
                })?,
            aggchain_data,
            metadata: if let Some(metadata) = value.metadata {
                Metadata::new(
                    metadata
                        .try_into()
                        .map_err(CertificateConversionError::from)?,
                )
            } else {
                Metadata::default()
            },
            custom_chain_data: value.custom_chain_data.to_vec(),
            l1_info_tree_leaf_count: value.l1_info_tree_leaf_count,
        };

        Ok(certificate)
    }
}

impl TryFrom<Certificate> for v1::Certificate {
    type Error = Error;

    fn try_from(value: Certificate) -> Result<Self, Self::Error> {
        Ok(v1::Certificate {
            network_id: value.network_id.into(),
            height: value.height.as_u64(),
            prev_local_exit_root: Some(value.prev_local_exit_root.into()),
            new_local_exit_root: Some(value.new_local_exit_root.into()),
            bridge_exits: value.bridge_exits.into_iter().map(Into::into).collect(),
            imported_bridge_exits: value
                .imported_bridge_exits
                .into_iter()
                .map(Into::into)
                .collect(),
            aggchain_data: Some(value.aggchain_data.try_into()?),
            metadata: Some((*value.metadata).into()),
            custom_chain_data: value.custom_chain_data.into(),
            l1_info_tree_leaf_count: value.l1_info_tree_leaf_count,
        })
    }
}
