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

#[test]
fn storage_certificate_roundtrip_is_lossless() {
    use crate::types::generated::agglayer::storage::v0::{
        storage_aggchain_data, storage_imported_bridge_exit, ProofMode, ProofSystem,
        StorageAggchainData, StorageAggchainProofPublicValues, StorageBridgeExit,
        StorageCertificate, StorageClaimFromMainnet, StorageFixedBytes20, StorageFixedBytes32,
        StorageFixedBytes65, StorageGeneric, StorageImportedBridgeExit, StorageL1InfoTreeLeaf,
        StorageL1InfoTreeLeafWithContext, StorageLeafType, StorageMerkleProof,
        StorageMultisigEntry, StorageMultisigOnly, StorageMultisigPayload, StorageProof,
        StorageTokenInfo,
    };

    fn fb20(b: u8) -> StorageFixedBytes20 {
        StorageFixedBytes20 {
            value: vec![b; 20].into(),
        }
    }
    fn fb32(b: u8) -> StorageFixedBytes32 {
        StorageFixedBytes32 {
            value: vec![b; 32].into(),
        }
    }
    fn fb65(b: u8) -> StorageFixedBytes65 {
        StorageFixedBytes65 {
            value: vec![b; 65].into(),
        }
    }

    let proof = StorageProof {
        proof_system: ProofSystem::Sp1 as i32,
        version: "v5.2.2".to_string(),
        mode: ProofMode::Compressed as i32,
        proof: vec![1, 2, 3, 4, 5].into(),
        public_values: vec![9, 8, 7].into(),
        vkey: vec![0xAA; 32].into(),
    };

    let bridge_exit = StorageBridgeExit {
        leaf_type: StorageLeafType::Transfer as i32,
        token_info: Some(StorageTokenInfo {
            origin_network: 7,
            origin_token_address: Some(fb20(0x11)),
        }),
        dest_network: 42,
        dest_address: Some(fb20(0x22)),
        amount: Some(fb32(0x33)),
        metadata: Some(fb32(0x44)),
    };

    let l1_leaf = StorageL1InfoTreeLeafWithContext {
        l1_info_tree_index: 3,
        rer: Some(fb32(0x55)),
        mer: Some(fb32(0x66)),
        inner: Some(StorageL1InfoTreeLeaf {
            global_exit_root: Some(fb32(0x77)),
            block_hash: Some(fb32(0x88)),
            timestamp: 1_000_000,
        }),
    };

    let imported_bridge_exit = StorageImportedBridgeExit {
        bridge_exit: Some(bridge_exit.clone()),
        global_index: Some(fb32(0x99)),
        claim: Some(storage_imported_bridge_exit::Claim::Mainnet(
            StorageClaimFromMainnet {
                proof_leaf_mer: Some(StorageMerkleProof {
                    root: Some(fb32(0xAA)),
                    siblings: vec![fb32(0xBB), fb32(0xCC)],
                }),
                proof_ger_l1root: Some(StorageMerkleProof {
                    root: Some(fb32(0xDD)),
                    siblings: vec![],
                }),
                l1_leaf: Some(l1_leaf),
            },
        )),
    };

    let aggchain_data = StorageAggchainData {
        data: Some(storage_aggchain_data::Data::Generic(StorageGeneric {
            proof: Some(proof),
            aggchain_params: Some(fb32(0xEE)),
            signature: Some(fb65(0xFF)),
            public_values: Some(StorageAggchainProofPublicValues {
                prev_local_exit_root: Some(fb32(0x01)),
                new_local_exit_root: Some(fb32(0x02)),
                l1_info_root: Some(fb32(0x03)),
                origin_network: 4,
                commit_imported_bridge_exits: Some(fb32(0x05)),
                aggchain_params: Some(fb32(0x06)),
            }),
        })),
    };

    let original = StorageCertificate {
        network_id: 1,
        height: 0,
        prev_local_exit_root: Some(fb32(0x10)),
        new_local_exit_root: Some(fb32(0x20)),
        bridge_exits: vec![bridge_exit],
        imported_bridge_exits: vec![imported_bridge_exit],
        aggchain_data: Some(aggchain_data),
        metadata: Some(fb32(0x30)),
        custom_chain_data: vec![0xCA, 0xFE].into(),
        l1_info_tree_leaf_count: Some(5),
    };

    let bytes = prost::Message::encode_to_vec(&original);
    let decoded = <StorageCertificate as prost::Message>::decode(&*bytes).unwrap();
    assert_eq!(original, decoded);

    // Also exercise the multisig-only variant.
    let multisig_only = StorageCertificate {
        aggchain_data: Some(StorageAggchainData {
            data: Some(storage_aggchain_data::Data::MultisigOnly(
                StorageMultisigOnly {
                    multisig: Some(StorageMultisigPayload {
                        signatures: vec![
                            StorageMultisigEntry {
                                signature: Some(fb65(0xA1)),
                            },
                            StorageMultisigEntry { signature: None },
                        ],
                    }),
                },
            )),
        }),
        ..original.clone()
    };
    let bytes = prost::Message::encode_to_vec(&multisig_only);
    let decoded = <StorageCertificate as prost::Message>::decode(&*bytes).unwrap();
    assert_eq!(multisig_only, decoded);
}
