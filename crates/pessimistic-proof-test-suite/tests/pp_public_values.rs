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
                multisig_hash: Some(multisig_hash.into()),
            }
            .hash(),
        };

        let expect = expected_output.as_hash();
        // let prefix = if *expect == *computed { "✅" } else { "❌" };

        // println!("{prefix} expected hash: {:?}", expect);
        // println!("{prefix} got hash: {:?}", computed);

        // println!("");
        assert_eq!(*expect, *computed);
    }
}

// Big-endian: [u8;32] -> [u32;8], groups b[0..4] as the first u32, etc.
fn u8x32_to_u32x8_be(b: [u8; 32]) -> [u32; 8] {
    core::array::from_fn(|i| {
        u32::from_be_bytes([b[i * 4], b[i * 4 + 1], b[i * 4 + 2], b[i * 4 + 3]])
    })
}
