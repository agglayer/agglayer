use std::fmt;

use crate::CertificateStatusError;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum CertificateStatus {
    /// Received certificate from the network, nothing checked yet.
    ///
    /// Certificate will stay in this state until rate-limiting is lifted or an
    /// epoch-change event is triggered. A pending certificate can then be
    /// processed by the agglayer to be proven, or it could end up in error.
    Pending,

    /// Pessimistic proof has been generated for the certificate and stored in
    /// the rocksdb in the agglayer node.
    Proven,

    /// Settlement of the certificate's proof has already been started on L1
    /// (and acknowledged by its RPC) by issuing a contract call to the
    /// RollupManager, but the associated transaction has not yet seen
    /// enough confirmations.
    ///
    /// The certificate can move from Candidate to Settled if the associated
    /// transaction is accepted and the transaction receipt is a success. If the
    /// transaction receipt fails, the certificate will end up in Error.
    Candidate,

    /// Hit some error while moving the certificate through the pipeline.
    ///
    /// For example, proving failed (Pending -> InError), L1 reorg'd (Candidate
    /// -> InError)... See the documentation of `CertificateStatusError` for
    /// more details.
    ///
    /// Note that a certificate can be InError in agglayer but settled on L1,
    /// eg. if there was an error in agglayer but the certificate was valid
    /// and settled on L1.
    InError { error: Box<CertificateStatusError> },

    /// Transaction to settle the certificate was completed successfully on L1.
    Settled,
}

impl fmt::Display for CertificateStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CertificateStatus::Pending => write!(f, "Pending"),
            CertificateStatus::Proven => write!(f, "Proven"),
            CertificateStatus::Candidate => write!(f, "Candidate"),
            CertificateStatus::InError { error } => write!(f, "InError: {error}"),
            CertificateStatus::Settled => write!(f, "Settled"),
        }
    }
}

impl CertificateStatus {
    pub fn error(err: CertificateStatusError) -> Self {
        CertificateStatus::InError {
            error: Box::new(err),
        }
    }
}
