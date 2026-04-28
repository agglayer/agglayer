use agglayer_interop_types::aggchain_proof::{Proof, SP1StarkWithContext};
use agglayer_sp1::{AcceptancePolicy, ProofError, ProofExt, Sp1ProofVersion};
use sp1_sdk::Prover;

const EMPTY_ELF: &[u8] = include_bytes!("empty.elf");

fn mock_proof(version: &str) -> Proof {
    let client = sp1_sdk::ProverClient::builder().mock().build();
    let (proving_key, vkey) = client.setup(EMPTY_ELF);
    let proof = sp1_sdk::SP1ProofWithPublicValues::create_mock_proof(
        &proving_key,
        sp1_sdk::SP1PublicValues::new(),
        sp1_sdk::SP1ProofMode::Compressed,
        sp1_sdk::SP1_CIRCUIT_VERSION,
    )
    .proof
    .try_as_compressed()
    .unwrap();

    Proof::SP1Stark(SP1StarkWithContext {
        proof,
        vkey,
        version: version.to_owned(),
    })
}

#[test]
fn default_reads_v5_and_v6() {
    AcceptancePolicy::DEFAULT
        .ensure_readable(Sp1ProofVersion::V5, "v5.2.2")
        .unwrap();
    AcceptancePolicy::DEFAULT
        .ensure_readable(Sp1ProofVersion::V6, "v6.0.1")
        .unwrap();
}

#[test]
fn default_rejects_v6_writes() {
    let err = AcceptancePolicy::DEFAULT
        .ensure_writable(Sp1ProofVersion::V6, "v6.0.1")
        .unwrap_err();
    assert!(matches!(
        err,
        ProofError::UnsupportedWritableSp1Version { .. }
    ));
}

#[test]
fn default_rejects_v6_executes() {
    let err = AcceptancePolicy::DEFAULT
        .ensure_executable(Sp1ProofVersion::V6, "v6.0.1")
        .unwrap_err();
    assert!(matches!(
        err,
        ProofError::UnsupportedExecutableSp1Version { .. }
    ));
}

#[test]
fn proof_reports_write_specific_error_for_unknown_major() {
    let err = mock_proof("v7.0.0")
        .ensure_writable(&AcceptancePolicy::DEFAULT)
        .unwrap_err();

    assert!(matches!(
        err,
        ProofError::UnsupportedWritableSp1Version { .. }
    ));
}

#[test]
fn proof_reports_execute_specific_error_for_unknown_major() {
    let err = mock_proof("v7.0.0")
        .ensure_executable(&AcceptancePolicy::DEFAULT)
        .unwrap_err();

    assert!(matches!(
        err,
        ProofError::UnsupportedExecutableSp1Version { .. }
    ));
}

#[test]
fn proof_error_exposes_unsupported_version_for_write_rejections() {
    let err = mock_proof("v6.0.1")
        .ensure_writable(&AcceptancePolicy::DEFAULT)
        .unwrap_err();

    assert_eq!(err.unsupported_version(), Some("v6.0.1"));
    assert_eq!(err.invalid_version(), None);
}

#[test]
fn proof_error_exposes_invalid_version_for_unparsable_inputs() {
    let err = mock_proof("abc")
        .ensure_writable(&AcceptancePolicy::DEFAULT)
        .unwrap_err();

    assert_eq!(err.invalid_version(), Some("abc"));
    assert_eq!(err.unsupported_version(), None);
}

#[test]
fn proof_vkey_hash_helpers_are_infallible() {
    let proof = mock_proof("v5.2.2");

    let bytes = proof.vkey_hash_bytes();
    let words = proof.vkey_hash_u32();

    assert_eq!(bytes.len(), 32);
    assert_eq!(words.len(), 8);
}
