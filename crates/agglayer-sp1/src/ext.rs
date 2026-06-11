use agglayer_interop_types::aggchain_proof::{Proof, SP1StarkWithContext};
use serde::{de::DeserializeOwned, Serialize};
use sp1_sdk::{HashableKey, SP1VerifyingKey};

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
/// `sp1-sdk` directly after the storage decoupling work.
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

    /// 32-byte hash of the verifying key in both byte and u32 forms.
    fn vkey_hashes(&self) -> Result<([u8; 32], [u32; 8]), ProofError>;

    /// Deserialize the v6 SP1 verifying key carried by this proof.
    fn verifying_key(&self) -> Result<SP1VerifyingKey, ProofError>;

    /// Deserialize the v6 executable SP1 proof and verifying key.
    fn executable_sp1(
        &self,
        policy: &AcceptancePolicy,
    ) -> Result<V6Sp1StarkWithContext, ProofError>;
}

pub type V6Sp1StarkProof = sp1_core_executor::SP1RecursionProof<
    sp1_primitives::SP1GlobalContext,
    sp1_hypercube::SP1PcsProofInner,
>;

#[derive(Clone)]
pub struct V6Sp1StarkWithContext {
    pub proof: V6Sp1StarkProof,
    pub vkey: SP1VerifyingKey,
}

impl std::fmt::Debug for V6Sp1StarkWithContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("V6Sp1StarkWithContext")
            .field("proof", &self.proof)
            .field("vkey", &"<SP1VerifyingKey>")
            .finish()
    }
}

fn panic_bincode_error(
    payload: Box<dyn std::any::Any + Send>,
) -> agglayer_interop_types::bincode::Error {
    let message = if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "panic while deserializing SP1 bytes".to_owned()
    };

    agglayer_interop_types::bincode::ErrorKind::Custom(message).into()
}

fn deserialize_sp1_bytes<T, F>(bytes: &[u8], map_err: F) -> Result<T, ProofError>
where
    T: DeserializeOwned,
    F: Fn(agglayer_interop_types::bincode::Error) -> ProofError,
{
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        agglayer_interop_types::bincode::default().deserialize(bytes)
    })) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(source)) => Err(map_err(source)),
        Err(payload) => Err(map_err(panic_bincode_error(payload))),
    }
}

fn ensure_v6_readable(sp1: &SP1StarkWithContext) -> Result<(), ProofError> {
    let v = version_kind(&sp1.version).map_err(|err| match err {
        ProofError::UnsupportedSp1VersionMajor { version } => {
            ProofError::UnsupportedReadableSp1Version { version }
        }
        other => other,
    })?;

    match v {
        Sp1ProofVersion::V6 => Ok(()),
        Sp1ProofVersion::V5 => Err(ProofError::UnsupportedReadableSp1Version {
            version: sp1.version.clone(),
        }),
    }
}

fn ensure_v6_writable(version: &str) -> Result<(), ProofError> {
    let v = version_kind(version).map_err(|err| match err {
        ProofError::UnsupportedSp1VersionMajor { version } => {
            ProofError::UnsupportedWritableSp1Version { version }
        }
        other => other,
    })?;

    match v {
        Sp1ProofVersion::V6 => Ok(()),
        Sp1ProofVersion::V5 => Err(ProofError::UnsupportedWritableSp1Version {
            version: version.to_owned(),
        }),
    }
}

fn deserialize_v6_sp1_proof(sp1: &SP1StarkWithContext) -> Result<V6Sp1StarkProof, ProofError> {
    ensure_v6_readable(sp1)?;

    deserialize_sp1_bytes(&sp1.proof, |source| ProofError::DeserializeSp1Proof {
        version: sp1.version.clone(),
        source,
    })
}

fn deserialize_legacy_v5_vkey(
    sp1: &SP1StarkWithContext,
) -> Result<sp1_sdk_v5::SP1VerifyingKey, ProofError> {
    match version_kind(&sp1.version)? {
        Sp1ProofVersion::V5 => {}
        Sp1ProofVersion::V6 => {
            return Err(ProofError::UnsupportedReadableSp1Version {
                version: sp1.version.clone(),
            })
        }
    }

    deserialize_sp1_bytes(&sp1.vkey, |source| ProofError::DeserializeSp1Vkey {
        version: sp1.version.clone(),
        source,
    })
}

