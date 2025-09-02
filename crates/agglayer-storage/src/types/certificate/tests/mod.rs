use super::*;
use crate::columns::Codec;
use agglayer_types::{
    aggchain_proof::{Proof, SP1StarkWithContext},
    bincode, U256,
};
use alloy_primitives::Bytes;
use pessimistic_proof_test_suite::sample_data;
use sp1_sdk::Prover;

mod header;
mod status;
mod structure;

const EMPTY_ELF: &[u8] = include_bytes!("empty.elf");

#[rstest::rstest]
#[case(0.into(), [0, 0, 0, 0])]
#[case(100.into(), [0, 0, 0, 100])]
#[case(258.into(), [0, 0, 1, 2])]
#[case(0x12345678.into(), [0x12, 0x34, 0x56, 0x78])]
fn network_id_encoding(#[case] network_id: NetworkId, #[case] expected: [u8; 4]) {
    let encoded = bincode_codec().serialize(&network_id).unwrap();
    assert_eq!(encoded, expected);
    assert_eq!(network_id.to_u32().to_be_bytes(), expected);
}

fn load_sample_bytes(filename: &str) -> Vec<u8> {
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

fn sig(r_byte: u8, s_byte: u8) -> Signature {
    let r = U256::from_be_bytes([r_byte; 32]);
    let s = U256::from_be_bytes([s_byte; 32]);
    Signature::new(r, s, false)
}

impl AggchainDataV1<'static> {
    fn proof0() -> Proof {
        let (proof, vkey) = {
            let client = sp1_sdk::ProverClient::builder().mock().build();
            let (proving_key, verif_key) = client.setup(EMPTY_ELF);
            let dummy_proof = sp1_sdk::SP1ProofWithPublicValues::create_mock_proof(
                &proving_key,
                sp1_sdk::SP1PublicValues::new(),
                sp1_sdk::SP1ProofMode::Compressed,
                sp1_sdk::SP1_CIRCUIT_VERSION,
            );
            let proof = dummy_proof.proof.try_as_compressed().unwrap();
            (proof, verif_key)
        };

        Proof::SP1Stark(SP1StarkWithContext {
            proof,
            vkey,
            version: String::from("1.2.3"),
        })
    }

    fn aggchain_proof_public_values0(aggchain_params: Digest) -> AggchainProofPublicValues {
        AggchainProofPublicValues {
            prev_local_exit_root: Default::default(),
            new_local_exit_root: Default::default(),
            l1_info_root: Default::default(),
            origin_network: NetworkId::new(0u32),
            commit_imported_bridge_exits: Default::default(),
            aggchain_params,
        }
    }

    fn sig0() -> Signature {
        sig(0x7a, 0x9b)
    }

    fn test0() -> Self {
        let signature = Self::sig0();
        Self::ECDSA { signature }
    }

    fn test1() -> Self {
        AggchainDataV1::GenericWithSignature {
            proof: Cow::Owned(Self::proof0()),
            aggchain_params: Digest([0x58; 32]),
            signature: Cow::Owned(Box::new(sig(0x78, 0x9a))),
        }
    }

    fn test2() -> Self {
        AggchainDataV1::GenericNoSignature {
            proof: Cow::Owned(Self::proof0()),
            aggchain_params: Digest([0x59; 32]),
        }
    }

    fn test3() -> Self {
        let aggchain_params = Digest([0x60; 32]);
        AggchainDataV1::GenericWithPublicValues {
            proof: Cow::Owned(Self::proof0()),
            aggchain_params,
            signature: None,
            public_values: Cow::Owned(Box::new(Self::aggchain_proof_public_values0(
                aggchain_params,
            ))),
        }
    }

    fn test4() -> Self {
        AggchainDataV1::MultisigOnly {
            multisig: Cow::Owned(vec![
                None,
                None,
                Some(sig(0x11, 0x22)),
                None,
                Some(sig(0x33, 0x44)),
            ]),
        }
    }

    fn test5() -> Self {
        let aggchain_params = Digest([0x61; 32]);
        AggchainDataV1::MultisigAndAggchainProof {
            multisig: Cow::Owned(vec![
                Some(sig(0x55, 0x66)),
                None,
                Some(sig(0x77, 0x88)),
                None,
                None,
            ]),
            proof: Cow::Owned(Self::proof0()),
            aggchain_params,
            public_values: Some(Cow::Owned(Box::new(Self::aggchain_proof_public_values0(
                aggchain_params,
            )))),
        }
    }
}

