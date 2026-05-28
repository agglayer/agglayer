use agglayer_sp1::{version_kind, ProofError, ProofExt};
use agglayer_tries::roots::LocalExitRoot;
use agglayer_types::{
    aggchain_proof::{Proof as AggchainProof, SP1StarkWithContext},
    bincode,
    primitives::Digest,
    Proof as StoredProof,
};
use pessimistic_proof::PessimisticProofOutput;

use crate::{schema::CodecError, types::generated::agglayer::storage::v0 as proto};

#[derive(Debug, thiserror::Error)]
pub enum ProofConversionError {
    #[error(transparent)]
    Sp1(#[from] ProofError),

    #[error("missing field `{0}`")]
    MissingField(&'static str),

    #[error("invalid data for `{field}`: {reason}")]
    InvalidData {
        field: &'static str,
        reason: String,
    },

    #[error("unsupported proof system `{proof_system}`")]
    UnsupportedProofSystem { proof_system: i32 },

    #[error("unsupported SP1 proof mode `{mode}`")]
    UnsupportedProofMode { mode: i32 },
}

#[derive(Debug, Clone)]
pub struct LegacyProof(pub StoredProof);

impl From<LegacyProof> for StoredProof {
    fn from(LegacyProof(proof): LegacyProof) -> Self {
        proof
    }
}

fn expect_bytes<const N: usize>(
    bytes: &[u8],
    field: &'static str,
) -> Result<[u8; N], ProofConversionError> {
    bytes.try_into().map_err(|_| ProofConversionError::InvalidData {
        field,
        reason: format!("expected {N} bytes, got {}", bytes.len()),
    })
}

fn stored_proof_mode(proof: &StoredProof) -> i32 {
    match proof {
        StoredProof::SP1(sp1) => match agglayer_sp1::ProofMode::from(&sp1.proof) {
            agglayer_sp1::ProofMode::Core => proto::ProofMode::Unspecified as i32,
            agglayer_sp1::ProofMode::Compressed => proto::ProofMode::Compressed as i32,
            agglayer_sp1::ProofMode::Plonk => proto::ProofMode::Plonk as i32,
            agglayer_sp1::ProofMode::Groth16 => proto::ProofMode::Groth16 as i32,
        },
    }
}

impl TryFrom<&AggchainProof> for proto::Proof {
    type Error = ProofConversionError;

    fn try_from(value: &AggchainProof) -> Result<Self, Self::Error> {
        let sp1 = value.sp1();
        version_kind(&sp1.version)?;

        Ok(Self {
            proof_system: proto::ProofSystem::Sp1 as i32,
            version: sp1.version.clone(),
            mode: proto::ProofMode::Compressed as i32,
            proof: sp1.proof.clone().into(),
            vkey: sp1.vkey.clone().into(),
        })
    }
}

impl TryFrom<proto::Proof> for AggchainProof {
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
        version_kind(&version)?;

        Ok(AggchainProof::SP1Stark(SP1StarkWithContext {
            proof: value.proof.to_vec(),
            vkey: value.vkey.to_vec(),
            version,
        }))
    }
}

impl From<PessimisticProofOutput> for proto::PessimisticProofOutput {
    fn from(value: PessimisticProofOutput) -> Self {
        Self {
            prev_local_exit_root: Some(proto::LocalExitRoot {
                value: Digest::from(value.prev_local_exit_root).0.to_vec().into(),
            }),
            prev_pessimistic_root: Some(proto::PessimisticRoot {
                value: value.prev_pessimistic_root.0.to_vec().into(),
            }),
            l1_info_root: Some(proto::L1InfoRoot {
                value: value.l1_info_root.0.to_vec().into(),
            }),
            origin_network: value.origin_network.to_u32(),
            aggchain_hash: Some(proto::AggchainHash {
                value: value.aggchain_hash.0.to_vec().into(),
            }),
            new_local_exit_root: Some(proto::LocalExitRoot {
                value: Digest::from(value.new_local_exit_root).0.to_vec().into(),
            }),
            new_pessimistic_root: Some(proto::PessimisticRoot {
                value: value.new_pessimistic_root.0.to_vec().into(),
            }),
        }
    }
}

impl TryFrom<proto::PessimisticProofOutput> for PessimisticProofOutput {
    type Error = ProofConversionError;

