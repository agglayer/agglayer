use agglayer_types::CertificateHeader;

use super::{load_sample_bytes, Codec};

#[rstest::rstest]
#[case("regression_header_01.hex")]
fn regressions(#[case] cert_hdr_filename: &str) {
    let bytes = load_sample_bytes(cert_hdr_filename);
    let _header = CertificateHeader::decode(&bytes).expect("decoding failed");
}
