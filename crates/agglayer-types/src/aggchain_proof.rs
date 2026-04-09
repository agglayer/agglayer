pub use agglayer_interop_types::aggchain_proof::*;
use sp1_hypercube::{SP1PcsProofInner, SP1RecursionProof};
use sp1_primitives::SP1GlobalContext;
use sp1_sdk::{HashableKey as _, SP1VerifyingKey};
use sp1_sdk_v5::{HashableKey as _, SP1VerifyingKey as SP1VerifyingKeyV5};

use crate::bincode;

pub type CurrentSp1StarkProof = SP1RecursionProof<SP1GlobalContext, SP1PcsProofInner>;

pub struct ExecutableSp1Proof {
    pub proof: CurrentSp1StarkProof,
    pub vkey: SP1VerifyingKey,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Sp1ProofVersion {
    V4,
    V6,
}

#[derive(Debug, thiserror::Error)]
pub enum ProofError {
    #[error("invalid SP1 proof version `{version}`")]
    InvalidSp1Version { version: String },

    #[error("unsupported SP1 proof version `{version}` for read")]
    UnsupportedReadableSp1Version { version: String },

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

pub fn current_sp1_stark_with_context(
    proof: &CurrentSp1StarkProof,
    vkey: &SP1VerifyingKey,
    version: impl Into<String>,
) -> Result<SP1StarkWithContext, bincode::Error> {
    Ok(SP1StarkWithContext {
        version: version.into(),
        proof: bincode::sp1v4().serialize(proof)?,
        vkey: bincode::sp1v4().serialize(vkey)?,
    })
}

pub trait SP1StarkWithContextExt {
    fn version_kind(&self) -> Result<Sp1ProofVersion, ProofError>;
    fn ensure_readable(&self) -> Result<Sp1ProofVersion, ProofError>;
    fn ensure_executable(&self) -> Result<(), ProofError>;
    fn ensure_writable(&self) -> Result<(), ProofError>;
    fn executable(&self) -> Result<ExecutableSp1Proof, ProofError>;
    fn vkey_hash_bytes(&self) -> Result<[u8; 32], ProofError>;
    fn vkey_hash_u32(&self) -> Result<[u32; 8], ProofError>;
}

impl SP1StarkWithContextExt for SP1StarkWithContext {
    fn version_kind(&self) -> Result<Sp1ProofVersion, ProofError> {
        let major = self
            .version
            .trim_start_matches('v')
            .split('.')
            .next()
            .filter(|segment| !segment.is_empty())
            .ok_or_else(|| ProofError::InvalidSp1Version {
                version: self.version.clone(),
            })?;

        match major {
            "4" => Ok(Sp1ProofVersion::V4),
            "6" => Ok(Sp1ProofVersion::V6),
            _ if major.chars().all(|c| c.is_ascii_digit()) => {
                Err(ProofError::UnsupportedReadableSp1Version {
                    version: self.version.clone(),
                })
            }
            _ => Err(ProofError::InvalidSp1Version {
                version: self.version.clone(),
            }),
        }
    }

    fn ensure_readable(&self) -> Result<Sp1ProofVersion, ProofError> {
        self.version_kind()
    }

    fn ensure_executable(&self) -> Result<(), ProofError> {
        match self.ensure_readable()? {
            Sp1ProofVersion::V6 => Ok(()),
            Sp1ProofVersion::V4 => Err(ProofError::UnsupportedExecutableSp1Version {
                version: self.version.clone(),
            }),
        }
    }

    fn ensure_writable(&self) -> Result<(), ProofError> {
        match self.ensure_readable()? {
            Sp1ProofVersion::V6 => Ok(()),
            Sp1ProofVersion::V4 => Err(ProofError::UnsupportedWritableSp1Version {
                version: self.version.clone(),
            }),
        }
    }

    fn executable(&self) -> Result<ExecutableSp1Proof, ProofError> {
        self.ensure_executable()?;

        let proof = bincode::sp1v4()
            .deserialize(&self.proof)
            .map_err(|source| ProofError::DeserializeSp1Proof {
                version: self.version.clone(),
                source,
            })?;
        let vkey = bincode::sp1v4().deserialize(&self.vkey).map_err(|source| {
            ProofError::DeserializeSp1Vkey {
                version: self.version.clone(),
                source,
            }
        })?;

        Ok(ExecutableSp1Proof { proof, vkey })
    }

    fn vkey_hash_bytes(&self) -> Result<[u8; 32], ProofError> {
        Ok(match self.ensure_readable()? {
            Sp1ProofVersion::V4 => bincode::sp1v4()
                .deserialize::<SP1VerifyingKeyV5>(&self.vkey)
                .map_err(|source| ProofError::DeserializeSp1Vkey {
                    version: self.version.clone(),
                    source,
                })?
                .vk
                .hash_bytes(),
            Sp1ProofVersion::V6 => bincode::sp1v4()
                .deserialize::<SP1VerifyingKey>(&self.vkey)
                .map_err(|source| ProofError::DeserializeSp1Vkey {
                    version: self.version.clone(),
                    source,
                })?
                .vk
                .hash_bytes(),
        })
    }

    fn vkey_hash_u32(&self) -> Result<[u32; 8], ProofError> {
        Ok(match self.ensure_readable()? {
            Sp1ProofVersion::V4 => bincode::sp1v4()
                .deserialize::<SP1VerifyingKeyV5>(&self.vkey)
                .map_err(|source| ProofError::DeserializeSp1Vkey {
                    version: self.version.clone(),
                    source,
                })?
                .vk
                .hash_u32(),
            Sp1ProofVersion::V6 => bincode::sp1v4()
                .deserialize::<SP1VerifyingKey>(&self.vkey)
                .map_err(|source| ProofError::DeserializeSp1Vkey {
                    version: self.version.clone(),
                    source,
                })?
                .vk
                .hash_u32(),
        })
    }
}

pub trait ProofExt {
    fn ensure_writable(&self) -> Result<(), ProofError>;
    fn executable_sp1(&self) -> Result<ExecutableSp1Proof, ProofError>;
    fn vkey_hash_bytes(&self) -> Result<[u8; 32], ProofError>;
    fn vkey_hash_u32(&self) -> Result<[u32; 8], ProofError>;
    fn sp1(&self) -> &SP1StarkWithContext;
}

impl ProofExt for Proof {
    fn ensure_writable(&self) -> Result<(), ProofError> {
        match self {
            Self::SP1Stark(proof) => proof.ensure_writable(),
        }
    }

    fn executable_sp1(&self) -> Result<ExecutableSp1Proof, ProofError> {
        self.sp1().executable()
    }

    fn vkey_hash_bytes(&self) -> Result<[u8; 32], ProofError> {
        self.sp1().vkey_hash_bytes()
    }

    fn vkey_hash_u32(&self) -> Result<[u32; 8], ProofError> {
        self.sp1().vkey_hash_u32()
    }

    fn sp1(&self) -> &SP1StarkWithContext {
        match self {
            Self::SP1Stark(proof) => proof,
        }
    }
}
