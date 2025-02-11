use agglayer_types::U256;

use super::*;
use crate::columns::Codec;

#[test]
fn height_same_size_as_u64() {
    // Just a sanity check to see if the encoded types overlap properly.
    assert_eq!(u64::BITS, Height::BITS);
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
                    aggchain_params: Digest([0x9e; 32]),
                    stark_proof: Default::default(),
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
