use agglayer_sp1::{version_kind, AcceptancePolicy, ProofError, ProofExt};
use agglayer_types::aggchain_proof::{Proof as TypedProof, SP1StarkWithContext};

use crate::types::generated::agglayer::storage::v0 as proto;

#[derive(Debug, thiserror::Error)]
pub enum ProofConversionError {
    #[error(transparent)]
    Sp1(#[from] ProofError),

    #[error("unsupported proof system `{proof_system}`")]
    UnsupportedProofSystem { proof_system: i32 },

    #[error("unsupported SP1 proof mode `{mode}`")]
    UnsupportedProofMode { mode: i32 },
}

impl TryFrom<&TypedProof> for proto::Proof {
    type Error = ProofConversionError;

    fn try_from(value: &TypedProof) -> Result<Self, Self::Error> {
        let sp1 = value.sp1();
        value.ensure_readable(&AcceptancePolicy::DEFAULT)?;

        Ok(Self {
            proof_system: proto::ProofSystem::Sp1 as i32,
            version: sp1.version.clone(),
            mode: proto::ProofMode::Compressed as i32,
            proof: sp1.proof.clone().into(),
            vkey: sp1.vkey.clone().into(),
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
        let proof_version = version_kind(&version).map_err(|err| match err {
            ProofError::UnsupportedSp1VersionMajor { version } => {
                ProofError::UnsupportedReadableSp1Version { version }
            }
            other => other,
        })?;
        AcceptancePolicy::DEFAULT.ensure_readable(proof_version, &version)?;

        Ok(TypedProof::SP1Stark(SP1StarkWithContext {
            proof: value.proof.to_vec(),
            vkey: value.vkey.to_vec(),
            version,
        }))
    }
}

#[cfg(test)]
mod tests {
    use agglayer_sp1::ProofError;
    use agglayer_types::{
        aggchain_proof::{Proof as TypedProof, SP1StarkWithContext},
        testutils::dummy_sp1_stark_proof_with_version,
    };

    use super::*;
    use crate::types::generated::agglayer::storage::v0::{ProofMode, ProofSystem};

    #[test]
    fn proof_proto_roundtrip_is_lossless_for_writable_versions() {
        let proof = dummy_sp1_stark_proof_with_version("v5.2.2");

        let proto = proto::Proof::try_from(&proof).unwrap();
        let decoded = TypedProof::try_from(proto).unwrap();

        assert_eq!(decoded, proof);
    }

    #[test]
    fn proof_proto_stores_sp1_bytes_directly() {
        let proof = dummy_sp1_stark_proof_with_version("v5.2.2");
        let TypedProof::SP1Stark(sp1) = &proof;

        let proto = proto::Proof::try_from(&proof).unwrap();

        assert_eq!(proto.proof.as_ref(), sp1.proof.as_slice());
        assert_eq!(proto.vkey.as_ref(), sp1.vkey.as_slice());
    }

    #[test]
    fn proof_proto_reads_direct_sp1_bytes() {
        let proof = dummy_sp1_stark_proof_with_version("v5.2.2");
        let TypedProof::SP1Stark(sp1) = &proof;

        let decoded = TypedProof::try_from(proto::Proof {
            proof_system: ProofSystem::Sp1 as i32,
            version: sp1.version.clone(),
            mode: ProofMode::Compressed as i32,
            proof: sp1.proof.clone().into(),
            vkey: sp1.vkey.clone().into(),
        })
        .unwrap();

        assert_eq!(decoded, proof);
    }

    #[test]
    fn proof_proto_reads_supported_read_only_versions() {
        let proof = dummy_sp1_stark_proof_with_version("v5.2.2");
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
    fn proof_proto_writes_current_versions() {
        let proof = dummy_sp1_stark_proof_with_version("v6.0.1");

        let proto = proto::Proof::try_from(&proof).unwrap();

        assert_eq!(proto.version, "v6.0.1");
    }

    #[test]
    fn proof_proto_rejects_unknown_storage_version() {
        let mut proof = dummy_sp1_stark_proof_with_version("v6.0.1");
        let TypedProof::SP1Stark(sp1) = &mut proof;
        sp1.version = "v7.0.0".to_owned();

        let err = proto::Proof::try_from(&proof).unwrap_err();

        assert!(matches!(
            err,
            ProofConversionError::Sp1(ProofError::UnsupportedReadableSp1Version { .. })
        ));
    }

    #[test]
    fn proof_proto_rejects_unknown_read_version() {
        let proof = dummy_sp1_stark_proof_with_version("v5.2.2");
        let mut proto = proto::Proof::try_from(&proof).unwrap();
        proto.version = "v7.0.0".to_owned();

        let err = TypedProof::try_from(proto).unwrap_err();

        assert!(matches!(
            err,
            ProofConversionError::Sp1(ProofError::UnsupportedReadableSp1Version { .. })
        ));
    }

    #[test]
    fn proof_proto_preserves_opaque_sp1_bytes() {
        let decoded = TypedProof::try_from(proto::Proof {
            proof_system: ProofSystem::Sp1 as i32,
            version: "v5.2.2".to_owned(),
            mode: ProofMode::Compressed as i32,
            proof: vec![0xde, 0xad, 0xbe, 0xef].into(),
            vkey: vec![0xca, 0xfe].into(),
        })
        .unwrap();

        assert_eq!(
            decoded,
            TypedProof::SP1Stark(SP1StarkWithContext {
                proof: vec![0xde, 0xad, 0xbe, 0xef],
                vkey: vec![0xca, 0xfe],
                version: "v5.2.2".to_owned(),
            })
        );
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
