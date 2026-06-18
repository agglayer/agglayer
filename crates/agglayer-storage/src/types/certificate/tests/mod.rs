use agglayer_interop_types_v13 as legacy_interop_types_v13;
use agglayer_sp1::ProofExt as _;
use agglayer_types::{
    aggchain_proof::{AggchainData, AggchainProof, MultisigPayload, Proof},
    bincode, Address, Digest, U256,
};
use alloy_primitives::Bytes;
use pessimistic_proof::unified_bridge::{
    AggchainProofPublicValues, BridgeExit, Claim, ClaimFromMainnet, ClaimFromRollup, GlobalIndex,
    ImportedBridgeExit, L1InfoTreeLeaf, L1InfoTreeLeafInner, LeafType, MerkleProof, TokenInfo,
};
use pessimistic_proof_test_suite::sample_data;

use super::*;
use crate::{schema::Codec, types::generated::agglayer::storage::v0 as proto};

mod header;
mod status;
mod structure;

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

fn digest(byte: u8) -> Digest {
    Digest([byte; 32])
}

const EMPTY_ELF: &[u8] = agglayer_types::testutils::EMPTY_ELF_V5;

fn legacy_sp1_proof(version: &str) -> legacy_interop_types_v13::aggchain_proof::Proof {
    use sp1_sdk_v5::Prover as _;

    let client = sp1_sdk_v5::ProverClient::builder().mock().build();
    let (proving_key, vkey) = client.setup(EMPTY_ELF);
    let proof = sp1_sdk_v5::SP1ProofWithPublicValues::create_mock_proof(
        &proving_key,
        sp1_sdk_v5::SP1PublicValues::new(),
        sp1_sdk_v5::SP1ProofMode::Compressed,
        sp1_sdk_v5::SP1_CIRCUIT_VERSION,
    )
    .proof
    .try_as_compressed()
    .unwrap();

    legacy_interop_types_v13::aggchain_proof::Proof::SP1Stark(
        legacy_interop_types_v13::aggchain_proof::SP1StarkWithContext {
            proof,
            vkey,
            version: version.to_owned(),
        },
    )
}

fn mock_sp1_proof(version: &str) -> Proof {
    agglayer_types::testutils::dummy_sp1_stark_proof_with_version(version)
}

fn proto_aggchain_data(aggchain_data: AggchainData) -> proto::AggchainData {
    proto::AggchainData::try_from(&aggchain_data).unwrap()
}

fn bridge_exit(seed: u8) -> BridgeExit {
    BridgeExit {
        leaf_type: LeafType::Transfer,
        token_info: TokenInfo {
            origin_network: NetworkId::new(seed as u32),
            origin_token_address: Address::from([seed; 20]),
        },
        dest_network: NetworkId::new(seed as u32 + 100),
        dest_address: Address::from([seed.wrapping_add(1); 20]),
        amount: U256::from_be_bytes([seed.wrapping_add(2); 32]),
        metadata: Some(digest(seed.wrapping_add(3))),
    }
}

fn merkle_proof(root: u8, sibling_seed: u8) -> MerkleProof {
    MerkleProof::new(
        digest(root),
        std::array::from_fn(|offset| digest(sibling_seed.wrapping_add(offset as u8))),
    )
}

fn l1_leaf(seed: u8) -> L1InfoTreeLeaf {
    L1InfoTreeLeaf {
        l1_info_tree_index: seed as u32,
        rer: digest(seed.wrapping_add(1)),
        mer: digest(seed.wrapping_add(2)),
        inner: L1InfoTreeLeafInner {
            global_exit_root: digest(seed.wrapping_add(3)),
            block_hash: digest(seed.wrapping_add(4)),
            timestamp: 1_000 + seed as u64,
        },
    }
}

fn imported_bridge_exit(seed: u8, claim: Claim) -> ImportedBridgeExit {
    ImportedBridgeExit {
        bridge_exit: bridge_exit(seed),
        global_index: GlobalIndex::new(NetworkId::new(0), seed as u32),
        claim_data: claim,
    }
}

