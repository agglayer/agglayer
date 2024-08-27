use sp1_sdk::{SP1ProofWithPublicValues, SP1PublicValues};

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
    let mut stdin = sp1_sdk::SP1Stdin::new();
    stdin.write_slice(&[2; 32]);

    let value = Proof::SP1(SP1ProofWithPublicValues {
        proof: sp1_sdk::proof::SP1Proof::Core(Vec::new()),
        stdin,
        public_values: SP1PublicValues::new(),
        sp1_version: String::new(),
    });
    let encoded = value.encode().expect("Unable to encode value");

    let expected_value = Proof::decode(&encoded[..]).expect("Unable to decode value");

    assert!(matches!(expected_value, Proof::SP1(_)));
}
