use agglayer_interop_types::aggchain_proof::{Proof, SP1StarkWithContext};
use agglayer_sp1::{
    current_sp1_stark_with_context, AcceptancePolicy, ProofError, ProofExt, Sp1ProofVersion,
};
use serde::{ser::Error as _, Serialize, Serializer};
use sp1_sdk::{
    blocking::{Prover, ProverClient},
    ProvingKey,
};

const EMPTY_ELF: &[u8] = include_bytes!("empty.elf");
const EMPTY_ELF_V5: &[u8] = include_bytes!("empty_v5.elf");

struct FailingSerializeProof;

impl Serialize for FailingSerializeProof {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Err(S::Error::custom("proof serialization failed"))
    }
}

fn mock_proof(version: &str) -> Proof {
    match agglayer_sp1::version_kind(version) {
        Ok(Sp1ProofVersion::V5) => {
            use sp1_sdk_v5::Prover;

            let client = sp1_sdk_v5::ProverClient::builder().mock().build();
            let (proving_key, vkey) = client.setup(EMPTY_ELF_V5);
            let proof = sp1_sdk_v5::SP1ProofWithPublicValues::create_mock_proof(
                &proving_key,
                sp1_sdk_v5::SP1PublicValues::new(),
                sp1_sdk_v5::SP1ProofMode::Compressed,
                sp1_sdk_v5::SP1_CIRCUIT_VERSION,
            )
            .proof
            .try_as_compressed()
            .unwrap();

            Proof::SP1Stark(SP1StarkWithContext {
                proof: agglayer_interop_types::bincode::default()
                    .serialize(proof.as_ref())
                    .unwrap(),
                vkey: agglayer_interop_types::bincode::default()
                    .serialize(&vkey)
                    .unwrap(),
                version: version.to_owned(),
            })
        }
        _ => {
            let client = ProverClient::builder().mock().build();
            let proving_key = client.setup(sp1_sdk::Elf::Static(EMPTY_ELF)).unwrap();
            let vkey = proving_key.verifying_key().clone();
            let proof = sp1_sdk::SP1ProofWithPublicValues::create_mock_proof(
                proving_key.verifying_key(),
                sp1_sdk::SP1PublicValues::new(),
                sp1_sdk::SP1ProofMode::Compressed,
                sp1_sdk::SP1_CIRCUIT_VERSION,
            )
            .proof
            .try_as_compressed()
            .unwrap();

            Proof::SP1Stark(current_sp1_stark_with_context(proof.as_ref(), &vkey, version).unwrap())
        }
    }
}

#[test]
fn default_reads_v5_and_v6() {
    AcceptancePolicy::DEFAULT
        .ensure_readable(Sp1ProofVersion::V5, "v5.2.2")
        .unwrap();
    AcceptancePolicy::DEFAULT
        .ensure_readable(Sp1ProofVersion::V6, "v6.1.0")
        .unwrap();
}

#[test]
fn default_writes_and_executes_v6() {
    AcceptancePolicy::DEFAULT
        .ensure_writable(Sp1ProofVersion::V6, "v6.1.0")
        .unwrap();
    AcceptancePolicy::DEFAULT
        .ensure_executable(Sp1ProofVersion::V6, "v6.1.0")
        .unwrap();
}

#[test]
fn default_rejects_v5_writes_and_executes() {
    let write_err = AcceptancePolicy::DEFAULT
        .ensure_writable(Sp1ProofVersion::V5, "v5.2.2")
        .unwrap_err();
    assert!(matches!(
        write_err,
        ProofError::UnsupportedWritableSp1Version { .. }
    ));

    let execute_err = AcceptancePolicy::DEFAULT
        .ensure_executable(Sp1ProofVersion::V5, "v5.2.2")
        .unwrap_err();
    assert!(matches!(
        execute_err,
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
fn executable_sp1_uses_explicit_acceptance_policy() {
    let err = mock_proof("v5.2.2")
        .executable_sp1(&AcceptancePolicy::DEFAULT)
        .unwrap_err();

    assert!(matches!(
        err,
        ProofError::UnsupportedExecutableSp1Version { .. }
    ));
}

#[test]
fn proof_error_exposes_unsupported_version_for_write_rejections() {
    let err = mock_proof("v5.2.2")
        .ensure_writable(&AcceptancePolicy::DEFAULT)
        .unwrap_err();

    assert_eq!(err.unsupported_version(), Some("v5.2.2"));
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

    let bytes = proof.vkey_hash_bytes().unwrap();
    let words = proof.vkey_hash_u32().unwrap();

    assert_eq!(bytes.len(), 32);
    assert_eq!(words.len(), 8);
}

#[test]
fn proof_vkey_hash_helpers_report_deserialization_errors() {
    let proof = Proof::SP1Stark(SP1StarkWithContext {
        proof: Vec::new(),
        vkey: vec![0xde, 0xad, 0xbe, 0xef],
        version: "v6.1.0".to_owned(),
    });

    let err = proof.vkey_hash_bytes().unwrap_err();
    assert!(matches!(err, ProofError::DeserializeSp1Vkey { .. }));
}

#[test]
fn current_sp1_stark_with_context_reports_proof_serialization_errors() {
    let client = ProverClient::builder().mock().build();
    let proving_key = client.setup(sp1_sdk::Elf::Static(EMPTY_ELF)).unwrap();
    let vkey = proving_key.verifying_key();

    let err = current_sp1_stark_with_context(&FailingSerializeProof, vkey, "v6.1.0").unwrap_err();

    assert!(matches!(err, ProofError::SerializeSp1Proof { .. }));
}
