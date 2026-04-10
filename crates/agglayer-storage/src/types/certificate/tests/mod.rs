use agglayer_types::{
    aggchain_proof::{
        current_sp1_stark_with_context, AggchainData, CurrentSp1StarkProof, Proof,
        SP1StarkWithContextExt as _, Sp1ProofVersion,
    },
    bincode, U256,
};
use alloy_primitives::Bytes;
use pessimistic_proof_test_suite::sample_data;
use sp1_sdk::{blocking::Prover, ProvingKey};
use sp1_sdk_v5::Prover as _;

use super::*;
use crate::schema::Codec;

mod header;
mod status;
mod structure;

const EMPTY_ELF: &[u8] = include_bytes!("empty.elf");

/// Any valid riscv64 ELF works here; we only need it to derive proving keys for
/// mock proof creation.
const TEST_ELF: &[u8] = pessimistic_proof::ELF;
const LEGACY_SP1_VERSION: &str = "v4.0.0-rc.3";

impl PartialEq for AggchainDataV1<'_> {
    fn eq(&self, other: &Self) -> bool {
        bincode::default().serialize(self).unwrap() == bincode::default().serialize(other).unwrap()
    }
}

impl Eq for AggchainDataV1<'_> {}

impl PartialEq for CertificateV1<'_> {
    fn eq(&self, other: &Self) -> bool {
        bincode::default().serialize(self).unwrap() == bincode::default().serialize(other).unwrap()
    }
}

impl Eq for CertificateV1<'_> {}

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

fn legacy_sp1_proof0() -> legacy_interop_types::aggchain_proof::Proof {
    let client = sp1_sdk_v5::ProverClient::builder().mock().build();
    let (proving_key, verifying_key) = client.setup(EMPTY_ELF);
    let dummy_proof = sp1_sdk_v5::SP1ProofWithPublicValues::create_mock_proof(
        &proving_key,
        sp1_sdk_v5::SP1PublicValues::new(),
        sp1_sdk_v5::SP1ProofMode::Compressed,
        sp1_sdk_v5::SP1_CIRCUIT_VERSION,
    );
    let proof = dummy_proof.proof.try_as_compressed().unwrap();

    legacy_interop_types::aggchain_proof::Proof::SP1Stark(
        legacy_interop_types::aggchain_proof::SP1StarkWithContext {
            proof,
            vkey: verifying_key,
            version: LEGACY_SP1_VERSION.to_string(),
        },
    )
}

fn current_sp1_parts() -> (Box<CurrentSp1StarkProof>, sp1_sdk::SP1VerifyingKey) {
    let client = sp1_sdk::blocking::ProverClient::builder().mock().build();
    let proving_key = client.setup(sp1_sdk::Elf::Static(TEST_ELF)).unwrap();
    let verifying_key = proving_key.verifying_key().clone();
    let dummy_proof = sp1_sdk::SP1ProofWithPublicValues::create_mock_proof(
        proving_key.verifying_key(),
        sp1_sdk::SP1PublicValues::new(),
        sp1_sdk::SP1ProofMode::Compressed,
        sp1_sdk::SP1_CIRCUIT_VERSION,
    );
    let proof = dummy_proof.proof.try_as_compressed().unwrap();
    (proof, verifying_key)
}

