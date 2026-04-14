use agglayer_types::{
    aggchain_proof::{AggchainData, Proof},
    bincode,
    primitives::SignatureError,
    Certificate, CertificateId, Digest, EpochConfiguration,
};
use prost::Message;
use sp1_sdk_v5::Prover as _;

use super::Error;
use crate::node::types::v1;

const LEGACY_EMPTY_ELF: &[u8] =
    include_bytes!("../../../../agglayer-storage/src/types/certificate/tests/empty.elf");

fn legacy_sp1_bytes() -> (Vec<u8>, Vec<u8>) {
    let client = sp1_sdk_v5::ProverClient::builder().mock().build();
    let (proving_key, verifying_key) = client.setup(LEGACY_EMPTY_ELF);
    let dummy_proof = sp1_sdk_v5::SP1ProofWithPublicValues::create_mock_proof(
        &proving_key,
        sp1_sdk_v5::SP1PublicValues::new(),
        sp1_sdk_v5::SP1ProofMode::Compressed,
        sp1_sdk_v5::SP1_CIRCUIT_VERSION,
    );
    let proof = dummy_proof.proof.try_as_compressed().unwrap();

    (
        bincode::sp1v4().serialize(proof.as_ref()).unwrap(),
        bincode::sp1v4().serialize(&verifying_key).unwrap(),
    )
}

fn sample_public_values() -> agglayer_interop::types::aggchain_proof::AggchainProofPublicValues {
    agglayer_interop::types::aggchain_proof::AggchainProofPublicValues {
        prev_local_exit_root: Digest([0x11; 32]),
        new_local_exit_root: Digest([0x22; 32]),
        l1_info_root: Digest([0x33; 32]),
        origin_network: agglayer_types::NetworkId::new(9),
        commit_imported_bridge_exits: Digest([0x44; 32]),
        aggchain_params: Digest([0x55; 32]),
    }
}

#[rstest::rstest]
#[case::error("no_proof", Error::missing_field("proof"))]
#[case::error("bad_data", Error::invalid_data("invalid value".to_owned()))]
#[case::error("bad_data_in_field", Error::invalid_data("invalid value".to_owned()).inside_field("value"))]
#[case::error("bad_data_in_nested", Error::invalid_data("invalid value".to_owned()).inside_field("value").inside_field("data"))]
#[case::error("failed_ser", Error::serializing_proof(Box::new(bincode::ErrorKind::Custom("failed".to_owned()))))]
#[case::error("failed_deser", Error::deserializing_proof(Box::new(bincode::ErrorKind::Custom("failed".to_owned()))))]
#[case::error("bad_sig", Error::parsing_signature(SignatureError::InvalidParity(5)))]
#[case::error("bad_sig_in_nested", Error::parsing_signature(SignatureError::InvalidParity(5)).inside_field("signature").inside_field("data"))]
fn error_messages(#[case] name: &str, #[case] error: Error) {
    insta::assert_snapshot!(format!("{name}/display"), error);
    insta::assert_debug_snapshot!(format!("{name}/kind"), error.kind());
    insta::with_settings!({
        filters => vec![
            // Remove the whole "Location:" block (common eyre pretty format)
            (r"(?m)^Location:\n([ \t]+.*\n?)+", "Location:\n    <REDACTED>\n"),
        ],
    }, {
        insta::assert_snapshot!(
            format!("{name}/debug"),
            format!("{:?}", eyre::Error::from(error))
        );
    });
}

macro_rules! make_parser_fuzzers {
    ($test:ident, $proto:ty, $type:ty) => {
        #[test]
        fn $test() {
            bolero::check!().for_each(|bytes| {
                if let Ok(proto) = <$proto>::decode(bytes) {
                    let _ = <$type>::try_from(proto);
                };
            })
        }
    };
}

make_parser_fuzzers!(fuzz_parser_certificate_id, v1::CertificateId, CertificateId);
make_parser_fuzzers!(
    fuzz_parser_epoch_configuration,
    v1::EpochConfiguration,
    EpochConfiguration
);

