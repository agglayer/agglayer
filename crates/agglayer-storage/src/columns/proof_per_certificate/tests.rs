use insta::assert_snapshot;
use serde::{de::IgnoredAny, Deserialize};
use sp1_prover::{Groth16Bn254Proof, PlonkBn254Proof};
use sp1_sdk::{SP1Proof, SP1ProofWithPublicValues, SP1PublicValues};

use crate::{
    columns::proof_per_certificate::{CertificateId, Proof},
    schema::Codec as _,
};

#[test]
fn can_parse_key() {
    let key = CertificateId::new([1; 32].into());

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
struct LegacySP1ProofWithPublicValues {
    proof: LegacySP1Proof,
    #[allow(unused)]
    stdin: sp1_sdk::SP1Stdin,
    public_values: SP1PublicValues,
    sp1_version: String,
}

#[derive(Deserialize)]
enum LegacySP1Proof {
    Core(IgnoredAny),
    Compressed(IgnoredAny),
    Plonk(LegacyPlonkBn254Proof),
    Groth16(LegacyGroth16Bn254Proof),
}

#[derive(Deserialize)]
struct LegacyPlonkBn254Proof {
    public_inputs: [String; 2],
    encoded_proof: String,
    raw_proof: String,
    plonk_vkey_hash: [u8; 32],
}

#[derive(Deserialize)]
struct LegacyGroth16Bn254Proof {
    public_inputs: [String; 2],
    encoded_proof: String,
    raw_proof: String,
    groth16_vkey_hash: [u8; 32],
}

impl LegacySP1ProofWithPublicValues {
    fn load(path: impl AsRef<std::path::Path>) -> eyre::Result<Self> {
        agglayer_types::bincode::sp1v4()
            .deserialize_from(std::fs::File::open(path).expect("failed to open file"))
            .map_err(Into::into)
    }
}

fn expand_legacy_public_inputs([vkey_hash, public_values_hash]: [String; 2]) -> [String; 5] {
    [
        vkey_hash,
        public_values_hash,
        "0".to_string(),
        "0".to_string(),
        "0".to_string(),
    ]
}

impl From<LegacySP1Proof> for SP1Proof {
    fn from(legacy: LegacySP1Proof) -> Self {
        match legacy {
            LegacySP1Proof::Plonk(plonk) => SP1Proof::Plonk(PlonkBn254Proof {
                public_inputs: expand_legacy_public_inputs(plonk.public_inputs),
                encoded_proof: plonk.encoded_proof,
                raw_proof: plonk.raw_proof,
                plonk_vkey_hash: plonk.plonk_vkey_hash,
            }),
            LegacySP1Proof::Groth16(groth16) => SP1Proof::Groth16(Groth16Bn254Proof {
                public_inputs: expand_legacy_public_inputs(groth16.public_inputs),
                encoded_proof: groth16.encoded_proof,
                raw_proof: groth16.raw_proof,
                groth16_vkey_hash: groth16.groth16_vkey_hash,
            }),
            LegacySP1Proof::Core(_) | LegacySP1Proof::Compressed(_) => {
                panic!("historical proof fixture unexpectedly contains a non-BN254 proof")
            }
        }
    }
}

impl From<LegacySP1ProofWithPublicValues> for SP1ProofWithPublicValues {
    fn from(legacy: LegacySP1ProofWithPublicValues) -> Self {
        Self {
            proof: legacy.proof.into(),
            public_values: legacy.public_values,
            sp1_version: legacy.sp1_version,
            tee_proof: None,
        }
    }
}

/// Guard against accidental changes in the on-chain proof bytes and public
/// values layout for already persisted proofs.
#[test]
fn non_regression_proof_encoding() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("./src/columns/proof_per_certificate/fixtures/non_regression_proof_encoding.proof");

    let proof: SP1ProofWithPublicValues =
        LegacySP1ProofWithPublicValues::load(path).unwrap().into();
    assert_snapshot!("proof hex format", hex::encode(proof.bytes()));

    assert_snapshot!(
        "proof public input hex format",
        hex::encode(proof.public_values.as_slice())
    );
}

/// Snapshot the current v6 storage encoding separately so the legacy fixture
/// remains a backward-compatibility check.
#[test]
fn non_regression_proof_encoding_v6() {
    use agglayer_types::{
        aggchain_data::CertificateAggchainDataCtx, L1WitnessCtx, PessimisticRootInput,
    };
    use pessimistic_proof::core::commitment::{
        PessimisticRootCommitmentVersion, SignatureCommitmentVersion,
    };
    use pessimistic_proof_test_suite::forest::Forest;

    let mut state = Forest::new([]);
    let old_state = state.local_state();
    let certificate =
        state
            .clone()
            .apply_bridge_exits([], std::iter::empty(), SignatureCommitmentVersion::V2);
    let multi_batch_header = state
        .state_b
        .apply_certificate(
            &certificate,
            L1WitnessCtx {
                l1_info_root: certificate.l1_info_root().unwrap().unwrap_or_default(),
                prev_pessimistic_root: PessimisticRootInput::Computed(
                    PessimisticRootCommitmentVersion::V2,
                ),
                aggchain_data_ctx: CertificateAggchainDataCtx::LegacyEcdsa {
                    signer: state.get_signer(),
                },
            },
        )
        .unwrap();
    let proof = Proof::new_for_test(&old_state.into(), &multi_batch_header);

    let encoded = proof.encode().expect("Unable to encode proof");
    let decoded = Proof::decode(&encoded[..]).expect("Unable to decode proof");

    assert_snapshot!("proof storage hex format v6", hex::encode(&encoded));

    let Proof::SP1(sp1_proof) = decoded;
    assert_snapshot!(
        "proof public input hex format v6",
        hex::encode(sp1_proof.public_values.as_slice())
    );
}
