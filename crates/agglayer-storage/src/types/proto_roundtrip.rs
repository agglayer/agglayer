//! Structural round-trip tests for the storage proto envelopes.

use prost::Message as _;

use crate::types::generated::agglayer::storage::v0::{Proof, ProofMode, ProofSystem};

#[test]
fn stored_proof_wrappers_roundtrip_is_lossless() {
    use crate::types::generated::agglayer::storage::v0::{
        AggchainHash, AggchainParams, AggchainProofPublicValues, AggchainStoredProof,
        CommitImportedBridgeExits, L1InfoRoot, LocalExitRoot, PessimisticProofOutput,
        PessimisticRoot, PessimisticStoredProof,
    };

    fn local_exit_root(b: u8) -> LocalExitRoot {
        LocalExitRoot {
            value: vec![b; 32].into(),
        }
    }

    fn pessimistic_root(b: u8) -> PessimisticRoot {
        PessimisticRoot {
            value: vec![b; 32].into(),
        }
    }

    fn l1_info_root(b: u8) -> L1InfoRoot {
        L1InfoRoot {
            value: vec![b; 32].into(),
        }
    }

    fn aggchain_hash(b: u8) -> AggchainHash {
        AggchainHash {
            value: vec![b; 32].into(),
        }
    }

    fn commit_imported_bridge_exits(b: u8) -> CommitImportedBridgeExits {
        CommitImportedBridgeExits {
            value: vec![b; 32].into(),
        }
    }

    fn aggchain_params(b: u8) -> AggchainParams {
        AggchainParams {
            value: vec![b; 32].into(),
        }
    }

    let aggchain = AggchainStoredProof {
        proof: Some(Proof {
            proof_system: ProofSystem::Sp1 as i32,
            version: "v5.2.2".to_string(),
            mode: ProofMode::Compressed as i32,
            proof: vec![1, 2, 3, 4, 5].into(),
            vkey: vec![0xAA; 32].into(),
        }),
        public_values: Some(AggchainProofPublicValues {
            prev_local_exit_root: Some(local_exit_root(0x01)),
            new_local_exit_root: Some(local_exit_root(0x02)),
            l1_info_root: Some(l1_info_root(0x03)),
            origin_network: 4,
            commit_imported_bridge_exits: Some(commit_imported_bridge_exits(0x05)),
            aggchain_params: Some(aggchain_params(0x06)),
        }),
    };
    let aggchain_bytes = aggchain.encode_to_vec();
    let aggchain_decoded = AggchainStoredProof::decode(aggchain_bytes.as_slice()).unwrap();
    assert_eq!(aggchain, aggchain_decoded);

    let pessimistic = PessimisticStoredProof {
        proof: Some(Proof {
            proof_system: ProofSystem::Sp1 as i32,
            version: "v5.2.2".to_string(),
            mode: ProofMode::Compressed as i32,
            proof: vec![6, 7, 8, 9].into(),
            vkey: vec![0xBB; 32].into(),
        }),
        public_values: Some(PessimisticProofOutput {
            prev_local_exit_root: Some(local_exit_root(0x11)),
            prev_pessimistic_root: Some(pessimistic_root(0x12)),
            l1_info_root: Some(l1_info_root(0x13)),
            origin_network: 0x14,
            aggchain_hash: Some(aggchain_hash(0x15)),
            new_local_exit_root: Some(local_exit_root(0x16)),
            new_pessimistic_root: Some(pessimistic_root(0x17)),
        }),
    };
    let pessimistic_bytes = pessimistic.encode_to_vec();
    let pessimistic_decoded = PessimisticStoredProof::decode(pessimistic_bytes.as_slice()).unwrap();
    assert_eq!(pessimistic, pessimistic_decoded);
}

