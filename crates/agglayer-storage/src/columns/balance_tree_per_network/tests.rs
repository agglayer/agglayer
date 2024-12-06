use agglayer_types::Digest;

use crate::columns::Codec as _;
use crate::types::{SmtKey, SmtKeyType};

#[test]
fn can_serialize_a_root_key() {
    let key = SmtKey {
        network_id: 1,
        key_type: SmtKeyType::Root,
    };

    //              -> [  network ][ key type ]
    let expected = vec![0, 0, 0, 1, 0, 0, 0, 0];

    assert_eq!(key.encode().unwrap(), expected);

    let expected_key = SmtKey::decode(&expected[..]).expect("Unable to decode key");

    assert_eq!(expected_key, key);
}

#[test]
fn can_serialize_a_node_key() {
    let key = SmtKey {
        network_id: 1,
        key_type: SmtKeyType::Node(Digest([1; 32])),
    };

    //                  -> [  network ][ key type ]
    let mut expected = vec![0, 0, 0, 1, 0, 0, 0, 1];
    //                       -> [ hash ]
    expected.extend_from_slice(&[1; 32]);

    assert_eq!(key.encode().unwrap(), expected);

    let expected_key = SmtKey::decode(&expected[..]).expect("Unable to decode key");

    assert_eq!(expected_key, key);
}

#[test]
fn can_serialize_a_leaf_key() {
    let key = SmtKey {
        network_id: 1,
        key_type: SmtKeyType::Node(Digest([1; 32])),
    };

    //                  -> [  network ][ key type ]
    let mut expected = vec![0, 0, 0, 1, 0, 0, 0, 1];
    //                       -> [ hash ]
    expected.extend_from_slice(&[1; 32]);

    assert_eq!(key.encode().unwrap(), expected);

    let expected_key = SmtKey::decode(&expected[..]).expect("Unable to decode key");

    assert_eq!(expected_key, key);
}
