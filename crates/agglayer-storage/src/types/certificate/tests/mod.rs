use agglayer_types::{
    aggchain_proof::{Proof, SP1StarkWithContext},
    U256,
};
use pessimistic_proof_test_suite::sample_data;
use sp1_sdk::Prover;

use super::*;
use crate::columns::Codec;

#[rstest::rstest]
#[case(0.into(), [0, 0, 0, 0])]
#[case(100.into(), [0, 0, 0, 100])]
#[case(258.into(), [0, 0, 1, 2])]
#[case(0x12345678.into(), [0x12, 0x34, 0x56, 0x78])]
fn network_id_encoding(#[case] network_id: NetworkId, #[case] expected: [u8; 4]) {
    let encoded = default_bincode_options().serialize(&network_id).unwrap();
    assert_eq!(encoded, expected);
    assert_eq!(network_id.to_u32().to_be_bytes(), expected);
}

fn load_sample_certificate_bytes(filename: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/types/certificate/tests")
        .join(filename);
    hex::decode(std::fs::read(path).unwrap().trim_ascii()).unwrap()
}

impl From<NetworkId> for NetworkIdV0 {
    fn from(value: NetworkId) -> Self {
        let [b3, b2, b1, b0] = value.to_u32().to_be_bytes();
        assert_eq!(b3, 0);
        NetworkIdV0([b2, b1, b0])
    }
}

impl CertificateV0 {
    fn test0() -> Self {
        Self {
            version: VersionTag,
            network_id: NetworkId::new(55).into(),
            height: 987,
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

    fn with_network_id(mut self, network_id: NetworkId) -> Self {
        self.network_id = network_id.into();
        self
    }
}

impl CertificateV1<'static> {
    fn test0() -> Self {
        Self {
            version: VersionTag,
            network_id: NetworkId::new(57),
            height: 987,
            prev_local_exit_root: Digest([0x02; 32]),
            new_local_exit_root: Digest([0x65; 32]),
            bridge_exits: Vec::new().into(),
            imported_bridge_exits: Vec::new().into(),
            aggchain_data: AggchainDataV1::ECDSA {
                signature: Signature::new(
                    U256::from_be_bytes([0x7a; 32]),
                    U256::from_be_bytes([0x9b; 32]),
                    false,
                ),
            },
            metadata: Digest([0xa9; 32]),
            custom_chain_data: Cow::Owned(vec![]),
            l1_info_root: None,
        }
    }

    fn test1() -> Self {
        let (proof, vkey) = {
            let client = sp1_sdk::ProverClient::builder().mock().build();
            let (proving_key, verif_key) = client.setup(pessimistic_proof::ELF);
            let dummy_proof = sp1_sdk::SP1ProofWithPublicValues::create_mock_proof(
                &proving_key,
                sp1_sdk::SP1PublicValues::new(),
                sp1_sdk::SP1ProofMode::Compressed,
                sp1_sdk::SP1_CIRCUIT_VERSION,
            );
            let proof = dummy_proof.proof.try_as_compressed().unwrap();
            (proof, verif_key)
        };

        let proof = Proof::SP1Stark(SP1StarkWithContext {
            proof,
            vkey,
            version: String::from("1.2.3"),
        });

        Self {
            version: VersionTag,
            network_id: NetworkId::new(59),
            height: 987.try_into().unwrap(),
            prev_local_exit_root: Digest([0x03; 32]),
            new_local_exit_root: Digest([0x61; 32]),
            bridge_exits: Vec::new().into(),
            imported_bridge_exits: Vec::new().into(),
            aggchain_data: AggchainDataV1::Generic {
                proof: Cow::Owned(proof),
                aggchain_params: Digest([0x58; 32]),
            },
            metadata: Digest([0xb9; 32]),
            custom_chain_data: Cow::Owned(vec![]),
            l1_info_root: None,
        }
    }
}

impl CertificateV1<'_> {
    fn into_owned(self) -> CertificateV1<'static> {
        let Self {
            version,
            network_id,
            height,
            prev_local_exit_root,
            new_local_exit_root,
            bridge_exits,
            imported_bridge_exits,
            aggchain_data,
            metadata,
            custom_chain_data,
            l1_info_root,
        } = self;

        CertificateV1 {
            version,
            network_id,
            height,
            prev_local_exit_root,
            new_local_exit_root,
            bridge_exits: bridge_exits.into_owned().into(),
            imported_bridge_exits: imported_bridge_exits.into_owned().into(),
            aggchain_data: match aggchain_data {
                AggchainDataV1::ECDSA { signature } => AggchainDataV1::ECDSA { signature },
                AggchainDataV1::Generic {
                    proof,
                    aggchain_params,
                } => AggchainDataV1::Generic {
                    proof: Cow::Owned(proof.into_owned()),
                    aggchain_params,
                },
            },
            metadata,
            custom_chain_data: Cow::Owned(custom_chain_data.into_owned()),
            l1_info_root,
        }
    }
}

#[rstest::rstest]
#[case(CertificateV0::test0(), &[0x00, 0x00, 0x00, 55])]
#[case(CertificateV0::test0().with_network_id(0x123456.into()), &[0x00, 0x12, 0x34, 0x56])]
#[case(CertificateV1::test0(), &[0x01, 0x00, 0x00, 0x00, 57])]
fn encoding_starts_with(#[case] cert: impl Serialize, #[case] start: &[u8]) {
    let bytes = default_bincode_options().serialize(&cert).unwrap();
    assert!(bytes.starts_with(start));
}

#[rstest::rstest]
#[case(CertificateV0::test0())]
#[case(CertificateV1::test0())]
#[case(CertificateV1::test1())]
#[case(CertificateV1::from(&Certificate::new_for_test(74.into(), 998)).into_owned())]
fn encoding_roundtrip_consistent_with_into(#[case] orig: impl Into<Certificate> + Serialize) {
    let bytes = default_bincode_options().serialize(&orig).unwrap();
    let decoded = Certificate::decode(&bytes).unwrap();
    let converted: Certificate = orig.into();

    // This should really compare the certificates directly but that requires adding
    // whole bunch of `Eq` impl to many types.
    assert_eq!(format!("{converted:?}"), format!("{decoded:?}"));
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

#[test]
fn bad_format() {
    const NEXT_VERSION: u8 = 2;

    assert!(matches!(
        Certificate::decode(&[]).unwrap_err(),
        CodecError::CertificateEmpty
    ));

    for v in 0..NEXT_VERSION {
        assert!(matches!(
            Certificate::decode(&[v]).unwrap_err(),
            CodecError::Serialization(_)
        ));
    }

    for v in NEXT_VERSION..=u8::MAX {
        match Certificate::decode(&[v]).unwrap_err() {
            CodecError::BadCertificateVersion { version } => assert_eq!(version, v),
            err => panic!("Unexpected error: {err:?}"),
        }
    }
}
