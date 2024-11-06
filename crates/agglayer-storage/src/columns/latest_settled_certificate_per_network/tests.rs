use agglayer_types::NetworkId;

use super::{Key, SettledCertificate};
use crate::columns::Codec as _;

#[test]
fn can_parse_key() {
    let key: NetworkId = 1.into();

    let encoded = key.encode().expect("Unable to encode key");

    let expected_key = Key::decode(&encoded[..]).expect("Unable to decode key");

    assert_eq!(expected_key, key);
}

#[test]
fn can_parse_value() {
    let value = SettledCertificate([1; 32].into(), 10, 21, 0);

    let encoded = value.encode().expect("Unable to encode value");

    let expected_value = SettledCertificate::decode(&encoded[..]).expect("Unable to decode value");

    assert_eq!(expected_value, value);

    // certificate_id
    assert_eq!(
        encoded[..32],
        [
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1,
        ]
    );
    // height
    assert_eq!(encoded[32..40], [0, 0, 0, 0, 0, 0, 0, 10]);
    // epoch_number
    assert_eq!(encoded[40..48], [0, 0, 0, 0, 0, 0, 0, 21]);
}
