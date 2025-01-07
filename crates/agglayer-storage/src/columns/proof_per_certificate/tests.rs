use insta::assert_snapshot;
use serde::Deserialize;
use sp1_sdk::SP1ProofWithPublicValues;

use crate::columns::{
    proof_per_certificate::{CertificateId, Proof},
    Codec as _,
};

#[test]
fn can_parse_key() {
    let key: CertificateId = [1; 32].into();

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
    let value = Proof::dummy();

    let encoded = value.encode().expect("Unable to encode value");

    let expected_value = Proof::decode(&encoded[..]).expect("Unable to decode value");

    assert!(matches!(expected_value, Proof::SP1(_)));
}

#[derive(Deserialize)]
struct SP1ProofWithPublicValuesV3 {
    pub proof: sp1_sdk::SP1Proof,
    #[allow(unused)]
    pub stdin: sp1_sdk::SP1Stdin,
    pub public_values: sp1_sdk::SP1PublicValues,
    pub sp1_version: String,
}

impl SP1ProofWithPublicValuesV3 {
    fn load(path: impl AsRef<std::path::Path>) -> Result<Self, Box<dyn std::error::Error>> {
        bincode::deserialize_from(std::fs::File::open(path).expect("failed to open file"))
            .map_err(Into::into)
    }
}

impl From<SP1ProofWithPublicValuesV3> for SP1ProofWithPublicValues {
    fn from(v3: SP1ProofWithPublicValuesV3) -> Self {
        Self {
            proof: v3.proof,
            public_values: v3.public_values,
            sp1_version: v3.sp1_version,
        }
    }
}

#[test]
fn non_regression_proof_encoding() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("./src/columns/proof_per_certificate/fixtures/non_regression_proof_encoding.proof");

    let proof: SP1ProofWithPublicValues = SP1ProofWithPublicValuesV3::load(path).unwrap().into();
    assert_snapshot!("proof hex format", hex::encode(proof.bytes()));

    assert_snapshot!(
        "proof public input hex format",
        hex::encode(proof.public_values.as_slice())
    );
}