impl CertificateV0 {
    fn test0() -> Self {
        Self {
            version: VersionTag,
            network_id: NetworkId::new(55).into(),
            height: Height::new(987),
            prev_local_exit_root: LocalExitRoot::from([0x01; 32]),
            new_local_exit_root: LocalExitRoot::from([0x67; 32]),
            bridge_exits: Vec::new(),
            imported_bridge_exits: Vec::new(),
            signature: sig(0x78, 0x9a),
            metadata: Metadata::new(Digest([0xa5; 32])),
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
            height: Height::new(987),
            prev_local_exit_root: LocalExitRoot::from([0x02; 32]),
            new_local_exit_root: LocalExitRoot::from([0x65; 32]),
            bridge_exits: Vec::new().into(),
            imported_bridge_exits: Vec::new().into(),
            aggchain_data: AggchainDataV1::test0(),
            metadata: Metadata::new(Digest([0xa9; 32])),
            custom_chain_data: Cow::Owned(vec![]),
            l1_info_tree_leaf_count: None,
        }
    }

    fn test1() -> Self {
        Self {
            version: VersionTag,
            network_id: NetworkId::new(59),
            height: Height::new(987),
            prev_local_exit_root: LocalExitRoot::from([0x03; 32]),
            new_local_exit_root: LocalExitRoot::from([0x61; 32]),
            bridge_exits: Vec::new().into(),
            imported_bridge_exits: Vec::new().into(),
            aggchain_data: AggchainDataV1::test1(),
            metadata: Metadata::new(Digest([0xb9; 32])),
            custom_chain_data: Cow::Owned(vec![]),
            l1_info_tree_leaf_count: None,
        }
    }

    fn test4() -> Self {
        Self {
            version: VersionTag,
            network_id: NetworkId::new((1 << 24) - 1),
            height: Height::new(987),
            prev_local_exit_root: LocalExitRoot::from([0x04; 32]),
            new_local_exit_root: LocalExitRoot::from([0x62; 32]),
            bridge_exits: Vec::new().into(),
            imported_bridge_exits: Vec::new().into(),
            aggchain_data: AggchainDataV1::test4(),
            metadata: Metadata::new(Digest([0; 32])),
            custom_chain_data: Cow::Owned(vec![]),
            l1_info_tree_leaf_count: None,
        }
    }

    fn test5() -> Self {
        Self {
            version: VersionTag,
            network_id: NetworkId::new(u32::MAX - 1),
            height: Height::new(987),
            prev_local_exit_root: LocalExitRoot::from([0x04; 32]),
            new_local_exit_root: LocalExitRoot::from([0x62; 32]),
            bridge_exits: Vec::new().into(),
            imported_bridge_exits: Vec::new().into(),
            aggchain_data: AggchainDataV1::test5(),
            metadata: Metadata::new(Digest([0; 32])),
            custom_chain_data: Cow::Owned(vec![]),
            l1_info_tree_leaf_count: None,
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
            l1_info_tree_leaf_count,
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
                AggchainDataV1::GenericNoSignature {
                    proof,
                    aggchain_params,
                } => AggchainDataV1::GenericNoSignature {
                    proof: Cow::Owned(proof.into_owned()),
                    aggchain_params,
                },
                AggchainDataV1::GenericWithSignature {
                    proof,
                    aggchain_params,
                    signature,
                } => AggchainDataV1::GenericWithSignature {
                    proof: Cow::Owned(proof.into_owned()),
                    aggchain_params,
                    signature: Cow::Owned(signature.into_owned()),
                },
                AggchainDataV1::GenericWithPublicValues {
                    proof,
                    aggchain_params,
                    signature,
                    public_values,
                } => AggchainDataV1::GenericWithPublicValues {
                    proof: Cow::Owned(proof.into_owned()),
                    aggchain_params,
                    signature,
                    public_values: Cow::Owned(public_values.into_owned()),
                },
                AggchainDataV1::MultisigOnly { multisig } => AggchainDataV1::MultisigOnly {
                    multisig: Cow::Owned(multisig.into_owned()),
                },
                AggchainDataV1::MultisigAndAggchainProof {
                    multisig,
                    proof,
                    aggchain_params,
                    public_values,
                } => AggchainDataV1::MultisigAndAggchainProof {
                    multisig: Cow::Owned(multisig.into_owned()),
                    proof: Cow::Owned(proof.into_owned()),
                    aggchain_params,
                    public_values: public_values.map(|pv| Cow::Owned(pv.into_owned())),
                },
            },
            metadata,
            custom_chain_data: Cow::Owned(custom_chain_data.into_owned()),
            l1_info_tree_leaf_count,
        }
    }
}