fn mainnet_imported_bridge_exit(seed: u8) -> ImportedBridgeExit {
    imported_bridge_exit(
        seed,
        Claim::Mainnet(Box::new(ClaimFromMainnet {
            proof_leaf_mer: merkle_proof(seed.wrapping_add(11), seed.wrapping_add(12)),
            proof_ger_l1root: merkle_proof(seed.wrapping_add(13), seed.wrapping_add(14)),
            l1_leaf: l1_leaf(seed.wrapping_add(13)),
        })),
    )
}

fn rollup_imported_bridge_exit(seed: u8) -> ImportedBridgeExit {
    let mut imported_bridge_exit = imported_bridge_exit(
        seed,
        Claim::Rollup(Box::new(ClaimFromRollup {
            proof_leaf_ler: merkle_proof(seed.wrapping_add(21), seed.wrapping_add(22)),
            proof_ler_rer: merkle_proof(seed.wrapping_add(23), seed.wrapping_add(24)),
            proof_ger_l1root: merkle_proof(seed.wrapping_add(25), seed.wrapping_add(26)),
            l1_leaf: l1_leaf(seed.wrapping_add(24)),
        })),
    );
    imported_bridge_exit.global_index =
        GlobalIndex::new(NetworkId::new(seed as u32 + 1), seed as u32);
    imported_bridge_exit
}

fn public_values(seed: u8) -> AggchainProofPublicValues {
    AggchainProofPublicValues {
        prev_local_exit_root: digest(seed),
        new_local_exit_root: digest(seed.wrapping_add(1)),
        l1_info_root: digest(seed.wrapping_add(2)),
        origin_network: NetworkId::new(seed as u32),
        commit_imported_bridge_exits: digest(seed.wrapping_add(3)),
        aggchain_params: digest(seed.wrapping_add(4)),
    }
}

fn proto_certificate(aggchain_data: proto::AggchainData) -> proto::Certificate {
    let mut certificate = Certificate::new_for_test(9.into(), Height::new(42));
    certificate.prev_local_exit_root = digest(0x11).into();
    certificate.new_local_exit_root = digest(0x22).into();
    certificate.bridge_exits = vec![bridge_exit(0x33)];
    certificate.imported_bridge_exits = vec![
        mainnet_imported_bridge_exit(0x44),
        rollup_imported_bridge_exit(0x55),
    ];
    certificate.metadata = Metadata::new(digest(0x66));
    certificate.custom_chain_data = vec![0xca, 0xfe, 0xba, 0xbe];
    certificate.l1_info_tree_leaf_count = Some(7);

    let mut proto = proto::Certificate::try_from(&certificate).unwrap();
    proto.aggchain_data = Some(aggchain_data);
    proto
}

