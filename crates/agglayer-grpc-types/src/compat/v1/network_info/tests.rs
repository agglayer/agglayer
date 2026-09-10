use agglayer_types::{CertificateId, Digest, NetworkId, NetworkInfo};

use super::*;

#[test]
fn pending_certificate_id_is_preserved() {
    let certificate_id = CertificateId::new(Digest::from([1u8; 32]));
    let network_info = NetworkInfo {
        latest_pending_certificate_id: Some(certificate_id),
        ..NetworkInfo::from_network_id(NetworkId::new(1))
    };

    let grpc_network_info = v1::NetworkInfo::from(network_info);

    assert_eq!(
        grpc_network_info.latest_pending_certificate_id,
        Some(certificate_id.into())
    );
}

#[test]
fn absent_pending_certificate_id_stays_absent() {
    let network_info = NetworkInfo::from_network_id(NetworkId::new(1));

    let grpc_network_info = v1::NetworkInfo::from(network_info);

    assert_eq!(grpc_network_info.latest_pending_certificate_id, None);
}
