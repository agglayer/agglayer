use agglayer_types::CertificateHeader;

use super::Key;
use crate::columns::Codec as _;

#[test]
fn can_parse_key() {
    let key = Key {
        network_id: 1,
        height: 200,
    };

    let encoded = key.encode().expect("Unable to encode key");

    let expected_key = Key::decode(&encoded[..]).expect("Unable to decode key");

    assert_eq!(expected_key, key);
}

#[test]
fn can_parse_value() {
    let value = CertificateHeader {
        network_id: 1.into(),
        certificate_id: [1; 32].into(),
        epoch_number: Some(3),
        certificate_index: Some(4),
        height: 0,
        new_local_exit_root: [0; 32].into(),
        status: agglayer_types::CertificateStatus::Pending,
    };

    let encoded = value.encode().expect("Unable to encode value");

    let expected_value = CertificateHeader::decode(&encoded[..]).expect("Unable to decode value");

    assert_eq!(expected_value, value);

    // network_id
    assert_eq!(encoded[..4], [0, 0, 0, 1]);
    // height
    assert_eq!(encoded[4..12], [0, 0, 0, 0, 0, 0, 0, 0]);
    // epoch_number
    assert_eq!(encoded[12..21], [1, 0, 0, 0, 0, 0, 0, 0, 3]);
    // certificate_index
    assert_eq!(encoded[21..30], [1, 0, 0, 0, 0, 0, 0, 0, 4]);
    // CertificateId
    assert_eq!(
        encoded[30..62],
        [
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1
        ]
    );
    // new_local_exit_root
    assert_eq!(
        encoded[62..94],
        [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0
        ]
    );

    assert_eq!(encoded[94..98], [0, 0, 0, 0]);
    assert!(encoded[98..].is_empty());
}