    fn try_from(value: proto::PessimisticProofOutput) -> Result<Self, Self::Error> {
        let prev_local_exit_root = value
            .prev_local_exit_root
            .ok_or(ProofConversionError::MissingField("prev_local_exit_root"))?;
        let prev_pessimistic_root = value
            .prev_pessimistic_root
            .ok_or(ProofConversionError::MissingField("prev_pessimistic_root"))?;
        let l1_info_root = value
            .l1_info_root
            .ok_or(ProofConversionError::MissingField("l1_info_root"))?;
        let aggchain_hash = value
            .aggchain_hash
            .ok_or(ProofConversionError::MissingField("aggchain_hash"))?;
        let new_local_exit_root = value
            .new_local_exit_root
            .ok_or(ProofConversionError::MissingField("new_local_exit_root"))?;
        let new_pessimistic_root = value
            .new_pessimistic_root
            .ok_or(ProofConversionError::MissingField("new_pessimistic_root"))?;

        Ok(Self {
            prev_local_exit_root: LocalExitRoot::from(Digest::from(expect_bytes::<32>(
                prev_local_exit_root.value.as_ref(),
                "prev_local_exit_root",
            )?)),
            prev_pessimistic_root: Digest::from(expect_bytes::<32>(
                prev_pessimistic_root.value.as_ref(),
                "prev_pessimistic_root",
            )?),
            l1_info_root: Digest::from(expect_bytes::<32>(
                l1_info_root.value.as_ref(),
                "l1_info_root",
            )?),
            origin_network: value.origin_network.into(),
            aggchain_hash: Digest::from(expect_bytes::<32>(
                aggchain_hash.value.as_ref(),
                "aggchain_hash",
            )?),
            new_local_exit_root: LocalExitRoot::from(Digest::from(expect_bytes::<32>(
                new_local_exit_root.value.as_ref(),
                "new_local_exit_root",
            )?)),
            new_pessimistic_root: Digest::from(expect_bytes::<32>(
                new_pessimistic_root.value.as_ref(),
                "new_pessimistic_root",
            )?),
        })
    }
}

impl TryFrom<&StoredProof> for proto::PessimisticStoredProof {
    type Error = ProofConversionError;

    fn try_from(value: &StoredProof) -> Result<Self, Self::Error> {
        match value {
            StoredProof::SP1(sp1) => {
                let public_values = if sp1.public_values.as_slice().is_empty() {
                    None
                } else {
                    Some(proto::PessimisticProofOutput::from(
                        PessimisticProofOutput::bincode_codec()
                            .deserialize::<PessimisticProofOutput>(sp1.public_values.as_slice())
                            .map_err(|error| ProofConversionError::InvalidData {
                                field: "public_values",
                                reason: error.to_string(),
                            })?,
                    ))
                };

                Ok(Self {
                    proof: Some(proto::Proof {
                        proof_system: proto::ProofSystem::Sp1 as i32,
                        version: sp1.sp1_version.clone(),
                        mode: stored_proof_mode(value),
                        proof: bincode::default()
                            .serialize(&sp1.proof)
                            .map_err(|error| ProofConversionError::InvalidData {
                                field: "proof",
                                reason: error.to_string(),
                            })?
                            .into(),
                        vkey: bincode::default()
                            .serialize(&sp1.tee_proof)
                            .map_err(|error| ProofConversionError::InvalidData {
                                field: "tee_proof",
                                reason: error.to_string(),
                            })?
                            .into(),
                    }),
                    public_values,
                })
            }
        }
    }
}

impl TryFrom<proto::PessimisticStoredProof> for StoredProof {
    type Error = ProofConversionError;

    fn try_from(value: proto::PessimisticStoredProof) -> Result<Self, Self::Error> {
        let proof = value
            .proof
            .ok_or(ProofConversionError::MissingField("proof"))?;

        if proof.proof_system != proto::ProofSystem::Sp1 as i32 {
            return Err(ProofConversionError::UnsupportedProofSystem {
                proof_system: proof.proof_system,
            });
        }

        let public_values = value
            .public_values
            .map(TryInto::try_into)
            .transpose()?
            .map(|output: PessimisticProofOutput| {
                PessimisticProofOutput::bincode_codec()
                    .serialize(&output)
                    .map_err(|error| ProofConversionError::InvalidData {
                        field: "public_values",
                        reason: error.to_string(),
                    })
            })
            .transpose()?;

        let mut stored = StoredProof::dummy();
        let StoredProof::SP1(sp1) = &mut stored;

        sp1.proof = bincode::default()
            .deserialize(&proof.proof)
            .map_err(|error| ProofConversionError::InvalidData {
                field: "proof",
                reason: error.to_string(),
            })?;
        sp1.public_values = Default::default();
        sp1.public_values
            .write_slice(public_values.as_deref().unwrap_or(&[]));
        sp1.sp1_version = proof.version;
        sp1.tee_proof = if proof.vkey.is_empty() {
            None
        } else {
            bincode::default()
                .deserialize(&proof.vkey)
                .map_err(|error| ProofConversionError::InvalidData {
                    field: "tee_proof",
                    reason: error.to_string(),
                })?
        };

        Ok(stored)
    }
}

impl crate::schema::Codec for StoredProof {
    fn encode_into<W: std::io::Write>(&self, mut writer: W) -> Result<(), CodecError> {
        let proto = proto::PessimisticStoredProof::try_from(self)
            .map_err(|error| CodecError::Conversion(error.to_string()))?;
        let mut buf = prost::bytes::BytesMut::with_capacity(
            <proto::PessimisticStoredProof as prost::Message>::encoded_len(&proto),
        );

        <proto::PessimisticStoredProof as prost::Message>::encode(&proto, &mut buf)?;
        writer.write_all(&buf)?;

        Ok(())
    }