macro_rules! make_round_trip_fuzzers {
    ($test:ident, $proto:ty, $type:ty) => {
        #[test]
        fn $test() {
            bolero::check!()
                .with_arbitrary::<$type>()
                .for_each(|input: &$type| {
                    let proto: $proto = input.clone().into();
                    let output = <$type>::try_from(proto).unwrap();
                    assert_eq!(input, &output);
                })
        }
    };
}

make_round_trip_fuzzers!(
    fuzz_round_trip_certificate_id,
    v1::CertificateId,
    CertificateId
);
make_round_trip_fuzzers!(
    fuzz_round_trip_epoch_configuration,
    v1::EpochConfiguration,
    EpochConfiguration
);

#[test]
fn certificate_round_trip_preserves_readable_legacy_sp1_proof() {
    let (proof_bytes, vkey_bytes) = legacy_sp1_bytes();
    let legacy_version = sp1_sdk_v5::SP1_CIRCUIT_VERSION.to_string();

    let proto = v1::Certificate {
        network_id: 17,
        height: 3,
        prev_local_exit_root: Some(agglayer_interop::grpc::v1::FixedBytes32 {
            value: vec![0; 32].into(),
        }),
        new_local_exit_root: Some(agglayer_interop::grpc::v1::FixedBytes32 {
            value: vec![0; 32].into(),
        }),
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        aggchain_data: Some(agglayer_interop::grpc::v1::AggchainData {
            data: Some(agglayer_interop::grpc::v1::aggchain_data::Data::Generic(
                agglayer_interop::grpc::v1::AggchainProof {
                    aggchain_params: Some(Digest([0x42; 32]).into()),
                    signature: None,
                    context: Default::default(),
                    proof: Some(agglayer_interop::grpc::v1::aggchain_proof::Proof::Sp1Stark(
                        agglayer_interop::grpc::v1::Sp1StarkProof {
                            version: legacy_version.clone(),
                            proof: proof_bytes.clone().into(),
                            vkey: vkey_bytes.clone().into(),
                        },
                    )),
                },
            )),
        }),
        metadata: None,
        custom_chain_data: Vec::new().into(),
        l1_info_tree_leaf_count: None,
    };

    let output = Certificate::try_from(proto).unwrap();

    let AggchainData::Generic { ref proof, .. } = output.aggchain_data else {
        panic!("expected generic aggchain data")
    };
    let Proof::SP1Stark(proof) = proof;

    assert_eq!(proof.version, legacy_version);
    assert_eq!(proof.proof, proof_bytes);
    assert_eq!(proof.vkey, vkey_bytes);

    let encoded: v1::Certificate = output.try_into().unwrap();
    let sp1 = match encoded.aggchain_data.unwrap().data.unwrap() {
        agglayer_interop::grpc::v1::aggchain_data::Data::Generic(proof) => {
            match proof.proof.unwrap() {
                agglayer_interop::grpc::v1::aggchain_proof::Proof::Sp1Stark(proof) => proof,
            }
        }
        _ => panic!("expected generic aggchain proof"),
    };

    assert_eq!(sp1.version, sp1_sdk_v5::SP1_CIRCUIT_VERSION);
    assert_eq!(sp1.proof, proof_bytes);
    assert_eq!(sp1.vkey, vkey_bytes);
}

#[test]
fn certificate_rejects_multisig_index_overflow() {
    let proto = v1::Certificate {
        network_id: 17,
        height: 3,
        prev_local_exit_root: Some(agglayer_interop::grpc::v1::FixedBytes32 {
            value: vec![0; 32].into(),
        }),
        new_local_exit_root: Some(agglayer_interop::grpc::v1::FixedBytes32 {
            value: vec![0; 32].into(),
        }),
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        aggchain_data: Some(agglayer_interop::grpc::v1::AggchainData {
            data: Some(agglayer_interop::grpc::v1::aggchain_data::Data::Multisig(
                agglayer_interop::grpc::v1::Multisig {
                    data: Some(agglayer_interop::grpc::v1::multisig::Data::Ecdsa(
                        agglayer_interop::grpc::v1::EcdsaMultisig {
                            signatures: vec![
                                agglayer_interop::grpc::v1::ecdsa_multisig::EcdsaMultisigEntry {
                                    index: u32::MAX,
                                    signature: None,
                                },
                            ],
                        },
                    )),
                },
            )),
        }),
        metadata: None,
        custom_chain_data: Vec::new().into(),
        l1_info_tree_leaf_count: None,
    };

    let error = Certificate::try_from(proto).unwrap_err();

    assert!(
        error.to_string().contains("too many signers") || error.to_string().contains("overflow")
    );
}

