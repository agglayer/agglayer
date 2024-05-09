use ethers_gcp_kms_signer::CKMSError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KmsError {
    #[error("KMS Provider error: {0}")]
    KmsProvider(#[from] CKMSError),
    #[error("KMS configuration error: missing key or env {0}")]
    KmsConfig(String),
    #[error("KMS error: {0}")]
    Gcp(CKMSError),
}
