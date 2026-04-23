//! Structural round-trip tests for the storage proto envelopes.

use prost::Message as _;

use crate::types::generated::agglayer::storage::v0::{Proof, ProofMode, ProofSystem};

#[test]
fn proof_roundtrip_is_lossless() {
    let original = Proof {
        proof_system: ProofSystem::Sp1 as i32,
        version: "v5.2.2".to_string(),
        mode: ProofMode::Compressed as i32,
        proof: vec![1, 2, 3, 4, 5].into(),
        public_values: vec![9, 8, 7].into(),
        vkey: vec![0xAA; 32].into(),
    };

    let bytes = original.encode_to_vec();
    let decoded = Proof::decode(bytes.as_slice()).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn proof_defaults_on_empty_bytes() {
    let decoded = Proof::decode(&[][..]).unwrap();
    assert_eq!(decoded.proof_system, ProofSystem::Unspecified as i32);
    assert!(decoded.version.is_empty());
    assert_eq!(decoded.mode, ProofMode::Unspecified as i32);
    assert!(decoded.proof.is_empty());
    assert!(decoded.public_values.is_empty());
    assert!(decoded.vkey.is_empty());
}

#[test]
fn certificate_roundtrip_is_lossless() {
    use crate::types::generated::agglayer::storage::v0::{
        aggchain_data, imported_bridge_exit, AggchainData, AggchainProofPublicValues, BridgeExit,
        Certificate, ClaimFromMainnet, FixedBytes20, FixedBytes32, FixedBytes65, Generic,
        ImportedBridgeExit, L1InfoTreeLeaf, L1InfoTreeLeafWithContext, LeafType, MerkleProof,
        MultisigEntry, MultisigOnly, MultisigPayload, Proof, ProofMode, ProofSystem, TokenInfo,
    };

    fn fb20(b: u8) -> FixedBytes20 {
        FixedBytes20 {
            value: vec![b; 20].into(),
        }
    }
    fn fb32(b: u8) -> FixedBytes32 {
        FixedBytes32 {
            value: vec![b; 32].into(),
        }
    }
    fn fb65(b: u8) -> FixedBytes65 {
        FixedBytes65 {
            value: vec![b; 65].into(),
        }
    }

    let proof = Proof {
        proof_system: ProofSystem::Sp1 as i32,
        version: "v5.2.2".to_string(),
        mode: ProofMode::Compressed as i32,
        proof: vec![1, 2, 3, 4, 5].into(),
        public_values: vec![9, 8, 7].into(),
        vkey: vec![0xAA; 32].into(),
    };

    let bridge_exit = BridgeExit {
        leaf_type: LeafType::Transfer as i32,
        token_info: Some(TokenInfo {
            origin_network: 7,
            origin_token_address: Some(fb20(0x11)),
        }),
        dest_network: 42,
        dest_address: Some(fb20(0x22)),
        amount: Some(fb32(0x33)),
        metadata: Some(fb32(0x44)),
    };

    let l1_leaf = L1InfoTreeLeafWithContext {
        l1_info_tree_index: 3,
        rer: Some(fb32(0x55)),
        mer: Some(fb32(0x66)),
        inner: Some(L1InfoTreeLeaf {
            global_exit_root: Some(fb32(0x77)),
            block_hash: Some(fb32(0x88)),
            timestamp: 1_000_000,
        }),
    };

    let imported_bridge_exit = ImportedBridgeExit {
        bridge_exit: Some(bridge_exit.clone()),
        global_index: Some(fb32(0x99)),
        claim: Some(imported_bridge_exit::Claim::Mainnet(ClaimFromMainnet {
            proof_leaf_mer: Some(MerkleProof {
                root: Some(fb32(0xAA)),
                siblings: vec![fb32(0xBB), fb32(0xCC)],
            }),
            proof_ger_l1root: Some(MerkleProof {
                root: Some(fb32(0xDD)),
                siblings: vec![],
            }),
            l1_leaf: Some(l1_leaf),
        })),
    };

    let aggchain_data = AggchainData {
        data: Some(aggchain_data::Data::Generic(Generic {
            proof: Some(proof),
            aggchain_params: Some(fb32(0xEE)),
            signature: Some(fb65(0xFF)),
            public_values: Some(AggchainProofPublicValues {
                prev_local_exit_root: Some(fb32(0x01)),
                new_local_exit_root: Some(fb32(0x02)),
                l1_info_root: Some(fb32(0x03)),
                origin_network: 4,
                commit_imported_bridge_exits: Some(fb32(0x05)),
                aggchain_params: Some(fb32(0x06)),
            }),
        })),
    };

    let original = Certificate {
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
    let decoded = <Certificate as prost::Message>::decode(&*bytes).unwrap();
    assert_eq!(original, decoded);

    // Also exercise the multisig-only variant.
    let multisig_only = Certificate {
        aggchain_data: Some(AggchainData {
            data: Some(aggchain_data::Data::MultisigOnly(MultisigOnly {
                multisig: Some(MultisigPayload {
                    signatures: vec![
                        MultisigEntry {
                            signature: Some(fb65(0xA1)),
                        },
                        MultisigEntry { signature: None },
                    ],
                }),
            })),
        }),
        ..original.clone()
    };
    let bytes = prost::Message::encode_to_vec(&multisig_only);
    let decoded = <Certificate as prost::Message>::decode(&*bytes).unwrap();
    assert_eq!(multisig_only, decoded);
}

#[test]
fn certificate_keeps_canonical_metadata_and_aggchain_tags() {
    use crate::types::generated::agglayer::storage::v0::{
        aggchain_data, AggchainData, Certificate, FixedBytes32,
    };

    let metadata_only = Certificate {
        metadata: Some(FixedBytes32 {
            value: vec![0xAB; 32].into(),
        }),
        ..Default::default()
    };
    let metadata_bytes = metadata_only.encode_to_vec();
    assert_eq!(metadata_bytes.first().copied(), Some(0x3a));

    let aggchain_only = Certificate {
        aggchain_data: Some(AggchainData {
            data: Some(aggchain_data::Data::Ecdsa(
                crate::types::generated::agglayer::storage::v0::FixedBytes65 {
                    value: vec![0xCD; 65].into(),
                },
            )),
        }),
        ..Default::default()
    };
    let aggchain_bytes = aggchain_only.encode_to_vec();
    assert_eq!(aggchain_bytes.first().copied(), Some(0x42));
}
