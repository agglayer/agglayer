use agglayer_types::U256;
use pessimistic_proof::aggchain_proof::StarkProof;
use pessimistic_proof_test_suite::sample_data;
use sp1_sdk::Prover;

use super::*;
use crate::columns::Codec;

#[test]
fn height_same_size_as_u64() {
    // Just a sanity check to see if the encoded types overlap properly.
    assert_eq!(u64::BITS, Height::BITS);
}

fn load_sample_certificate_bytes(filename: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/types/certificate/tests")
        .join(filename);
    hex::decode(std::fs::read(path).unwrap().trim_ascii()).unwrap()
}

impl CertificateV0 {
    fn test0() -> Self {
        Self {
            network_id: NetworkId::new(55),
            height: 987.try_into().unwrap(),
            prev_local_exit_root: Digest([0x01; 32]),
            new_local_exit_root: Digest([0x67; 32]),
            bridge_exits: Vec::new(),
            imported_bridge_exits: Vec::new(),
            signature: Signature::new(
                U256::from_be_bytes([0x78; 32]),
                U256::from_be_bytes([0x9a; 32]),
                false,
            ),
            metadata: Digest([0xa5; 32]),
        }
    }
}

impl Codec for CertificateV0 {}

impl CertificateV1 {
    fn test0() -> Self {
        Self {
            network_id: NetworkId::new(57),
            version: Default::default(),
            height: 987.try_into().unwrap(),
            prev_local_exit_root: Digest([0x02; 32]),
            new_local_exit_root: Digest([0x65; 32]),
            bridge_exits: Vec::new(),
            imported_bridge_exits: Vec::new(),
            aggchain_proof: AggchainProofV1::ECDSA {
                signature: Signature::new(
                    U256::from_be_bytes([0x7a; 32]),
                    U256::from_be_bytes([0x9b; 32]),
                    false,
                ),
            },
            metadata: Digest([0xa9; 32]),
        }
    }

    fn test1() -> Self {
        let stark_proof = {
            let client = sp1_sdk::ProverClient::builder().mock().build();
            let (proving_key, _verif_key) = client.setup(pessimistic_proof::ELF);
            let dummy_proof = sp1_sdk::SP1ProofWithPublicValues::create_mock_proof(
                &proving_key,
                sp1_sdk::SP1PublicValues::new(),
                sp1_sdk::SP1ProofMode::Compressed,
                sp1_sdk::SP1_CIRCUIT_VERSION,
            );
            *dummy_proof.proof.try_as_compressed().unwrap()
        };

        Self {
            network_id: NetworkId::new(59),
            version: Default::default(),
            height: 987.try_into().unwrap(),
            prev_local_exit_root: Digest([0x03; 32]),
            new_local_exit_root: Digest([0x61; 32]),
            bridge_exits: Vec::new(),
            imported_bridge_exits: Vec::new(),
            aggchain_proof: AggchainProofV1::SP1 {
                aggchain_proof: AggchainProofSP1 {
                    aggchain_params: Digest([0x58; 32]),
                    stark_proof,
                },
            },
            metadata: Digest([0xb9; 32]),
        }
    }
}

impl Codec for CertificateV1 {}

#[rstest::rstest]
#[case(CertificateV0::test0())]
#[case(CertificateV1::test0())]
#[case(CertificateV1::test1())]
#[case(Certificate::new_for_test(74.into(), 998))]
fn roundtrip_through_versioned(#[case] certificate: impl Codec + Into<Certificate>) {
    let bytes = certificate.encode().unwrap();
    let decoded = Certificate::decode(&bytes).unwrap();
    let orig: Certificate = certificate.into();

    // This should really compare the certificates directly but that requires adding
    // whole bunch of `Eq` impl to many types.
    assert_eq!(format!("{orig:?}"), format!("{decoded:?}"));
}

#[rstest::rstest]
#[case("n15-cert_h0")]
#[case("n15-cert_h1")]
#[case("n15-cert_h2")]
#[case("n15-cert_h3")]
fn cert_in_v0_format_decodes(#[case] cert_name: &str) {
    let from_json = sample_data::load_certificate(&format!("{cert_name}.json"));

    let bytes = load_sample_certificate_bytes(&format!("encoded_v0-{cert_name}.hex"));
    let from_bytes = Certificate::decode(&bytes).expect("v0 certificate to decode successfully");

    // Again comparing debug output due to lack of `Eq`.
    assert_eq!(format!("{from_bytes:?}"), format!("{from_json:?}"));
}