#[test]
fn proof_roundtrip_is_lossless() {
    let original = Proof {
        proof_system: ProofSystem::Sp1 as i32,
        version: "v5.2.2".to_string(),
        mode: ProofMode::Compressed as i32,
        proof: vec![1, 2, 3, 4, 5].into(),
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
    assert!(decoded.vkey.is_empty());
}

#[test]
fn certificate_roundtrip_is_lossless() {
    use crate::types::generated::agglayer::storage::v0::{
        aggchain_data, imported_bridge_exit, Address, AggchainData, AggchainParams,
        AggchainProofPublicValues, Amount, BlockHash, BridgeExit, Certificate, ClaimFromMainnet,
        CommitImportedBridgeExits, Generic, GlobalExitRoot, GlobalIndex, ImportedBridgeExit,
        L1InfoRoot, L1InfoTreeLeaf, L1InfoTreeLeafWithContext, LeafType, LocalExitRoot,
        MainnetExitRoot, MerkleProof, MerkleRoot, Metadata, MultisigEntry, MultisigOnly,
        MultisigPayload, Proof, ProofMode, ProofSystem, RollupExitRoot, Signature, TokenInfo,
    };

    fn address(b: u8) -> Address {
        Address {
            address: vec![b; 20].into(),
        }
    }
    fn local_exit_root(b: u8) -> LocalExitRoot {
        LocalExitRoot {
            value: vec![b; 32].into(),
        }
    }
    fn merkle_root(b: u8) -> MerkleRoot {
        MerkleRoot {
            value: vec![b; 32].into(),
        }
    }
    fn global_exit_root(b: u8) -> GlobalExitRoot {
        GlobalExitRoot {
            value: vec![b; 32].into(),
        }
    }
    fn block_hash(b: u8) -> BlockHash {
        BlockHash {
            hash: vec![b; 32].into(),
        }
    }
    fn rollup_exit_root(b: u8) -> RollupExitRoot {
        RollupExitRoot {
            value: vec![b; 32].into(),
        }
    }
    fn mainnet_exit_root(b: u8) -> MainnetExitRoot {
        MainnetExitRoot {
            value: vec![b; 32].into(),
        }
    }
    fn global_index(b: u8) -> GlobalIndex {
        GlobalIndex {
            value: vec![b; 32].into(),
        }
    }
    fn l1_info_root(b: u8) -> L1InfoRoot {
        L1InfoRoot {
            value: vec![b; 32].into(),
        }
    }
    fn commit_imported_bridge_exits(b: u8) -> CommitImportedBridgeExits {
        CommitImportedBridgeExits {
            value: vec![b; 32].into(),
        }
    }
    fn aggchain_params(b: u8) -> AggchainParams {
        AggchainParams {
            value: vec![b; 32].into(),
        }
    }
    fn metadata(b: u8) -> Metadata {
        Metadata {
            value: vec![b; 32].into(),
        }
    }
    fn amount(b: u8) -> Amount {
        Amount {
            value: vec![b; 32].into(),
        }
    }
    fn signature(b: u8) -> Signature {
        Signature {
            value: vec![b; 65].into(),
        }
    }

    let proof = Proof {
        proof_system: ProofSystem::Sp1 as i32,
        version: "v5.2.2".to_string(),
        mode: ProofMode::Compressed as i32,
        proof: vec![1, 2, 3, 4, 5].into(),
        vkey: vec![0xAA; 32].into(),
    };

    let bridge_exit = BridgeExit {
        leaf_type: LeafType::Transfer as i32,
        token_info: Some(TokenInfo {
            origin_network: 7,
            origin_token_address: Some(address(0x11)),
        }),
        dest_network: 42,
        dest_address: Some(address(0x22)),
        amount: Some(amount(0x33)),
        metadata: Some(metadata(0x44)),
    };

    let l1_leaf = L1InfoTreeLeafWithContext {
        l1_info_tree_index: 3,
        rer: Some(rollup_exit_root(0x55)),
        mer: Some(mainnet_exit_root(0x66)),
        inner: Some(L1InfoTreeLeaf {
            global_exit_root: Some(global_exit_root(0x77)),
            block_hash: Some(block_hash(0x88)),
            timestamp: 1_000_000,
        }),
    };

    let imported_bridge_exit = ImportedBridgeExit {
        bridge_exit: Some(bridge_exit.clone()),
        global_index: Some(global_index(0x99)),
        claim: Some(imported_bridge_exit::Claim::Mainnet(ClaimFromMainnet {
            proof_leaf_mer: Some(MerkleProof {
                root: Some(merkle_root(0xAA)),
                siblings: vec![merkle_root(0xBB), merkle_root(0xCC)],
            }),
            proof_ger_l1root: Some(MerkleProof {
                root: Some(merkle_root(0xDD)),
                siblings: vec![],
            }),
            l1_leaf: Some(l1_leaf),
        })),
    };

    let aggchain_data = AggchainData {
        data: Some(aggchain_data::Data::Generic(Generic {
            proof: Some(proof),
            aggchain_params: Some(aggchain_params(0xEE)),
            signature: Some(signature(0xFF)),
            public_values: Some(AggchainProofPublicValues {
                prev_local_exit_root: Some(local_exit_root(0x01)),
                new_local_exit_root: Some(local_exit_root(0x02)),
                l1_info_root: Some(l1_info_root(0x03)),
                origin_network: 4,
                commit_imported_bridge_exits: Some(commit_imported_bridge_exits(0x05)),
                aggchain_params: Some(aggchain_params(0x06)),
            }),
        })),
    };

    let original = Certificate {
        network_id: 1,
        height: 0,
        prev_local_exit_root: Some(local_exit_root(0x10)),
        new_local_exit_root: Some(local_exit_root(0x20)),
        bridge_exits: vec![bridge_exit],
        imported_bridge_exits: vec![imported_bridge_exit],
        aggchain_data: Some(aggchain_data),
        metadata: Some(metadata(0x30)),
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
                            signature: Some(signature(0xA1)),
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
        aggchain_data, AggchainData, Certificate, Metadata,
    };

    let metadata_only = Certificate {
        metadata: Some(Metadata {
            value: vec![0xAB; 32].into(),
        }),
        ..Default::default()
    };
    let metadata_bytes = metadata_only.encode_to_vec();
    assert_eq!(metadata_bytes.first().copied(), Some(0x3a));

    let aggchain_only = Certificate {
        aggchain_data: Some(AggchainData {
            data: Some(aggchain_data::Data::Ecdsa(
                crate::types::generated::agglayer::storage::v0::Signature {
                    value: vec![0xCD; 65].into(),
                },
            )),
        }),
        ..Default::default()
    };
    let aggchain_bytes = aggchain_only.encode_to_vec();
    assert_eq!(aggchain_bytes.first().copied(), Some(0x42));
}
