use super::*;

#[test]
fn pending_certificate_id_is_omitted_only_when_absent() {
    let mut network_info = NetworkInfo::from_network_id(NetworkId::new(1));

    let value = serde_json::to_value(&network_info).expect("network info must serialize");
    assert_eq!(value.get("latest_pending_certificate_id"), None);

    let certificate_id = CertificateId::new(Digest::from([1; 32]));
    network_info.latest_pending_certificate_id = Some(certificate_id);

    let value = serde_json::to_value(&network_info).expect("network info must serialize");
    assert_eq!(
        value.get("latest_pending_certificate_id"),
        Some(&serde_json::to_value(certificate_id).expect("certificate ID must serialize"))
    );
}