fn deserialize_v6_sp1_vkey(sp1: &SP1StarkWithContext) -> Result<SP1VerifyingKey, ProofError> {
    ensure_v6_readable(sp1)?;

    deserialize_sp1_bytes(&sp1.vkey, |source| ProofError::DeserializeSp1Vkey {
        version: sp1.version.clone(),
        source,
    })
}

pub fn v6_sp1_stark_with_context<P: Serialize>(
    proof: &P,
    vkey: &SP1VerifyingKey,
    version: &str,
) -> Result<SP1StarkWithContext, ProofError> {
    ensure_v6_writable(version)?;

    Ok(SP1StarkWithContext {
        proof: agglayer_interop_types::bincode::default()
            .serialize(proof)
            .map_err(|source| ProofError::SerializeSp1Proof {
                version: version.to_owned(),
                source,
            })?,
        vkey: agglayer_interop_types::bincode::default()
            .serialize(vkey)
            .map_err(|source| ProofError::SerializeSp1Vkey {
                version: version.to_owned(),
                source,
            })?,
        version: version.to_owned(),
    })
}

impl ProofExt for Proof {
    fn sp1(&self) -> &SP1StarkWithContext {
        match self {
            Proof::SP1Stark(inner) => inner,
        }
    }

    fn ensure_readable(&self, policy: &AcceptancePolicy) -> Result<Sp1ProofVersion, ProofError> {
        let sp1 = self.sp1();
        let v = version_kind(&sp1.version).map_err(|err| match err {
            ProofError::UnsupportedSp1VersionMajor { version } => {
                ProofError::UnsupportedReadableSp1Version { version }
            }
            other => other,
        })?;
        policy.ensure_readable(v, &sp1.version)?;
        Ok(v)
    }

    fn ensure_executable(&self, policy: &AcceptancePolicy) -> Result<(), ProofError> {
        let sp1 = self.sp1();
        let v = version_kind(&sp1.version).map_err(|err| match err {
            ProofError::UnsupportedSp1VersionMajor { version } => {
                ProofError::UnsupportedExecutableSp1Version { version }
            }
            other => other,
        })?;
        policy.ensure_executable(v, &sp1.version)
    }

    fn ensure_writable(&self, policy: &AcceptancePolicy) -> Result<(), ProofError> {
        let sp1 = self.sp1();
        let v = version_kind(&sp1.version).map_err(|err| match err {
            ProofError::UnsupportedSp1VersionMajor { version } => {
                ProofError::UnsupportedWritableSp1Version { version }
            }
            other => other,
        })?;
        policy.ensure_writable(v, &sp1.version)
    }

    fn vkey_hash_bytes(&self) -> Result<[u8; 32], ProofError> {
        self.vkey_hashes().map(|(bytes, _)| bytes)
    }

    fn vkey_hash_u32(&self) -> Result<[u32; 8], ProofError> {
        self.vkey_hashes().map(|(_, words)| words)
    }

    fn vkey_hashes(&self) -> Result<([u8; 32], [u32; 8]), ProofError> {
        let sp1 = self.sp1();
        match version_kind(&sp1.version)? {
            Sp1ProofVersion::V5 => {
                use sp1_sdk_v5::HashableKey as _;

                let vkey = deserialize_legacy_v5_vkey(sp1)?;
                Ok((vkey.hash_bytes(), vkey.hash_u32()))
            }
            Sp1ProofVersion::V6 => {
                let vkey = self.verifying_key()?;
                Ok((vkey.hash_bytes(), vkey.hash_u32()))
            }
        }
    }

    fn verifying_key(&self) -> Result<SP1VerifyingKey, ProofError> {
        deserialize_v6_sp1_vkey(self.sp1())
    }

    fn executable_sp1(
        &self,
        policy: &AcceptancePolicy,
    ) -> Result<V6Sp1StarkWithContext, ProofError> {
        self.ensure_executable(policy)?;

        let sp1 = self.sp1();
        Ok(V6Sp1StarkWithContext {
            proof: deserialize_v6_sp1_proof(sp1)?,
            vkey: self.verifying_key()?,
        })
    }
}
