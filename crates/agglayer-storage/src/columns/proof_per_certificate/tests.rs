use crate::columns::{
    proof_per_certificate::{CertificateId, Proof},
    Codec as _,
};

#[test]
fn can_parse_key() {
    let key: CertificateId = [1; 32];

    let encoded = key.encode().expect("Unable to encode key");

    let expected_key = CertificateId::decode(&encoded[..]).expect("Unable to decode key");

    assert_eq!(expected_key, key);

    assert_eq!(
        encoded[..32],
        [
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1,
        ]
    );
}

#[test]
fn can_parse_value() {
    let value = Proof::new_for_test();

    let encoded = value.encode().expect("Unable to encode value");

    let expected_value = Proof::decode(&encoded[..]).expect("Unable to decode value");

    assert!(matches!(expected_value, Proof::SP1(_)));
}
