use agglayer_types::{
    aggchain_proof::{AggchainData, Proof},
    bincode,
    primitives::SignatureError,
    Certificate, CertificateId, Digest, EpochConfiguration,
};
use prost::Message;

use super::Error;
use crate::node::types::v1;

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
                            version: "v4.0.0-rc.3".to_string(),
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

    let output = Certificate::try_from(proto).unwrap();

    let AggchainData::Generic { ref proof, .. } = output.aggchain_data else {
        panic!("expected generic aggchain data")
    };
    let Proof::SP1Stark(proof) = proof;

    assert_eq!(proof.version, "v4.0.0-rc.3");
    assert_eq!(proof.proof, vec![1, 2, 3, 4]);
    assert_eq!(proof.vkey, vec![5, 6, 7, 8]);

    let encoded: v1::Certificate = output.try_into().unwrap();
    let sp1 = match encoded.aggchain_data.unwrap().data.unwrap() {
        agglayer_interop::grpc::v1::aggchain_data::Data::Generic(proof) => {
            match proof.proof.unwrap() {
                agglayer_interop::grpc::v1::aggchain_proof::Proof::Sp1Stark(proof) => proof,
            }
        }
        _ => panic!("expected generic aggchain proof"),
    };

    assert_eq!(sp1.version, "v4.0.0-rc.3");
    assert_eq!(sp1.proof, vec![1, 2, 3, 4]);
    assert_eq!(sp1.vkey, vec![5, 6, 7, 8]);
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
