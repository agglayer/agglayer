use super::{Key, KeyType};
use crate::columns::Codec as _;

#[test]
fn can_serialize_a_root_key() {
    let key = Key {
        network_id: 1,
        key_type: KeyType::Root,
    };

    //              -> [  network ][ key type ]
    let expected = vec![0, 0, 0, 1, 0, 0, 0, 0];

    assert_eq!(key.encode().unwrap(), expected);

    let expected_key = Key::decode(&expected[..]).expect("Unable to decode key");

    assert_eq!(expected_key, key);
}

#[test]
fn can_serialize_a_node_key() {
    let key = Key {
        network_id: 1,
        key_type: KeyType::Node([1; 32]),
    };

    //                  -> [  network ][ key type ]
    let mut expected = vec![0, 0, 0, 1, 0, 0, 0, 1];
    //                       -> [ hash ]
    expected.extend_from_slice(&[1; 32]);

    assert_eq!(key.encode().unwrap(), expected);

    let expected_key = Key::decode(&expected[..]).expect("Unable to decode key");

    assert_eq!(expected_key, key);
}

#[test]
fn can_serialize_a_leaf_key() {
    let key = Key {
        network_id: 1,
        key_type: KeyType::Leaf([1; 32]),
    };

    //                  -> [  network ][ key type ]
    let mut expected = vec![0, 0, 0, 1, 0, 0, 0, 2];
    //                       -> [ hash ]
    expected.extend_from_slice(&[1; 32]);

    assert_eq!(key.encode().unwrap(), expected);

    let expected_key = Key::decode(&expected[..]).expect("Unable to decode key");

    assert_eq!(expected_key, key);
}

#[test]
fn can_serialize_deserialize_a_leaves_key() {
    let key = Key {
        network_id: 1,
        key_type: KeyType::Leaves,
    };

    //              -> [  network ][ key type ]
    let expected = vec![0, 0, 0, 1, 0, 0, 0, 3];

    assert_eq!(key.encode().unwrap(), expected);

    let expected_key = Key::decode(&expected[..]).expect("Unable to decode key");

    assert_eq!(expected_key, key);
}
