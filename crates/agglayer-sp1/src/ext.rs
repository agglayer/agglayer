use agglayer_interop_types::aggchain_proof::{Proof, SP1StarkWithContext};
use sp1_sdk::HashableKey;

use crate::{
    error::ProofError,
    policy::AcceptancePolicy,
    version::{version_kind, Sp1ProofVersion},
};

pub trait Sp1StarkExt {
    fn version_kind(&self) -> Result<Sp1ProofVersion, ProofError>;
    fn ensure_readable(&self, policy: &AcceptancePolicy) -> Result<Sp1ProofVersion, ProofError>;
    fn ensure_executable(&self, policy: &AcceptancePolicy) -> Result<(), ProofError>;
    fn ensure_writable(&self, policy: &AcceptancePolicy) -> Result<(), ProofError>;
    fn vkey_hash_bytes(&self) -> Result<[u8; 32], ProofError>;
    fn vkey_hash_u32(&self) -> Result<[u32; 8], ProofError>;
}

impl Sp1StarkExt for SP1StarkWithContext {
    fn version_kind(&self) -> Result<Sp1ProofVersion, ProofError> {
        version_kind(&self.version)
    }

    fn ensure_readable(&self, policy: &AcceptancePolicy) -> Result<Sp1ProofVersion, ProofError> {
        let v = self.version_kind()?;
        policy.ensure_readable(v, &self.version)?;
        Ok(v)
    }

    fn ensure_executable(&self, policy: &AcceptancePolicy) -> Result<(), ProofError> {
        let v = self.version_kind()?;
        policy.ensure_executable(v, &self.version)
    }

    fn ensure_writable(&self, policy: &AcceptancePolicy) -> Result<(), ProofError> {
        let v = self.version_kind()?;
        policy.ensure_writable(v, &self.version)
    }

    fn vkey_hash_bytes(&self) -> Result<[u8; 32], ProofError> {
        Ok(self.vkey.hash_bytes())
    }

    fn vkey_hash_u32(&self) -> Result<[u32; 8], ProofError> {
        Ok(self.vkey.hash_u32())
    }
}

pub trait ProofExt {
    fn sp1(&self) -> &SP1StarkWithContext;
    fn ensure_readable(&self, policy: &AcceptancePolicy) -> Result<Sp1ProofVersion, ProofError>;
    fn ensure_writable(&self, policy: &AcceptancePolicy) -> Result<(), ProofError>;
    fn vkey_hash_bytes(&self) -> Result<[u8; 32], ProofError>;
    fn vkey_hash_u32(&self) -> Result<[u32; 8], ProofError>;
}

impl ProofExt for Proof {
    fn sp1(&self) -> &SP1StarkWithContext {
        match self {
            Proof::SP1Stark(inner) => inner,
        }
    }

    fn ensure_readable(&self, policy: &AcceptancePolicy) -> Result<Sp1ProofVersion, ProofError> {
        self.sp1().ensure_readable(policy)
    }

    fn ensure_writable(&self, policy: &AcceptancePolicy) -> Result<(), ProofError> {
        self.sp1().ensure_writable(policy)
    }

    fn vkey_hash_bytes(&self) -> Result<[u8; 32], ProofError> {
        self.sp1().vkey_hash_bytes()
    }

    fn vkey_hash_u32(&self) -> Result<[u32; 8], ProofError> {
        self.sp1().vkey_hash_u32()
    }
}
