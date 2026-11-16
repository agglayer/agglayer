use strum::EnumCount;

use super::Key;
use crate::types::network_info::v0::network_info_value::ValueDiscriminants;

impl<'a> arbitrary::Arbitrary<'a> for Key {
    fn arbitrary(input: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let kind_index =
            <u8 as arbitrary::Arbitrary>::arbitrary(input)? as usize % ValueDiscriminants::COUNT;

        Ok(Self {
            network_id: <u32 as arbitrary::Arbitrary>::arbitrary(input)?,
            kind: ValueDiscriminants::from_repr(kind_index)
                .expect("modulo keeps discriminant in range"),
        })
    }
}

#[test]
fn test_discriminant_from_u32() {
    assert_eq!(ValueDiscriminants::COUNT, 5, "Expected 5 discriminants");
    for i in 0..ValueDiscriminants::COUNT {
        let discriminant = ValueDiscriminants::from_repr(i);
        assert!(discriminant.is_some());

        match discriminant.unwrap() {
            ValueDiscriminants::NetworkType => {
                assert_eq!(i, 0, "NetworkType should be at index 0")
            }
            ValueDiscriminants::SettledCertificate => {
                assert_eq!(i, 1, "SettledCertificate should be at index 1")
            }
            ValueDiscriminants::SettledClaim => {
                assert_eq!(i, 2, "SettledClaim should be at index 2")
            }
            ValueDiscriminants::LatestPendingCertificateInfo => {
                assert_eq!(i, 3, "LatestPendingCertificateInfo should be at index 3")
            }
            ValueDiscriminants::LatestProvenCertificateInfo => {
                assert_eq!(i, 4, "LatestProvenCertificateInfo should be at index 4")
            }
        }
    }
}

#[test]
fn network_type_maps_to_its_storage_value() {
    use agglayer_types::NetworkType as Domain;

    use crate::types::network_info::v0::NetworkType as Stored;

    for (domain, stored) in [
        (Domain::Unspecified, 0),
        (Domain::Ecdsa, 1),
        (Domain::Generic, 2),
        (Domain::MultisigOnly, 3),
        (Domain::MultisigAndAggchainProof, 4),
    ] {
        assert_eq!(Stored::from(domain.clone()) as i32, stored, "{domain:?}");
        let decoded = Domain::try_from(Stored::try_from(stored).unwrap()).unwrap();
        assert_eq!(decoded, domain);
    }
}

crate::types::codec_tests::codec_tests!(Key {
    network_id: 0x01020304,
    kind: ValueDiscriminants::SettledClaim,
});