fn current_sp1_proof_v2() -> Proof {
    let (proof, verifying_key) = current_sp1_parts();
    Proof::SP1Stark(
        current_sp1_stark_with_context(
            proof.as_ref(),
            &verifying_key,
            sp1_sdk::SP1_CIRCUIT_VERSION,
        )
        .unwrap(),
    )
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

fn normalize_legacy_aggchain_data_version(aggchain_data: &mut AggchainDataV1<'_>) {
    let proof = match aggchain_data {
        AggchainDataV1::ECDSA { .. } | AggchainDataV1::MultisigOnly { .. } => return,
        AggchainDataV1::GenericNoSignature { proof, .. }
        | AggchainDataV1::GenericWithSignature { proof, .. }
        | AggchainDataV1::GenericWithPublicValues { proof, .. }
        | AggchainDataV1::MultisigAndAggchainProof { proof, .. } => proof.to_mut(),
    };

    let legacy_interop_types::aggchain_proof::Proof::SP1Stark(proof) = proof;
    proof.version = LEGACY_SP1_VERSION.to_string();
}

fn normalize_legacy_certificate_version(certificate: &mut CertificateV1<'_>) {
    normalize_legacy_aggchain_data_version(&mut certificate.aggchain_data);
}

impl AggchainDataV1<'static> {
    fn test0() -> Self {
        Self::ECDSA { signature: sig0() }
    }

    fn test1() -> Self {
        Self::GenericWithSignature {
            proof: Cow::Owned(legacy_sp1_proof0()),
            aggchain_params: Digest([0x58; 32]),
            signature: Cow::Owned(Box::new(sig(0x78, 0x9a))),
        }
    }

    fn test2() -> Self {
        Self::GenericNoSignature {
            proof: Cow::Owned(legacy_sp1_proof0()),
            aggchain_params: Digest([0x59; 32]),
        }
    }

    fn test3() -> Self {
        let aggchain_params = Digest([0x60; 32]);
        Self::GenericWithPublicValues {
            proof: Cow::Owned(legacy_sp1_proof0()),
            aggchain_params,
            signature: None,
            public_values: Cow::Owned(Box::new(aggchain_proof_public_values0(aggchain_params))),
        }
    }

    fn test4() -> Self {
        Self::MultisigOnly {
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
        Self::MultisigAndAggchainProof {
            multisig: Cow::Owned(vec![
                Some(sig(0x55, 0x66)),
                None,
                Some(sig(0x77, 0x88)),
                None,
                None,
            ]),
            proof: Cow::Owned(legacy_sp1_proof0()),
            aggchain_params,
            public_values: Some(Cow::Owned(Box::new(aggchain_proof_public_values0(
                aggchain_params,
            )))),
        }
    }
}

impl AggchainDataV2<'static> {
    fn test0() -> Self {
        Self::ECDSA { signature: sig0() }
    }

    fn test1() -> Self {
        Self::GenericWithSignature {
            proof: Cow::Owned(current_sp1_proof_v2()),
            aggchain_params: Digest([0x58; 32]),
            signature: Cow::Owned(Box::new(sig(0x78, 0x9a))),
        }
    }

    fn test2() -> Self {
        Self::GenericNoSignature {
            proof: Cow::Owned(current_sp1_proof_v2()),
            aggchain_params: Digest([0x59; 32]),
        }
    }

    fn test3() -> Self {
        let aggchain_params = Digest([0x60; 32]);
        Self::GenericWithPublicValues {
            proof: Cow::Owned(current_sp1_proof_v2()),
            aggchain_params,
            signature: None,
            public_values: Cow::Owned(Box::new(aggchain_proof_public_values0(aggchain_params))),
        }
    }

    fn test4() -> Self {
        Self::MultisigOnly {
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
        Self::MultisigAndAggchainProof {
            multisig: Cow::Owned(vec![
                Some(sig(0x55, 0x66)),
                None,
                Some(sig(0x77, 0x88)),
                None,
                None,
            ]),
            proof: Cow::Owned(current_sp1_proof_v2()),
            aggchain_params,
            public_values: Some(Cow::Owned(Box::new(aggchain_proof_public_values0(
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

impl CertificateV2<'static> {
    fn test0() -> Self {
        Self {
            version: VersionTag,
            network_id: NetworkId::new(57),
            height: Height::new(987),
            prev_local_exit_root: LocalExitRoot::from([0x02; 32]),
            new_local_exit_root: LocalExitRoot::from([0x65; 32]),
            bridge_exits: Vec::new().into(),
            imported_bridge_exits: Vec::new().into(),
            aggchain_data: AggchainDataV2::test0(),
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
            aggchain_data: AggchainDataV2::test1(),
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
            aggchain_data: AggchainDataV2::test4(),
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
            aggchain_data: AggchainDataV2::test5(),
            metadata: Metadata::new(Digest([0; 32])),
            custom_chain_data: Cow::Owned(vec![]),
            l1_info_tree_leaf_count: None,
        }
    }
}

#[rstest::rstest]
#[case(CertificateV0::test0(), &[0x00, 0x00, 0x00, 55])]
#[case(CertificateV0::test0().with_network_id(0x123456.into()), &[0x00, 0x12, 0x34, 0x56])]
#[case(CertificateV1::test0(), &[0x01, 0x00, 0x00, 0x00, 57])]
#[case(CertificateV2::test0(), &[0x02, 0x00, 0x00, 0x00, 57])]
fn encoding_starts_with(#[case] cert: impl Serialize, #[case] start: &[u8]) {
    let bytes = bincode_codec().serialize(&cert).unwrap();
    assert!(bytes.starts_with(start));
}

#[rstest::rstest]
#[case(CertificateV0::test0(), Certificate::try_from(CertificateV0::test0()).unwrap())]
#[case(CertificateV1::test0(), Certificate::try_from(CertificateV1::test0()).unwrap())]
#[case(CertificateV1::test1(), Certificate::try_from(CertificateV1::test1()).unwrap())]
#[case(CertificateV1::test4(), Certificate::try_from(CertificateV1::test4()).unwrap())]
#[case(CertificateV1::test5(), Certificate::try_from(CertificateV1::test5()).unwrap())]
#[case(CertificateV2::test0(), Certificate::try_from(CertificateV2::test0()).unwrap())]
#[case(CertificateV2::test1(), Certificate::try_from(CertificateV2::test1()).unwrap())]
#[case(CertificateV2::test4(), Certificate::try_from(CertificateV2::test4()).unwrap())]
#[case(CertificateV2::test5(), Certificate::try_from(CertificateV2::test5()).unwrap())]
fn encoding_roundtrip_consistent_with_into<T>(#[case] orig: T, #[case] converted: Certificate)
where
    T: Serialize,
{
    let bytes = bincode_codec().serialize(&orig).unwrap();
    let decoded = Certificate::decode(&bytes).unwrap();

    assert_eq!(converted, decoded);
}

#[rstest::rstest]
#[case("cert_v0_00", CertificateV0::test0())]
#[case("cert_v1_00", CertificateV1::test0())]
#[case("cert_v1_01", CertificateV1::test1())]
#[case("cert_v1_04", CertificateV1::test4())]
#[case("cert_v1_05", CertificateV1::test5())]
#[case("cert_v2_00", CertificateV2::test0())]
#[case("cert_v2_01", CertificateV2::test1())]
#[case("cert_v2_04", CertificateV2::test4())]
#[case("cert_v2_05", CertificateV2::test5())]
#[case("aggdata_v1_00", AggchainDataV1::test0())]
#[case("aggdata_v1_01", AggchainDataV1::test1())]
#[case("aggdata_v1_02", AggchainDataV1::test2())]
#[case("aggdata_v1_03", AggchainDataV1::test3())]
#[case("aggdata_v1_04", AggchainDataV1::test4())]
#[case("aggdata_v1_05", AggchainDataV1::test5())]
#[case("aggdata_v2_00", AggchainDataV2::test0())]
#[case("aggdata_v2_01", AggchainDataV2::test1())]
#[case("aggdata_v2_02", AggchainDataV2::test2())]
#[case("aggdata_v2_03", AggchainDataV2::test3())]
#[case("aggdata_v2_04", AggchainDataV2::test4())]
#[case("aggdata_v2_05", AggchainDataV2::test5())]
fn encoding<T>(#[case] name: &str, #[case] value: T)
where
    T: Serialize + serde::de::DeserializeOwned + std::fmt::Debug + std::cmp::Eq,
{
    // The pre-existing V1 snapshots intentionally update in this branch because
    // they now encode the real historical <=0.13 proof shape and version string
    // (`v4.0.0-rc.3`) instead of the synthetic placeholder values that were
    // previously used in these test fixtures.
    let bytes = Bytes::from(bincode::default().serialize(&value).unwrap());
    insta::assert_snapshot!(name, bytes);

    let from_bytes: T = bincode::default()
        .deserialize(bytes.as_ref())
        .expect("deserialization failed");

    assert_eq!(from_bytes, value);
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

    assert_eq!(from_bytes, from_json);
}

#[rstest::rstest]
#[case::regression_01("encoded/regression_01.hex")]
#[case::regression_02("encoded/regression_02.hex")]
fn regressions(#[case] cert_filename: &str) {
    let bytes = load_sample_bytes(cert_filename);
    let _certificate = Certificate::decode(&bytes).expect("decoding failed");
}

#[test]
fn regression_01_legacy() {
    let bytes = load_sample_bytes("encoded/regression_01.hex");
    let certificate = Certificate::decode(&bytes).expect("decoding failed");

    let AggchainData::Generic { proof, .. } = certificate.aggchain_data else {
        panic!("expected generic aggchain data in legacy regression fixture")
    };

    let Proof::SP1Stark(proof) = proof;
    assert_eq!(proof.ensure_readable().unwrap(), Sp1ProofVersion::V4);
}

#[test]
fn store_writes_certificate_v2() {
    let certificate = Certificate::new_for_test(74.into(), Height::new(998));
    let encoded = Certificate::encode(&certificate).expect("encoding failed");

    // Byte 0 is the storage schema tag: 0 = V0, 1 = V1, 2 = the new V2 format.
    assert_eq!(encoded.first().copied(), Some(2));
}

#[test]
fn regression_01_legacy_can_be_reencoded_for_storage() {
    let bytes = load_sample_bytes("encoded/regression_01.hex");
    let certificate = Certificate::decode(&bytes).expect("decoding failed");

    let encoded = Certificate::encode(&certificate).expect("encoding failed");
    let decoded = Certificate::decode(&encoded).expect("re-decoding failed");

    // Legacy V1 rows are rewritten into the current V2 on-disk format.
    assert_eq!(encoded.first().copied(), Some(2));
    assert_eq!(decoded, certificate);
}

#[rstest::rstest]
#[case("encoded/main-preupdate-cert-v1-01.hex", CertificateV1::test1())]
#[case("encoded/main-preupdate-cert-v1-05.hex", CertificateV1::test5())]
fn main_v1_snapshots_normalize_to_current_historical_fixtures(
    #[case] fixture: &str,
    #[case] expected: CertificateV1<'static>,
) {
    let bytes = load_sample_bytes(fixture);
    let mut decoded: CertificateV1<'static> = bincode::default().deserialize(&bytes).unwrap();

    normalize_legacy_certificate_version(&mut decoded);

    assert_eq!(decoded, expected);
    assert_eq!(
        bincode::default().serialize(&decoded).unwrap(),
        bincode::default().serialize(&expected).unwrap()
    );
}

#[test]
fn bad_format() {
    const NEXT_VERSION: u8 = 3;

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
