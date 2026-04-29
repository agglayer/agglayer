use serde::{Deserialize, Serialize};
use sp1_sdk::SP1VerificationError;

#[derive(Clone, Debug, Serialize, Deserialize, thiserror::Error, PartialEq, Eq)]
pub enum ProofVerificationError {
    #[error("Version mismatch: {0}")]
    VersionMismatch(String),

    #[error("Core machine verification error: {0}")]
    Core(String),

    #[error("Recursion verification error: {0}")]
    Recursion(String),

    #[error("Plonk verification error: {0}")]
    Plonk(String),

    #[error("Groth16 verification error: {0}")]
    Groth16(String),

    #[error("Invalid public values")]
    InvalidPublicValues,

    #[error("Other error: {0}")]
    Other(String),
}

impl From<SP1VerificationError> for ProofVerificationError {
    fn from(err: SP1VerificationError) -> Self {
        match err {
            SP1VerificationError::VersionMismatch(version) => {
                ProofVerificationError::VersionMismatch(version)
            }
            SP1VerificationError::Core(core) => ProofVerificationError::Core(core.to_string()),
            SP1VerificationError::Recursion(recursion) => {
                ProofVerificationError::Recursion(recursion.to_string())
            }
            SP1VerificationError::Plonk(error) => ProofVerificationError::Plonk(error.to_string()),
            SP1VerificationError::Groth16(error) => {
                ProofVerificationError::Groth16(error.to_string())
            }
            SP1VerificationError::InvalidPublicValues => {
                ProofVerificationError::InvalidPublicValues
            }
            SP1VerificationError::UnexpectedExitCode(code) => {
                ProofVerificationError::Other(format!("Unexpected exit code: {code}"))
            }
            SP1VerificationError::Other(error) => ProofVerificationError::Other(error.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ProofVerificationError, SP1VerificationError};

    #[test]
    fn maps_unexpected_exit_code_to_other() {
        let err = ProofVerificationError::from(SP1VerificationError::UnexpectedExitCode(7));

        assert_eq!(
            err,
            ProofVerificationError::Other("Unexpected exit code: 7".to_owned())
        );
    }
}