    fn decode(buf: &[u8]) -> Result<Self, CodecError> {
        let proto = <proto::PessimisticStoredProof as prost::Message>::decode(buf)?;
        StoredProof::try_from(proto).map_err(|error| CodecError::Conversion(error.to_string()))
    }
}

impl crate::schema::Codec for LegacyProof {
    fn encode_into<W: std::io::Write>(&self, _writer: W) -> Result<(), CodecError> {
        Err(CodecError::Conversion(
            "LegacyProof is decode-only".to_string(),
        ))
    }

    fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        let proof = bincode::default().deserialize(bytes)?;
        Ok(Self(proof))
    }
}

#[cfg(test)]
mod tests {
    use agglayer_sp1::ProofError;
    use agglayer_types::{
        aggchain_proof::Proof as AggchainProof, testutils::dummy_sp1_stark_proof_with_version,
        Proof as StoredProof,
    };

    use super::*;
    use crate::{
        schema::Codec as _,
        types::generated::agglayer::storage::v0::{ProofMode, ProofSystem},
    };

    fn sample_pessimistic_output(seed: u8) -> PessimisticProofOutput {
        PessimisticProofOutput {
            prev_local_exit_root: LocalExitRoot::new(Digest([seed; 32].into())),
            prev_pessimistic_root: Digest([seed.wrapping_add(1); 32].into()),
            l1_info_root: Digest([seed.wrapping_add(2); 32].into()),
            origin_network: u32::from(seed).into(),
            aggchain_hash: Digest([seed.wrapping_add(3); 32].into()),
            new_local_exit_root: LocalExitRoot::new(Digest([seed.wrapping_add(4); 32].into())),
            new_pessimistic_root: Digest([seed.wrapping_add(5); 32].into()),
        }
    }

    fn sample_stored_proof() -> StoredProof {
        let mut proof = StoredProof::dummy();
        let StoredProof::SP1(sp1) = &mut proof;
        sp1.sp1_version = "v6.2.1".to_owned();
        sp1.public_values.write_slice(
            PessimisticProofOutput::bincode_codec()
                .serialize(&sample_pessimistic_output(0x11))
                .unwrap()
                .as_slice(),
        );
        sp1.tee_proof = Some(vec![0xAA, 0xBB, 0xCC]);
        proof
    }

    fn assert_same_stored_proof(left: &StoredProof, right: &StoredProof) {
        let left = bincode::default().serialize(left).unwrap();
        let right = bincode::default().serialize(right).unwrap();
        assert_eq!(left, right);
    }

    #[test]
    fn aggchain_proof_proto_roundtrip_is_lossless_for_writable_versions() {
        let proof = dummy_sp1_stark_proof_with_version("v5.2.2");

        let proto = proto::Proof::try_from(&proof).unwrap();
        let decoded = AggchainProof::try_from(proto).unwrap();

        assert_eq!(decoded, proof);
    }

    #[test]
    fn aggchain_proof_proto_stores_sp1_bytes_directly() {
        let proof = dummy_sp1_stark_proof_with_version("v6.0.1");
        let AggchainProof::SP1Stark(sp1) = &proof;

        let proto = proto::Proof::try_from(&proof).unwrap();

        assert_eq!(proto.proof.as_ref(), sp1.proof.as_slice());
        assert_eq!(proto.vkey.as_ref(), sp1.vkey.as_slice());
    }