impl AggchainDataV1<'static> {
    fn proof0() -> legacy_interop_types_v13::aggchain_proof::Proof {
        legacy_sp1_proof(sp1_sdk_v5::SP1_CIRCUIT_VERSION)
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

#[rstest::rstest]
#[case(CertificateV0::test0(), &[0x00, 0x00, 0x00, 55])]
#[case(CertificateV0::test0().with_network_id(0x123456.into()), &[0x00, 0x12, 0x34, 0x56])]
#[case(CertificateV1::test0(), &[0x01, 0x00, 0x00, 0x00, 57])]
fn encoding_starts_with(#[case] cert: impl Serialize, #[case] start: &[u8]) {
    let bytes = bincode_codec().serialize(&cert).unwrap();
    assert!(bytes.starts_with(start));
}

// SP1 v6 made the legacy `CertificateV1` → `Certificate` conversion fallible
// (`TryFrom`) and dropped the `From<&Certificate> for CertificateV1` impl, so
// the synthesised round-trip case is no longer expressible. The
// pre-existing fixtures still exercise the legacy CF decode path.
#[rstest::rstest]
#[case(CertificateV0::test0(), Certificate::from(CertificateV0::test0()))]
#[case(CertificateV1::test0(), Certificate::try_from(CertificateV1::test0()).unwrap())]
#[case(CertificateV1::test1(), Certificate::try_from(CertificateV1::test1()).unwrap())]
#[case(CertificateV1::test4(), Certificate::try_from(CertificateV1::test4()).unwrap())]
#[case(CertificateV1::test5(), Certificate::try_from(CertificateV1::test5()).unwrap())]
fn legacy_decoding_roundtrip_consistent_with_into<T>(
    #[case] orig: T,
    #[case] converted: Certificate,
) where
    T: Serialize,
{
    let bytes = bincode_codec().serialize(&orig).unwrap();
    let decoded = LegacyCertificate::decode(&bytes).unwrap();

    assert_eq!(converted, Certificate::from(decoded));
}

/// Regression test for the storage-v1 migration boundary: legacy v5 proof
/// bytes must remain readable by downstream `agglayer-sp1` hash helpers after
/// decoding a stored V1 certificate row.
#[test]
fn regression_certificate_v1_decode_preserves_legacy_v5_vkey_hash() {
    use sp1_sdk_v5::HashableKey as _;

    let certificate = CertificateV1::test1();
    let expected = match &certificate.aggchain_data {
        AggchainDataV1::GenericWithSignature { proof, .. } => match proof.as_ref() {
            legacy_interop_types_v13::aggchain_proof::Proof::SP1Stark(proof) => {
                proof.vkey.hash_bytes()
            }
        },
        _ => panic!("expected GenericWithSignature fixture"),
    };

    let bytes = bincode_codec().serialize(&certificate).unwrap();
    let decoded = Certificate::from(LegacyCertificate::decode(&bytes).unwrap());
    let proof = match &decoded.aggchain_data {
        AggchainData::Generic { proof, .. } => proof,
        _ => panic!("expected Generic aggchain data after V1 decode"),
    };

    assert_eq!(proof.vkey_hash_bytes().unwrap(), expected);
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
    T: Serialize + serde::de::DeserializeOwned + std::fmt::Debug + std::cmp::Eq,
{
    // Snapshots for types where the encoding must stay stable.
    let bytes = Bytes::from(bincode::default().serialize(&value).unwrap());
    insta::assert_snapshot!(name, bytes);

    // Also check decoding must produce the same value.
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
fn legacy_cert_in_v0_format_decodes(#[case] cert_name: &str) {
    let from_json = sample_data::load_certificate(&format!("{cert_name}.json"));

    let bytes = load_sample_bytes(&format!("encoded/v0-{cert_name}.hex"));
    let from_bytes =
        LegacyCertificate::decode(&bytes).expect("v0 certificate to decode successfully");

    assert_eq!(Certificate::from(from_bytes), from_json);
}

#[rstest::rstest]
#[case::regression_01("encoded/regression_01.hex")]
#[case::regression_02("encoded/regression_02.hex")]
fn legacy_regressions(#[case] cert_filename: &str) {
    let bytes = load_sample_bytes(cert_filename);
    let _legacy = LegacyCertificate::decode(&bytes).expect("legacy decoding failed");
}

#[test]
fn legacy_certificate_decode_per_byte_dispatch() {
    // Empty input is rejected explicitly.
    assert!(matches!(
        LegacyCertificate::decode(&[]).unwrap_err(),
        CodecError::CertificateEmpty
    ));

    // Bytes 0 and 1 enter the bincode path; with no payload, bincode
    // produces a serialization error.
    for v in 0..=1 {
        assert!(
            matches!(
                LegacyCertificate::decode(&[v]).unwrap_err(),
                CodecError::Serialization(_)
            ),
            "expected bincode serialization error for byte {v:#x}"
        );
    }

    // Other bytes enter the proto path. The legacy CF carried proto rows
    // before the proto CF split, so this branch must be live. With no
    // payload the proto decoder either rejects the bytes outright or
    // produces a default-valued message that fails `Certificate::try_from`.
    for v in 2..=u8::MAX {
        let err = LegacyCertificate::decode(&[v]).unwrap_err();
        assert!(
            matches!(
                err,
                CodecError::ProtobufDeserialization(_) | CodecError::Conversion(_)
            ),
            "expected proto decode/conversion error for byte {v:#x}, got {err:?}"
        );
    }
}

#[test]
fn certificate_proto_decode_rejects_empty_bytes() {
    // The proto codec must not silently accept zero-length payloads as a
    // legacy fallback. A real proto-encoded certificate always contains the
    // required `prev_local_exit_root`/`new_local_exit_root` fields, so an
    // empty input is unambiguously corrupt.
    let err = <Certificate as crate::schema::Codec>::decode(&[]).unwrap_err();
    assert!(
        matches!(
            err,
            CodecError::Conversion(_) | CodecError::ProtobufDeserialization(_)
        ),
        "unexpected error variant: {err:?}"
    );
}

/// Smoke test: malformed bytes must surface as a recoverable `CodecError`,
/// never as a process-level panic. Covers the common case where bincode
/// returns its own error (no panic reached).
#[test]
fn legacy_decode_of_corrupt_bytes_returns_codec_error() {
    // A valid v1 version byte followed by junk that bincode rejects.
    let bytes = [0x01u8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    let err = <LegacyCertificate as crate::schema::Codec>::decode(&bytes)
        .expect_err("expected decode to fail, not panic");
    assert!(
        matches!(err, crate::schema::CodecError::Serialization(_)),
        "unexpected error variant: {err:?}"
    );
}

/// Regression guard: if a deserializer ever panics mid-decode, the
/// `catch_unwind` wrapper in `deserialize_bincode` must convert the panic
/// into a `CodecError::Serialization`, not let it escape. We provoke this
/// with a test-only type whose `Deserialize` impl deliberately panics.
#[test]
fn catch_unwind_converts_deserializer_panic_into_codec_error() {
    use std::sync::{Mutex, OnceLock};

    use serde::de::{Deserialize, Deserializer};

    #[derive(Debug)]
    struct PanickingOnDeserialize;

    impl<'de> Deserialize<'de> for PanickingOnDeserialize {
        fn deserialize<D: Deserializer<'de>>(_deserializer: D) -> Result<Self, D::Error> {
            panic!("intentional panic for catch_unwind test");
        }
    }

    // Borrow the real catch_unwind helper by decoding through bincode.
    // `deserialize_bincode` is module-private, so call it via the
    // sibling `super::deserialize_bincode::<PanickingOnDeserialize>(...)`.
    //
    // Suppress the default panic hook during the intentional panic so CI
    // logs are not polluted by the stack trace. Protect the process-wide
    // panic hook with a lock so parallel tests cannot race on the hook
    // state. The hook is restored immediately after the call returns.
    static PANIC_HOOK_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let _guard = PANIC_HOOK_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("panic hook lock poisoned");
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = super::deserialize_bincode::<PanickingOnDeserialize>(&[0u8; 0]);
    std::panic::set_hook(prev_hook);

    match result {
        Err(crate::schema::CodecError::Serialization(_)) => {}
        Err(other) => panic!("unexpected error variant: {other:?}"),
        Ok(_) => panic!("deserialize_bincode returned Ok on a panicking type"),
    }
}

#[test]
fn certificate_proto_roundtrip_preserves_nested_certificate_data() {
    let proto = proto_certificate(proto_aggchain_data(AggchainData::Generic {
        proof: mock_sp1_proof("v5.2.2"),
        aggchain_params: digest(0x72),
        signature: Some(Box::new(sig(0x73, 0x74))),
        public_values: Some(Box::new(public_values(0x74))),
    }));

    let typed = Certificate::try_from(proto.clone()).unwrap();
    let roundtrip = proto::Certificate::try_from(&typed).unwrap();

    assert_eq!(roundtrip, proto);
    assert_eq!(typed.metadata, Metadata::new(Digest([0x66; 32])));
    assert_eq!(typed.custom_chain_data, vec![0xca, 0xfe, 0xba, 0xbe]);
    assert_eq!(typed.l1_info_tree_leaf_count, Some(7));
}

#[rstest::rstest]
#[case(proto_aggchain_data(AggchainData::ECDSA {
    signature: sig(0x80, 0x81),
}))]
#[case(proto_aggchain_data(AggchainData::Generic {
    proof: mock_sp1_proof("v5.2.2"),
    aggchain_params: digest(0x82),
    signature: None,
    public_values: None,
}))]
#[case(proto_aggchain_data(AggchainData::MultisigOnly {
    multisig: MultisigPayload(vec![
        Some(sig(0x83, 0x84)),
        None,
        Some(sig(0x85, 0x86)),
        None,
    ]),
}))]
#[case(proto_aggchain_data(AggchainData::MultisigAndAggchainProof {
    multisig: MultisigPayload(vec![None, Some(sig(0x87, 0x88)), None]),
    aggchain_proof: AggchainProof {
        proof: mock_sp1_proof("v5.2.2"),
        aggchain_params: digest(0x87),
        public_values: Some(Box::new(public_values(0x88))),
    },
}))]
fn certificate_proto_roundtrip_preserves_aggchain_variants(
    #[case] aggchain_data: proto::AggchainData,
) {
    let proto = proto_certificate(aggchain_data);

    let typed = Certificate::try_from(proto.clone()).unwrap();
    let roundtrip = proto::Certificate::try_from(&typed).unwrap();

    assert_eq!(roundtrip.aggchain_data, proto.aggchain_data);
}

