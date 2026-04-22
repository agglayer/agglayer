//! Structural round-trip tests for the storage proto envelopes.
//! No typed conversion is exercised here — that is Task 4.

use prost::Message as _;

use crate::types::generated::agglayer::storage::v0::{ProofMode, ProofSystem, StorageProof};

#[test]
fn storage_proof_roundtrip_is_lossless() {
    let original = StorageProof {
        proof_system: ProofSystem::Sp1 as i32,
        version: "v5.2.2".to_string(),
        mode: ProofMode::Compressed as i32,
        proof: vec![1, 2, 3, 4, 5].into(),
        public_values: vec![9, 8, 7].into(),
        vkey: vec![0xAA; 32].into(),
    };

    let bytes = original.encode_to_vec();
    let decoded = StorageProof::decode(bytes.as_slice()).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn storage_proof_defaults_on_empty_bytes() {
    let decoded = StorageProof::decode(&[][..]).unwrap();
    assert_eq!(decoded.proof_system, ProofSystem::Unspecified as i32);
    assert!(decoded.version.is_empty());
    assert_eq!(decoded.mode, ProofMode::Unspecified as i32);
    assert!(decoded.proof.is_empty());
    assert!(decoded.public_values.is_empty());
    assert!(decoded.vkey.is_empty());
}
