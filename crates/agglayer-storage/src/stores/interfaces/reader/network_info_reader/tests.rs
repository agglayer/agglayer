use agglayer_types::{
    Certificate, CertificateHeader, CertificateStatus, Height, Metadata, NetworkId,
};

use super::LatestCertificateHeaders;

/// The header of a test certificate of network 1 at `height`.
fn header(height: u64, status: CertificateStatus) -> CertificateHeader {
    let certificate = Certificate::new_for_test(NetworkId::new(1), Height::new(height));
    CertificateHeader {
        network_id: certificate.network_id,
        height: certificate.height,
        epoch_number: None,
        certificate_index: None,
        certificate_id: certificate.hash(),
        prev_local_exit_root: certificate.prev_local_exit_root,
        new_local_exit_root: certificate.new_local_exit_root,
        metadata: Metadata::ZERO,
        status,
        settlement_tx_hash: None,
    }
}

/// The greatest height wins, whichever stage holds it.
#[test]
fn latest_known_is_the_header_at_the_greatest_height() {
    assert_eq!(LatestCertificateHeaders::default().latest_known(), None);

    let headers = LatestCertificateHeaders {
        pending: Some(header(6, CertificateStatus::Pending)),
        proven: None,
        settled: Some(header(5, CertificateStatus::Settled)),
    };
    assert_eq!(headers.latest_known(), headers.pending.as_ref());
}

/// At equal heights the later lifecycle stage wins: settled over proven,
/// proven over pending.
#[test]
fn latest_known_prefers_the_later_stage_at_equal_heights() {
    let mut headers = LatestCertificateHeaders {
        pending: Some(header(5, CertificateStatus::Pending)),
        proven: Some(header(5, CertificateStatus::Proven)),
        settled: Some(header(5, CertificateStatus::Settled)),
    };
    assert_eq!(headers.latest_known(), headers.settled.as_ref());

    headers.settled = None;
    assert_eq!(headers.latest_known(), headers.proven.as_ref());
}
