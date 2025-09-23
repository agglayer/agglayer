//! Gives access to pre-generated TLS certificates for testing.

/// Path to the CA certificate.
pub const CA_CERT_PATH: &str = concat!(env!("OUT_DIR"), "/ca.cert.pem");

/// Path to the server certificate.
pub const SERVER_CERT_PATH: &str = concat!(env!("OUT_DIR"), "/server.cert.pem");

/// Path to the server private key.
pub const SERVER_KEY_PATH: &str = concat!(env!("OUT_DIR"), "/server.key.pem");

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::*;

    #[rstest::rstest]
    #[case(CA_CERT_PATH)]
    #[case(SERVER_CERT_PATH)]
    #[case(SERVER_KEY_PATH)]
    fn cert_file_exists(#[case] path: &str) {
        assert!(Path::new(path).is_file(), "File not found: {path}");
    }
}
