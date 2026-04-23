use agglayer_sp1::{version_kind, AcceptancePolicy, ProofError, ProofExt};
use agglayer_types::aggchain_proof::{Proof as TypedProof, SP1StarkWithContext};

use crate::{schema::bincode_codec, types::generated::agglayer::storage::v0 as proto};

#[derive(Debug, thiserror::Error)]
pub enum ProofConversionError {
    #[error(transparent)]
    Sp1(#[from] ProofError),

    #[error("unsupported proof system `{proof_system}`")]
    UnsupportedProofSystem { proof_system: i32 },

    #[error("unsupported SP1 proof mode `{mode}`")]
    UnsupportedProofMode { mode: i32 },
    #[error("failed to serialize SP1 proof bytes for version `{version}`: {source}")]
    SerializeSp1Proof {
        version: String,
        #[source]
        source: agglayer_types::bincode::Error,
    },

    #[error("failed to serialize SP1 verifying key bytes for version `{version}`: {source}")]
    SerializeSp1Vkey {
        version: String,
        #[source]
        source: agglayer_types::bincode::Error,
    },
}

fn serialize_sp1_proof<T: serde::Serialize>(
    proof: &T,
    version: &str,
) -> Result<Vec<u8>, ProofConversionError> {
    bincode_codec()
        .serialize(proof)
        .map_err(|source| ProofConversionError::SerializeSp1Proof {
            version: version.to_owned(),
            source,
        })
}

fn serialize_sp1_vkey<T: serde::Serialize>(
    vkey: &T,
    version: &str,
) -> Result<Vec<u8>, ProofConversionError> {
    bincode_codec()
        .serialize(vkey)
        .map_err(|source| ProofConversionError::SerializeSp1Vkey {
            version: version.to_owned(),
            source,
        })
}

fn panic_bincode_error() -> agglayer_types::bincode::Error {
    Box::new(agglayer_types::bincode::ErrorKind::Custom(String::from(
        "panic during deserialization",
    )))
}

fn deserialize_sp1_proof<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    version: &str,
) -> Result<T, ProofConversionError> {
    std::panic::catch_unwind(|| bincode_codec().deserialize(bytes))
        .map_err(|_| ProofError::DeserializeSp1Proof {
            version: version.to_owned(),
            source: panic_bincode_error(),
        })?
        .map_err(|source| ProofError::DeserializeSp1Proof {
            version: version.to_owned(),
            source,
        })
        .map_err(Into::into)
}

fn deserialize_sp1_vkey<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    version: &str,
) -> Result<T, ProofConversionError> {
    std::panic::catch_unwind(|| bincode_codec().deserialize(bytes))
        .map_err(|_| ProofError::DeserializeSp1Vkey {
            version: version.to_owned(),
            source: panic_bincode_error(),
        })?
        .map_err(|source| ProofError::DeserializeSp1Vkey {
            version: version.to_owned(),
            source,
        })
        .map_err(Into::into)
}

impl TryFrom<&TypedProof> for proto::Proof {
    type Error = ProofConversionError;

    fn try_from(value: &TypedProof) -> Result<Self, Self::Error> {
        value.ensure_writable(&AcceptancePolicy::DEFAULT)?;

        let sp1 = value.sp1();

        Ok(Self {
            proof_system: proto::ProofSystem::Sp1 as i32,
            version: sp1.version.clone(),
            mode: proto::ProofMode::Compressed as i32,
            proof: serialize_sp1_proof(&sp1.proof, &sp1.version)?.into(),
            vkey: serialize_sp1_vkey(&sp1.vkey, &sp1.version)?.into(),
        })
    }
}

impl TryFrom<proto::Proof> for TypedProof {
    type Error = ProofConversionError;

    fn try_from(value: proto::Proof) -> Result<Self, Self::Error> {
        if value.proof_system != proto::ProofSystem::Sp1 as i32 {
            return Err(ProofConversionError::UnsupportedProofSystem {
                proof_system: value.proof_system,
            });
        }

        if value.mode != proto::ProofMode::Compressed as i32 {
            return Err(ProofConversionError::UnsupportedProofMode { mode: value.mode });
        }

        let version = value.version;
        let proof_version = version_kind(&version)?;
        AcceptancePolicy::DEFAULT.ensure_readable(proof_version, &version)?;

        Ok(TypedProof::SP1Stark(SP1StarkWithContext {
            proof: deserialize_sp1_proof(value.proof.as_ref(), &version)?,
            vkey: deserialize_sp1_vkey(value.vkey.as_ref(), &version)?,
            version,
        }))
    }
}

