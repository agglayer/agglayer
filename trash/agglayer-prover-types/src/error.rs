use prost::bytes::Bytes;
pub use prover_executor::Error;
use tonic::Status;

use crate::{
    bincode,
    v1::{ErrorKind, GenerateProofError},
};

pub struct ErrorWrapper;

impl ErrorWrapper {
    pub fn try_into_status(value: &Error) -> Result<Status, bincode::Error> {
        let (code, message, details) = match value {
            Error::UnableToExecuteProver => {
                let details = bincode::default().serialize(&GenerateProofError {
                    error: Bytes::new(),
                    error_type: ErrorKind::UnableToExecuteProver.into(),
                })?;

                (
                    tonic::Code::Internal,
                    "Unable to execute prover".to_string(),
                    details,
                )
            }
            Error::ProverFailed(_) => {
                let details = bincode::default().serialize(&GenerateProofError {
                    error: Bytes::new(),
                    error_type: ErrorKind::ProverFailed.into(),
                })?;
                (tonic::Code::Internal, value.to_string(), details)
            }
            Error::ProofVerificationFailed(ref proof_verification_error) => {
                let details = bincode::default().serialize(&GenerateProofError {
                    error: bincode::default()
                        .serialize(&proof_verification_error)?
                        .into(),
                    error_type: ErrorKind::ProofVerificationFailed.into(),
                })?;

                (tonic::Code::InvalidArgument, value.to_string(), details)
            }
            Error::ExecutorFailed(ref proof_error) => {
                let details = bincode::default().serialize(&GenerateProofError {
                    error: bincode::default().serialize(&proof_error)?.into(),
                    error_type: ErrorKind::ExecutorFailed.into(),
                })?;

                (tonic::Code::InvalidArgument, value.to_string(), details)
            }
            Error::UnableToInitializePrimaryProver => {
                let details = bincode::default().serialize(&GenerateProofError {
                    error: Bytes::new(),
                    error_type: ErrorKind::ExecutorFailed.into(),
                })?;

                (
                    tonic::Code::Internal,
                    "Executor failed to initialize the primary prover".to_string(),
                    details,
                )
            }
            Error::UnableToInitializeFallbackProver => {
                let details = bincode::default().serialize(&GenerateProofError {
                    error: Bytes::new(),
                    error_type: ErrorKind::ExecutorFailed.into(),
                })?;

                (
                    tonic::Code::Internal,
                    "Executor failed to initialize the fallback prover".to_string(),
                    details,
                )
            }
        };

        Ok(Status::with_details(code, message, details.into()))
    }
}
