use super::{Key, Value};
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
    let value = Value {
        certificate_id: [1; 32].into(),
        epoch_number: 3.into(),
        certificate_index: 4.into(),
    };

    let encoded = value.encode().expect("Unable to encode value");

    let expected_value = Value::decode(&encoded[..]).expect("Unable to decode value");

    assert_eq!(expected_value, value);

    // certificate_id
    assert_eq!(
        encoded[..32],
        [
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1,
        ]
    );
    // epoch_number
    assert_eq!(encoded[32..40], [0, 0, 0, 0, 0, 0, 0, 3]);
    // certificate_index
    assert_eq!(encoded[40..48], [0, 0, 0, 0, 0, 0, 0, 4]);
}
