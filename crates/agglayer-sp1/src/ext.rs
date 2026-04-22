use agglayer_interop_types::aggchain_proof::{Proof, SP1StarkWithContext};
use sp1_sdk::HashableKey;

use crate::{
    error::ProofError,
    policy::AcceptancePolicy,
    version::{version_kind, Sp1ProofVersion},
};

/// Extension trait giving `Proof` direct access to the typed SP1 helpers
/// exposed by this crate.
///
/// The trait keeps the `sp1-sdk` dependency hidden from callers in
/// `agglayer-types` and `agglayer-grpc-types`, which must not depend on
/// `sp1-sdk` directly after #1508's storage decoupling work.
pub trait ProofExt {
    /// Borrow the inner SP1 stark context carried by this proof.
    fn sp1(&self) -> &SP1StarkWithContext;

    /// Classify the SP1 version string carried by the proof and confirm
    /// it is readable under `policy`.
    fn ensure_readable(&self, policy: &AcceptancePolicy) -> Result<Sp1ProofVersion, ProofError>;

    /// Classify the SP1 version string and confirm it is executable
    /// under `policy`.
    fn ensure_executable(&self, policy: &AcceptancePolicy) -> Result<(), ProofError>;

    /// Classify the SP1 version string and confirm it is writable under
    /// `policy` (i.e. the current node may emit new rows carrying this
    /// proof version).
    fn ensure_writable(&self, policy: &AcceptancePolicy) -> Result<(), ProofError>;

    /// 32-byte hash of the verifying key.
    fn vkey_hash_bytes(&self) -> Result<[u8; 32], ProofError>;

    /// 32-byte hash of the verifying key returned as 8 big-endian u32s.
    fn vkey_hash_u32(&self) -> Result<[u32; 8], ProofError>;
}

impl ProofExt for Proof {
    fn sp1(&self) -> &SP1StarkWithContext {
        match self {
            Proof::SP1Stark(inner) => inner,
        }
    }

    fn ensure_readable(&self, policy: &AcceptancePolicy) -> Result<Sp1ProofVersion, ProofError> {
        let sp1 = self.sp1();
        let v = version_kind(&sp1.version)?;
        policy.ensure_readable(v, &sp1.version)?;
        Ok(v)
    }

    fn ensure_executable(&self, policy: &AcceptancePolicy) -> Result<(), ProofError> {
        let sp1 = self.sp1();
        let v = version_kind(&sp1.version)?;
        policy.ensure_executable(v, &sp1.version)
    }

    fn ensure_writable(&self, policy: &AcceptancePolicy) -> Result<(), ProofError> {
        let sp1 = self.sp1();
        let v = version_kind(&sp1.version)?;
        policy.ensure_writable(v, &sp1.version)
    }

    fn vkey_hash_bytes(&self) -> Result<[u8; 32], ProofError> {
        Ok(self.sp1().vkey.hash_bytes())
    }

    fn vkey_hash_u32(&self) -> Result<[u32; 8], ProofError> {
        Ok(self.sp1().vkey.hash_u32())
    }
}