#[cfg(test)]
mod tests {
    use agglayer_sp1::ProofError;
    use agglayer_types::aggchain_proof::{Proof as TypedProof, SP1StarkWithContext};
    use sp1_sdk::Prover;

    use super::*;
    use crate::types::generated::agglayer::storage::v0::{ProofMode, ProofSystem};

    const EMPTY_ELF: &[u8] = include_bytes!("certificate/tests/empty.elf");

    fn mock_proof(version: &str) -> TypedProof {
        let client = sp1_sdk::ProverClient::builder().mock().build();
        let (proving_key, vkey) = client.setup(EMPTY_ELF);
        let proof = sp1_sdk::SP1ProofWithPublicValues::create_mock_proof(
            &proving_key,
            sp1_sdk::SP1PublicValues::new(),
            sp1_sdk::SP1ProofMode::Compressed,
            sp1_sdk::SP1_CIRCUIT_VERSION,
        )
        .proof
        .try_as_compressed()
        .unwrap();

        TypedProof::SP1Stark(SP1StarkWithContext {
            proof,
            vkey,
            version: version.to_owned(),
        })
    }

    #[test]
    fn proof_proto_roundtrip_is_lossless_for_writable_versions() {
        let proof = mock_proof("v5.2.2");

        let proto = proto::Proof::try_from(&proof).unwrap();
        let decoded = TypedProof::try_from(proto).unwrap();

        assert_eq!(decoded, proof);
    }

    #[test]
    fn proof_proto_reads_supported_read_only_versions() {
        let proof = mock_proof("v5.2.2");
        let TypedProof::SP1Stark(expected) = proof.clone();

        let mut proto = proto::Proof::try_from(&proof).unwrap();
        proto.version = "v6.0.1".to_owned();

        let decoded = TypedProof::try_from(proto).unwrap();

        assert_eq!(
            decoded,
            TypedProof::SP1Stark(SP1StarkWithContext {
                version: "v6.0.1".to_owned(),
                ..expected
            })
        );
    }

    #[test]
    fn proof_proto_rejects_writes_for_read_only_versions() {
        let proof = mock_proof("v6.0.1");

        let err = proto::Proof::try_from(&proof).unwrap_err();

        assert!(matches!(
            err,
            ProofConversionError::Sp1(ProofError::UnsupportedWritableSp1Version { .. })
        ));
    }

    #[test]
    fn proof_proto_rejects_unknown_write_version() {
        let err = proto::Proof::try_from(&mock_proof("v7.0.0")).unwrap_err();

        assert!(matches!(
            err,
            ProofConversionError::Sp1(ProofError::UnsupportedWritableSp1Version { .. })
        ));
    }

    #[test]
    fn proof_proto_rejects_unknown_read_version() {
        let proof = mock_proof("v5.2.2");
        let mut proto = proto::Proof::try_from(&proof).unwrap();
        proto.version = "v7.0.0".to_owned();

        let err = TypedProof::try_from(proto).unwrap_err();

        assert!(matches!(
            err,
            ProofConversionError::Sp1(ProofError::UnsupportedSp1VersionMajor { .. })
        ));
    }

    #[test]
    fn proof_proto_rejects_malformed_sp1_bytes() {
        let err = TypedProof::try_from(proto::Proof {
            proof_system: ProofSystem::Sp1 as i32,
            version: "v5.2.2".to_owned(),
            mode: ProofMode::Compressed as i32,
            proof: vec![0xde, 0xad, 0xbe, 0xef].into(),
            vkey: vec![0xca, 0xfe].into(),
        })
        .unwrap_err();

        assert!(matches!(
            err,
            ProofConversionError::Sp1(ProofError::DeserializeSp1Proof { .. })
        ));
    }

    #[test]
    fn proof_proto_rejects_unsupported_system() {
        let unsupported_system = TypedProof::try_from(proto::Proof {
            proof_system: ProofSystem::Unspecified as i32,
            version: "v5.2.2".to_owned(),
            mode: ProofMode::Compressed as i32,
            proof: vec![1, 2, 3].into(),
            vkey: vec![4, 5, 6].into(),
        })
        .unwrap_err();

        assert!(matches!(
            unsupported_system,
            ProofConversionError::UnsupportedProofSystem { .. }
        ));
    }
}