    #[test]
    fn aggchain_proof_proto_reads_direct_sp1_bytes() {
        let proof = dummy_sp1_stark_proof_with_version("v6.0.1");
        let AggchainProof::SP1Stark(sp1) = &proof;

        let decoded = AggchainProof::try_from(proto::Proof {
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
    fn aggchain_proof_proto_reads_supported_read_only_versions() {
        let proof = dummy_sp1_stark_proof_with_version("v5.2.2");
        let AggchainProof::SP1Stark(expected) = proof.clone();

        let mut proto = proto::Proof::try_from(&proof).unwrap();
        proto.version = "v6.0.1".to_owned();

        let decoded = AggchainProof::try_from(proto).unwrap();

        assert_eq!(
            decoded,
            AggchainProof::SP1Stark(SP1StarkWithContext {
                version: "v6.0.1".to_owned(),
                ..expected
            })
        );
    }

    #[test]
    fn aggchain_proof_proto_roundtrip_is_lossless_for_read_only_versions() {
        let proof = dummy_sp1_stark_proof_with_version("v6.0.1");

        let proto = proto::Proof::try_from(&proof).unwrap();
        let decoded = AggchainProof::try_from(proto).unwrap();

        assert_eq!(decoded, proof);
    }

    #[test]
    fn aggchain_proof_proto_writes_supported_read_only_versions() {
        let proof = dummy_sp1_stark_proof_with_version("v6.0.1");

        let proto = proto::Proof::try_from(&proof).unwrap();

        assert_eq!(proto.version, "v6.0.1");
    }

    #[test]
    fn aggchain_proof_proto_rejects_unknown_write_version() {
        let mut proof = dummy_sp1_stark_proof_with_version("v6.0.1");
        let AggchainProof::SP1Stark(sp1) = &mut proof;
        sp1.version = "v7.0.0".to_owned();

        let err = proto::Proof::try_from(&proof).unwrap_err();

        assert!(matches!(
            err,
            ProofConversionError::Sp1(ProofError::UnsupportedSp1VersionMajor { .. })
        ));
    }

    #[test]
    fn aggchain_proof_proto_rejects_unknown_read_version() {
        let proof = dummy_sp1_stark_proof_with_version("v5.2.2");
        let mut proto = proto::Proof::try_from(&proof).unwrap();
        proto.version = "v7.0.0".to_owned();

        let err = AggchainProof::try_from(proto).unwrap_err();

        assert!(matches!(
            err,
            ProofConversionError::Sp1(ProofError::UnsupportedSp1VersionMajor { .. })
        ));
    }

    #[test]
    fn aggchain_proof_proto_preserves_opaque_sp1_bytes() {
        let decoded = AggchainProof::try_from(proto::Proof {
            proof_system: ProofSystem::Sp1 as i32,
            version: "v5.2.2".to_owned(),
            mode: ProofMode::Compressed as i32,
            proof: vec![0xde, 0xad, 0xbe, 0xef].into(),
            vkey: vec![0xca, 0xfe].into(),
        })
        .unwrap();

        assert_eq!(
            decoded,
            AggchainProof::SP1Stark(SP1StarkWithContext {
                proof: vec![0xde, 0xad, 0xbe, 0xef],
                vkey: vec![0xca, 0xfe],
                version: "v5.2.2".to_owned(),
            })
        );
    }

    #[test]
    fn aggchain_proof_proto_rejects_unsupported_system() {
        let unsupported_system = AggchainProof::try_from(proto::Proof {
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

    #[test]
    fn stored_proof_codec_roundtrip_uses_pessimistic_wrapper() {
        let proof = sample_stored_proof();

        let bytes = proof.encode().unwrap();
        let proto = <proto::PessimisticStoredProof as prost::Message>::decode(bytes.as_slice())
            .unwrap();
        let decoded = StoredProof::decode(bytes.as_slice()).unwrap();

        assert_same_stored_proof(&decoded, &proof);
        assert_eq!(
            proto.proof.unwrap().version,
            "v6.2.1",
            "storage proof version should live in the proto envelope"
        );
        assert!(proto.public_values.is_some());
    }

    #[test]
    fn stored_proof_codec_allows_empty_public_values() {
        let proof = StoredProof::dummy();

        let bytes = proof.encode().unwrap();
        let proto = <proto::PessimisticStoredProof as prost::Message>::decode(bytes.as_slice())
            .unwrap();
        let decoded = StoredProof::decode(bytes.as_slice()).unwrap();

        assert_same_stored_proof(&decoded, &proof);
        assert!(proto.public_values.is_none());
        assert_eq!(proto.proof.unwrap().mode, proto::ProofMode::Unspecified as i32);
    }
}
