use agglayer_primitives::{Address, Digest, Signature};
use unified_bridge::CommitmentVersion;

use crate::{Error, SignerError};

mod common;
mod header;
mod height;
mod id;
mod index;
mod metadata;
#[cfg(feature = "testutils")]
mod testutils;
mod v0;
mod v1;

pub use common::Fields;
pub use header::{CertificateHeader, CertificateStatus, SettlementTxHash};
pub use height::Height;
pub use id::CertificateId;
pub use index::CertificateIndex;
pub use metadata::Metadata;
#[cfg(feature = "testutils")]
pub use testutils::compute_signature_info;
pub use v0::CertificateV0;
pub use v1::CertificateV1;

// Make V0 as the current certificate for now.
// Should be eventually replaced with CertificateVx.
pub type Certificate = CertificateV0;

/// Certificate of any supported version
#[derive(Clone, Debug, derive_more::From)]
pub enum CertificateVx {
    V0(CertificateV0),
    V1(CertificateV1),
}

impl CertificateVx {
    pub fn version(&self) -> Version {
        match self {
            Self::V0(_) => Version::V0,
            Self::V1(_) => Version::V1,
        }
    }

    pub fn hash(&self) -> CertificateId {
        match self {
            Self::V0(cert) => cert.hash(),
            Self::V1(cert) => cert.hash(),
        }
    }

    /// Returns the L1 Info Tree leaf count considered for this [`Certificate`].
    /// Corresponds to the highest L1 Info Tree leaf index considered by the
    /// imported bridge exits.
    pub fn l1_info_tree_leaf_count(&self) -> Option<u32> {
        todo!("split on version as above")
    }

    /// Returns the L1 Info Root considered for this [`Certificate`].
    /// Fails if multiple L1 Info Root are considered among the inclusion proofs
    /// of the imported bridge exits.
    pub fn l1_info_root(&self) -> Result<Option<Digest>, Error> {
        todo!("split on version as above")
    }

    /// Verify the extra certificate signature.
    pub fn verify_extra_signature(
        &self,
        expected_signer: Address,
        signature: Signature,
    ) -> Result<(), SignerError> {
        match self {
            Self::V0(cert) => cert.verify_extra_signature(expected_signer, signature),
            Self::V1(cert) => cert.verify_extra_signature(expected_signer, signature),
        }
    }

    /// Verify the signature on the PP commitment.
    pub fn verify_cert_signature(&self, expected_signer: Address) -> Result<(), SignerError> {
        match self {
            Self::V0(cert) => cert.verify_cert_signature(expected_signer),
            Self::V1(cert) => cert.verify_cert_signature(expected_signer),
        }
    }

    /// Retrieve the signer from the certificate signature.
    pub fn retrieve_signer(&self, version: CommitmentVersion) -> Result<Address, SignerError> {
        match self {
            Self::V0(cert) => cert.retrieve_signer(version),
            Self::V1(cert) => cert.retrieve_signer(version),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Version {
    V0,
    V1,
}

impl Version {
    pub fn as_u32(&self) -> u32 {
        match self {
            Self::V0 => 0,
            Self::V1 => 1,
        }
    }
}
