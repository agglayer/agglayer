use super::{Key, Value};
use crate::{
    columns::Codec as _,
    types::{self, CertificateId},
};

#[test]
fn can_parse_key() {
    let key = CertificateId([1; 32]);

    let encoded = key.encode().expect("Unable to encode key");

    let expected_key = Key::decode(&encoded[..]).expect("Unable to decode key");

    assert_eq!(expected_key, key);
}

#[test]
fn can_parse_value() {
    let value = Value {
        network_id: types::NetworkId(1),
        height: types::Height(2),
        epoch_number: types::EpochNumber(3),
        certificate_index: types::CertificateIndex(4),
        local_exit_root: types::Hash([5; 32]),
    };

    let encoded = value.encode().expect("Unable to encode value");

    let expected_value = Value::decode(&encoded[..]).expect("Unable to decode value");

    assert_eq!(expected_value, value);

    // network_id
    assert_eq!(encoded[..4], [0, 0, 0, 1]);
    // height
    assert_eq!(encoded[4..12], [0, 0, 0, 0, 0, 0, 0, 2]);
    // epoch_number
    assert_eq!(encoded[12..20], [0, 0, 0, 0, 0, 0, 0, 3]);
    // certificate_index
    assert_eq!(encoded[20..28], [0, 0, 0, 0, 0, 0, 0, 4]);
    // local_exit_root
    assert_eq!(
        encoded[28..60],
        [
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            5, 5, 5
        ]
    );
}