#[test]
fn certificate_rejects_malformed_sp1_bytes_on_ingress() {
    let proto = v1::Certificate {
        network_id: 17,
        height: 3,
        prev_local_exit_root: Some(agglayer_interop::grpc::v1::FixedBytes32 {
            value: vec![0; 32].into(),
        }),
        new_local_exit_root: Some(agglayer_interop::grpc::v1::FixedBytes32 {
            value: vec![0; 32].into(),
        }),
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        aggchain_data: Some(agglayer_interop::grpc::v1::AggchainData {
            data: Some(agglayer_interop::grpc::v1::aggchain_data::Data::Generic(
                agglayer_interop::grpc::v1::AggchainProof {
                    aggchain_params: Some(Digest([0x42; 32]).into()),
                    signature: None,
                    context: Default::default(),
                    proof: Some(agglayer_interop::grpc::v1::aggchain_proof::Proof::Sp1Stark(
                        agglayer_interop::grpc::v1::Sp1StarkProof {
                            version: "v6.0.0".to_string(),
                            proof: vec![1, 2, 3, 4].into(),
                            vkey: vec![5, 6, 7, 8].into(),
                        },
                    )),
                },
            )),
        }),
        metadata: None,
        custom_chain_data: Vec::new().into(),
        l1_info_tree_leaf_count: None,
    };

    let error = Certificate::try_from(proto).unwrap_err();

    assert!(error.to_string().contains("deserialize"));
}

#[test]
fn generic_v1_wire_preserves_legacy_public_values_shape() {
    let public_values = Some(Box::new(sample_public_values()));
    let (proof_bytes, vkey_bytes) = legacy_sp1_bytes();
    let version = sp1_sdk_v5::SP1_CIRCUIT_VERSION.to_string();

    let proto = v1::Certificate {
        network_id: 17,
        height: 3,
        prev_local_exit_root: Some(agglayer_interop::grpc::v1::FixedBytes32 {
            value: vec![0; 32].into(),
        }),
        new_local_exit_root: Some(agglayer_interop::grpc::v1::FixedBytes32 {
            value: vec![0; 32].into(),
        }),
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        aggchain_data: Some(agglayer_interop::grpc::v1::AggchainData {
            data: Some(agglayer_interop::grpc::v1::aggchain_data::Data::Generic(
                agglayer_interop::grpc::v1::AggchainProof {
                    aggchain_params: Some(Digest([0x42; 32]).into()),
                    signature: None,
                    context: std::collections::HashMap::from([(
                        "public_values".to_owned(),
                        agglayer_interop::types::bincode::default()
                            .serialize(&public_values)
                            .unwrap()
                            .into(),
                    )]),
                    proof: Some(agglayer_interop::grpc::v1::aggchain_proof::Proof::Sp1Stark(
                        agglayer_interop::grpc::v1::Sp1StarkProof {
                            version,
                            proof: proof_bytes.into(),
                            vkey: vkey_bytes.into(),
                        },
                    )),
                },
            )),
        }),
        metadata: None,
        custom_chain_data: Vec::new().into(),
        l1_info_tree_leaf_count: None,
    };

    let output = Certificate::try_from(proto).unwrap();
    let AggchainData::Generic {
        public_values: ref decoded,
        ..
    } = output.aggchain_data
    else {
        panic!("expected generic aggchain data")
    };
    assert_eq!(
        agglayer_interop::types::bincode::default()
            .serialize(&decoded)
            .unwrap(),
        agglayer_interop::types::bincode::default()
            .serialize(&public_values)
            .unwrap()
    );

    let encoded: v1::Certificate = output.try_into().unwrap();
    let encoded = match encoded.aggchain_data.unwrap().data.unwrap() {
        agglayer_interop::grpc::v1::aggchain_data::Data::Generic(proof) => proof,
        _ => panic!("expected generic aggchain proof"),
    };

    let on_wire: Option<Box<agglayer_interop::types::aggchain_proof::AggchainProofPublicValues>> =
        agglayer_interop::types::bincode::default()
            .deserialize(encoded.context.get("public_values").unwrap().as_ref())
            .unwrap();
    assert_eq!(on_wire, public_values);
}

