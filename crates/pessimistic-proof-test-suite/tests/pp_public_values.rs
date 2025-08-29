use pessimistic_proof::core::{AggchainHashValues, MultiSignature};
use pessimistic_proof_test_suite::{
    event_data::load_json_data_file,
    test_vector::{AggchainInputs, Inputs, MultisigInputs, TestFile, TestVector},
};
use rstest::rstest;

#[rstest]
#[case::aggchain_hash("test_vector/aggchain_hash_computation.json")]
#[case::multisig_hash("test_vector/multisig_hash_computation.json")]
fn test(#[case] json_file_name: &str) {
    let TestFile {
        test_vectors: entries,
        ..
    } = load_json_data_file(json_file_name);

    for entry in entries {
        let TestVector {
            inputs,
            expected_output,
        } = entry;

        let computed = match inputs {
            Inputs::Multisig(MultisigInputs { threshold, signers }) => MultiSignature {
                signatures: vec![], // not needed
                expected_signers: signers,
                threshold: threshold as usize,
            }
            .multisig_hash(),
            Inputs::Aggchain(AggchainInputs {
                consensus_type: _, // not needed
                aggchain_vkey,
                aggchain_params,
                multisig_hash,
            }) => AggchainHashValues::ConsensusType1 {
                aggchain_vkey: Some(u8x32_to_u32x8_be(*aggchain_vkey)),
                aggchain_params: Some(aggchain_params.into()),
                multisig_hash: multisig_hash.into(),
            }
            .hash(),
        };

        let expect = expected_output.as_hash();

        assert_eq!(*expect, *computed);
    }
}

// Convert [u8;32] to [u32;8] in big endian
fn u8x32_to_u32x8_be(b: [u8; 32]) -> [u32; 8] {
    core::array::from_fn(|i| {
        u32::from_be_bytes([b[i * 4], b[i * 4 + 1], b[i * 4 + 2], b[i * 4 + 3]])
    })
}
