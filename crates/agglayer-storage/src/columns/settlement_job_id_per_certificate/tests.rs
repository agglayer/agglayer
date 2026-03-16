use agglayer_types::CertificateId;
use ulid::Ulid;

use crate::{
    schema::Codec as _,
    types::settlement::job_id_per_certificate::{Key, Value},
};

#[test]
fn settlement_job_id_per_certificate_roundtrip_codec() {
    let key = CertificateId::for_test(0x11);
    let value = Ulid::from(42u128);

    let encoded_key = key.encode().expect("Unable to encode key");
    let decoded_key = Key::decode(&encoded_key).expect("Unable to decode key");
    assert_eq!(decoded_key, key);

    let encoded_value = value.encode().expect("Unable to encode value");
    let decoded_value = Value::decode(&encoded_value).expect("Unable to decode value");
    assert_eq!(decoded_value, value);
}