#[test]
fn certificate_proto_roundtrip_preserves_read_only_proof_versions() {
    let proto = proto_certificate(proto_aggchain_data(AggchainData::Generic {
        proof: mock_sp1_proof("v6.0.1"),
        aggchain_params: digest(0xa0),
        signature: None,
        public_values: Some(Box::new(public_values(0xa1))),
    }));

    let typed = Certificate::try_from(proto.clone()).unwrap();
    let roundtrip = proto::Certificate::try_from(&typed).unwrap();

    assert_eq!(roundtrip.aggchain_data, proto.aggchain_data);
}

#[test]
fn certificate_proto_rejects_malformed_nested_submessages() {
    let missing_generic_proof = proto_certificate(proto::AggchainData {
        data: Some(proto::aggchain_data::Data::Generic(proto::Generic {
            proof: None,
            aggchain_params: Some(digest(0x91).into()),
            signature: None,
            public_values: None,
        })),
    });
    assert!(Certificate::try_from(missing_generic_proof).is_err());

    let missing_l1_leaf_inner = proto::Certificate {
        imported_bridge_exits: vec![proto::ImportedBridgeExit {
            bridge_exit: Some(bridge_exit(0x92).into()),
            global_index: Some(GlobalIndex::new(NetworkId::new(0), 0x93).into()),
            claim: Some(proto::imported_bridge_exit::Claim::Mainnet(
                proto::ClaimFromMainnet {
                    proof_leaf_mer: Some(merkle_proof(0x94, 0x95).into()),
                    proof_ger_l1root: Some(merkle_proof(0x96, 0x97).into()),
                    l1_leaf: Some(proto::L1InfoTreeLeafWithContext {
                        l1_info_tree_index: 5,
                        rer: Some(digest(0x98).into()),
                        mer: Some(digest(0x99).into()),
                        inner: None,
                    }),
                },
            )),
        }],
        ..proto_certificate(proto_aggchain_data(AggchainData::ECDSA {
            signature: sig(0x99, 0x9a),
        }))
    };
    assert!(Certificate::try_from(missing_l1_leaf_inner).is_err());
}

#[test]
fn legacy_v0_decode_proto_roundtrip_stays_lossless() {
    let bytes = load_sample_bytes("encoded/v0-n15-cert_h0.hex");
    let typed = Certificate::from(LegacyCertificate::decode(&bytes).unwrap());

    let proto = proto::Certificate::try_from(&typed).unwrap();
    let roundtrip = Certificate::try_from(proto).unwrap();

    assert_eq!(roundtrip, typed);
}
