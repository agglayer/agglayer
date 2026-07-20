pub const EMPTY_ELF: &[u8] = include_bytes!("../tests/empty.elf");
pub const EMPTY_ELF_V5: &[u8] = include_bytes!("../tests/empty_v5.elf");

/// Serialize a proof and verifying key into an [`SP1StarkWithContext`]
/// envelope.
///
/// Unlike [`crate::v6_sp1_stark_with_context`], this helper does not enforce a
/// v6 writable version, so it can serve the legacy v5 mock path.
///
/// [`SP1StarkWithContext`]: agglayer_interop_types::aggchain_proof::SP1StarkWithContext
fn sp1_stark_with_context<P: serde::Serialize, V: serde::Serialize>(
    proof: &P,
    vkey: &V,
    version: &str,
) -> agglayer_interop_types::aggchain_proof::SP1StarkWithContext {
    agglayer_interop_types::aggchain_proof::SP1StarkWithContext {
        proof: agglayer_interop_types::bincode::default()
            .serialize(proof)
            .unwrap(),
        vkey: agglayer_interop_types::bincode::default()
            .serialize(vkey)
            .unwrap(),
        version: version.to_owned(),
    }
}

/// Create a dummy STARK proof for testing purposes with a specific SP1 version.
pub fn dummy_sp1_stark_proof_with_version(
    version: &str,
) -> agglayer_interop_types::aggchain_proof::Proof {
    match crate::version_kind(version) {
        Ok(crate::Sp1ProofVersion::V5) => {
            use sp1_sdk_v5::Prover;

            // The legacy v5 mock prover spins up its own tokio runtime during
            // construction, so build it on a fresh OS thread to avoid nesting
            // runtimes inside async tests.
            std::thread::spawn({
                let version = version.to_owned();
                move || {
                    let client = sp1_sdk_v5::ProverClient::builder().mock().build();
                    let (proving_key, vkey) = client.setup(EMPTY_ELF_V5);
                    let dummy_proof = sp1_sdk_v5::SP1ProofWithPublicValues::create_mock_proof(
                        &proving_key,
                        sp1_sdk_v5::SP1PublicValues::new(),
                        sp1_sdk_v5::SP1ProofMode::Compressed,
                        sp1_sdk_v5::SP1_CIRCUIT_VERSION,
                    );
                    let proof = dummy_proof.proof.try_as_compressed().unwrap();

                    agglayer_interop_types::aggchain_proof::Proof::SP1Stark(sp1_stark_with_context(
                        proof.as_ref(),
                        &vkey,
                        &version,
                    ))
                }
            })
            .join()
            .expect("legacy v5 mock-proof thread should not panic")
        }
        _ => {
            use sp1_sdk::{blocking::Prover, ProvingKey};

            use crate::{v6_sp1_stark_with_context, V6Sp1StarkProof};

            let (proof, vkey) = {
                let client = sp1_sdk::blocking::ProverClient::builder().mock().build();
                let proving_key = client.setup(sp1_sdk::Elf::Static(EMPTY_ELF)).unwrap();
                let verif_key = proving_key.verifying_key().clone();
                let dummy_proof = sp1_sdk::SP1ProofWithPublicValues::create_mock_proof(
                    proving_key.verifying_key(),
                    sp1_sdk::SP1PublicValues::new(),
                    sp1_sdk::SP1ProofMode::Compressed,
                    sp1_sdk::SP1_CIRCUIT_VERSION,
                );
                let proof: Box<V6Sp1StarkProof> = dummy_proof.proof.try_as_compressed().unwrap();
                (proof, verif_key)
            };

            let mut sp1 = v6_sp1_stark_with_context(proof.as_ref(), &vkey, "v6.1.0").unwrap();
            sp1.version = version.to_owned();
            agglayer_interop_types::aggchain_proof::Proof::SP1Stark(sp1)
        }
    }
}
