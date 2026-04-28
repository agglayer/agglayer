use agglayer_interop::grpc::v1 as interop_v1;
use agglayer_types::{
    aggchain_proof::{AggchainData, AggchainProof, MultisigPayload},
    bincode,
    primitives::SignatureError,
    testutils::dummy_sp1_stark_proof_with_version,
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

#[derive(Clone, Copy)]
enum ProofCarrier {
    Generic,
    MultisigAndProof,
}

fn certificate_proto_fixture(aggchain_data: AggchainData) -> v1::Certificate {
    let certificate = Certificate {
        aggchain_data,
        ..Certificate::default()
    };

    let mut certificate =
        v1::Certificate::try_from(certificate).expect("test certificate should serialize");
    certificate.metadata = None;
    certificate
}

fn default_certificate_signature() -> agglayer_types::primitives::Signature {
    match Certificate::default().aggchain_data {
        AggchainData::ECDSA { signature } => signature,
        other => panic!("expected ECDSA certificate, got {other:?}"),
    }
}

fn proof_certificate_proto(carrier: ProofCarrier, version: &str) -> v1::Certificate {
    let proof = AggchainProof {
        proof: dummy_sp1_stark_proof_with_version(version),
        aggchain_params: Digest::default(),
        public_values: None,
    };

    match carrier {
        ProofCarrier::Generic => {
            let certificate = Certificate::default();
            let proof = interop_v1::AggchainProof::try_from(proof)
                .expect("aggchain proof should serialize");

            let mut certificate =
                v1::Certificate::try_from(certificate).expect("test certificate should serialize");
            certificate.aggchain_data = Some(interop_v1::AggchainData {
                data: Some(interop_v1::aggchain_data::Data::Generic(
                    interop_v1::AggchainProof {
                        signature: None,
                        ..proof
                    },
                )),
            });
            certificate.metadata = None;
            certificate
        }
        ProofCarrier::MultisigAndProof => {
            let signature = default_certificate_signature();

            certificate_proto_fixture(AggchainData::MultisigAndAggchainProof {
                multisig: MultisigPayload(vec![Some(signature)]),
                aggchain_proof: proof,
            })
        }
    }
}

fn multisig_only_certificate_proto() -> v1::Certificate {
    let signature = default_certificate_signature();

    certificate_proto_fixture(AggchainData::MultisigOnly {
        multisig: MultisigPayload(vec![Some(signature)]),
    })
}

#[rstest::rstest]
#[case::generic(proof_certificate_proto(ProofCarrier::Generic, "v5.2.2"))]
#[case::multisig_and_proof(proof_certificate_proto(ProofCarrier::MultisigAndProof, "v5.2.2"))]
fn accepts_writable_proof_versions_in_certificate_ingress(#[case] certificate: v1::Certificate) {
    let result = Certificate::try_from(certificate);

    assert!(
        result.is_ok(),
        "writable proof version should be accepted: {result:?}"
    );
}

#[rstest::rstest]
#[case::generic(proof_certificate_proto(ProofCarrier::Generic, "v6.0.1"))]
#[case::multisig_and_proof(proof_certificate_proto(ProofCarrier::MultisigAndProof, "v6.0.1"))]
fn rejects_non_writable_proof_versions_in_certificate_ingress(
    #[case] certificate: v1::Certificate,
) {
    let result = Certificate::try_from(certificate);
    let error = result.expect_err("non-writable proof version should be rejected");

    assert_eq!(error.unsupported_proof_version(), Some("v6.0.1"));
}

#[test]
fn invalid_proof_versions_remain_invalid_certificate_errors() {
    let error = Certificate::try_from(proof_certificate_proto(ProofCarrier::Generic, "abc"))
        .expect_err("invalid proof version should be rejected");

    assert_eq!(error.unsupported_proof_version(), None);
    assert_eq!(
        error.to_string(),
        "aggchain_proof: invalid proof version `abc`"
    );
}

#[rstest::rstest]
#[case::ecdsa(v1::Certificate::try_from(Certificate::default()).expect("test certificate should serialize"))]
#[case::multisig_only(multisig_only_certificate_proto())]
fn non_proof_certificate_variants_are_unaffected(#[case] certificate: v1::Certificate) {
    let result = Certificate::try_from(certificate);

    assert!(
        result.is_ok(),
        "non-proof variants should remain accepted: {result:?}"
    );
}
