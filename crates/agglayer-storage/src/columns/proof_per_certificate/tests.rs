use insta::assert_snapshot;

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

/// Guard against accidental changes in the storage encoding of [`Proof`].
///
/// The previous fixture was serialised with SP1 v3 types which are binary-
/// incompatible with SP1 v6 (`PlonkBn254Proof.public_inputs` grew from
/// `[String; 2]` to `[String; 5]`).  Instead of maintaining a cross-version
/// fixture shim, we now generate a real Plonk proof via the mock prover and
/// snapshot the storage-layer encoding.  Any future encoding drift will
/// cause the snapshot to fail.
#[test]
fn non_regression_proof_encoding() {
    use agglayer_types::{
        aggchain_data::CertificateAggchainDataCtx, L1WitnessCtx, PessimisticRootInput,
    };
    use pessimistic_proof::core::commitment::{
        PessimisticRootCommitmentVersion, SignatureCommitmentVersion,
    };
    use pessimistic_proof_test_suite::forest::Forest;

    // Build a real Plonk proof from an empty-state execution so the
    // snapshot exercises all codec fields (public_inputs, encoded_proof,
    // public_values, sp1_version, …).
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

    // Verify the agglayer storage codec round-trips correctly.
    let encoded = proof.encode().expect("Unable to encode proof");
    let decoded = Proof::decode(&encoded[..]).expect("Unable to decode proof");

    // Snapshot the storage-layer encoding (agglayer bincode codec).
    assert_snapshot!("proof hex format", hex::encode(&encoded));

    let Proof::SP1(sp1_proof) = decoded;
    assert_snapshot!(
        "proof public input hex format",
        hex::encode(sp1_proof.public_values.as_slice())
    );
}