#[test]
fn multisig_and_aggchain_proof_v1_wire_preserves_bare_public_values_shape() {
    let public_values = sample_public_values();
    let (proof_bytes, vkey_bytes) = legacy_sp1_bytes();
    let version = sp1_sdk_v5::SP1_CIRCUIT_VERSION.to_string();

    let proto = v1::Certificate {
        network_id: 17,
        height: 3,
        prev_local_exit_root: Some(agglayer_interop::grpc::v1::FixedBytes32 {
            value: vec![0; 32].into(),
        }),
        new_local_exit_root: Some(agglayer_interop::grpc::v1::FixedBytes32 {
            value: vec![0; 32].into(),
        }),
        bridge_exits: Vec::new(),
        imported_bridge_exits: Vec::new(),
        aggchain_data: Some(agglayer_interop::grpc::v1::AggchainData {
            data: Some(
                agglayer_interop::grpc::v1::aggchain_data::Data::MultisigAndAggchainProof(
                    agglayer_interop::grpc::v1::AggchainProofWithMultisig {
                        multisig: Some(agglayer_interop::grpc::v1::Multisig {
                            data: Some(agglayer_interop::grpc::v1::multisig::Data::Ecdsa(
                                agglayer_interop::grpc::v1::EcdsaMultisig {
                                    signatures: vec![agglayer_interop::grpc::v1::ecdsa_multisig::EcdsaMultisigEntry {
                                        index: 0,
                                        signature: None,
                                    }],
                                },
                            )),
                        }),
                        aggchain_proof: Some(agglayer_interop::grpc::v1::AggchainProof {
                            aggchain_params: Some(Digest([0x42; 32]).into()),
                            signature: None,
                            context: std::collections::HashMap::from([(
                                "public_values".to_owned(),
                                agglayer_interop::types::bincode::default()
                                    .serialize(&public_values)
                                    .unwrap()
                                    .into(),
                            )]),
                            proof: Some(agglayer_interop::grpc::v1::aggchain_proof::Proof::Sp1Stark(
                                agglayer_interop::grpc::v1::Sp1StarkProof {
                                    version,
                                    proof: proof_bytes.into(),
                                    vkey: vkey_bytes.into(),
                                },
                            )),
                        }),
                    },
                ),
            ),
        }),
        metadata: None,
        custom_chain_data: Vec::new().into(),
        l1_info_tree_leaf_count: None,
    };

    let output = Certificate::try_from(proto).unwrap();
    let AggchainData::MultisigAndAggchainProof {
        ref aggchain_proof, ..
    } = output.aggchain_data
    else {
        panic!("expected multisig and aggchain proof")
    };
    let decoded = aggchain_proof.public_values.as_ref().unwrap();
    assert_eq!(
        agglayer_interop::types::bincode::default()
            .serialize(decoded)
            .unwrap(),
        agglayer_interop::types::bincode::default()
            .serialize(&public_values)
            .unwrap()
    );

    let encoded: v1::Certificate = output.try_into().unwrap();
    let encoded = match encoded.aggchain_data.unwrap().data.unwrap() {
        agglayer_interop::grpc::v1::aggchain_data::Data::MultisigAndAggchainProof(
            with_multisig,
        ) => with_multisig.aggchain_proof.unwrap(),
        _ => panic!("expected multisig and aggchain proof"),
    };
    let on_wire: agglayer_interop::types::aggchain_proof::AggchainProofPublicValues =
        agglayer_interop::types::bincode::default()
            .deserialize(encoded.context.get("public_values").unwrap().as_ref())
            .unwrap();
    assert_eq!(on_wire, public_values);
}
