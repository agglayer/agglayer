use agglayer_types::{bincode, primitives::SignatureError, CertificateId, EpochConfiguration};
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
