use agglayer_types::{Digest, LocalNetworkStateData};
use insta::assert_snapshot;
use pessimistic_proof::{keccak::keccak256_combine, LocalNetworkState, PessimisticProofOutput};
use pessimistic_proof_test_suite::{forest::Forest, sample_data::USDC, PESSIMISTIC_PROOF_ELF};
use sp1_sdk::{
    provers::ProofOpts, MockProver, Prover, SP1Context, SP1ProofWithPublicValues, SP1Stdin,
};

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

#[test]
fn non_regression_proof_encoding() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("./src/columns/proof_per_certificate/fixtures/non_regression_proof_encoding.proof");

    let proof = SP1ProofWithPublicValues::load(path).unwrap();

    assert_snapshot!("proof hex format", hex::encode(proof.bytes()));

    assert_snapshot!(
        "proof public input hex format",
        hex::encode(proof.public_values.as_slice())
    );
}