#[rstest::rstest]
#[case(CertificateV0::test0(), &[0x00, 0x00, 0x00, 55])]
#[case(CertificateV0::test0().with_network_id(0x123456.into()), &[0x00, 0x12, 0x34, 0x56])]
#[case(CertificateV1::test0(), &[0x01, 0x00, 0x00, 0x00, 57])]
fn encoding_starts_with(#[case] cert: impl Serialize, #[case] start: &[u8]) {
    let bytes = bincode_codec().serialize(&cert).unwrap();
    assert!(bytes.starts_with(start));
}

#[rstest::rstest]
#[case(CertificateV0::test0())]
#[case(CertificateV1::test0())]
#[case(CertificateV1::test1())]
#[case(CertificateV1::test4())]
#[case(CertificateV1::test5())]
#[case(CertificateV1::from(&Certificate::new_for_test(74.into(), Height::new(998))).into_owned())]
fn encoding_roundtrip_consistent_with_into(#[case] orig: impl Into<Certificate> + Serialize) {
    let bytes = bincode_codec().serialize(&orig).unwrap();
    let decoded = Certificate::decode(&bytes).unwrap();
    let converted: Certificate = orig.into();

    // This should really compare the certificates directly but that requires adding
    // whole bunch of `Eq` impl to many types.
    assert_eq!(format!("{converted:?}"), format!("{decoded:?}"));
}

#[rstest::rstest]
#[case("cert_v0_00", CertificateV0::test0())]
#[case("cert_v1_00", CertificateV1::test0())]
#[case("cert_v1_01", CertificateV1::test1())]
#[case("cert_v1_04", CertificateV1::test4())]
#[case("cert_v1_05", CertificateV1::test5())]
#[case("aggdata_v1_00", AggchainDataV1::test0())]
#[case("aggdata_v1_01", AggchainDataV1::test1())]
#[case("aggdata_v1_02", AggchainDataV1::test2())]
#[case("aggdata_v1_03", AggchainDataV1::test3())]
#[case("aggdata_v1_04", AggchainDataV1::test4())]
#[case("aggdata_v1_05", AggchainDataV1::test5())]
#[case("aggdata_v1_04", AggchainDataV1::test4())]
#[case("aggdata_v1_05", AggchainDataV1::test5())]
fn encoding<T>(#[case] name: &str, #[case] value: T)
where
    T: Serialize + serde::de::DeserializeOwned + std::fmt::Debug,
{
    // Snapshots for types where the encoding must stay stable.
    let bytes = Bytes::from(bincode::default().serialize(&value).unwrap());
    insta::assert_snapshot!(name, bytes);

    // Also check decoding must produce the same value.
    let from_bytes: T = bincode::default()
        .deserialize(bytes.as_ref())
        .expect("deserialization failed");

    // This should really compare the certificates directly but that requires adding
    // whole bunch of `Eq` impl to many types.
    assert_eq!(format!("{from_bytes:?}"), format!("{value:?}"));
}

#[rstest::rstest]
#[case("n15-cert_h0")]
#[case("n15-cert_h1")]
#[case("n15-cert_h2")]
#[case("n15-cert_h3")]
fn cert_in_v0_format_decodes(#[case] cert_name: &str) {
    let from_json = sample_data::load_certificate(&format!("{cert_name}.json"));

    let bytes = load_sample_bytes(&format!("encoded/v0-{cert_name}.hex"));
    let from_bytes = Certificate::decode(&bytes).expect("v0 certificate to decode successfully");

    // Again comparing debug output due to lack of `Eq`.
    assert_eq!(format!("{from_bytes:?}"), format!("{from_json:?}"));
}

#[rstest::rstest]
#[case::regression_01("encoded/regression_01.hex")]
#[case::regression_02("encoded/regression_02.hex")]
fn regressions(#[case] cert_filename: &str) {
    let bytes = load_sample_bytes(cert_filename);
    let _certificate = Certificate::decode(&bytes).expect("decoding failed");
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
