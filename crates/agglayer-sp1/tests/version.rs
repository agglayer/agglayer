use agglayer_sp1::{version_kind, ProofError, Sp1ProofVersion};

#[test]
fn parses_v5_major() {
    assert_eq!(version_kind("v5.2.2").unwrap(), Sp1ProofVersion::V5);
    assert_eq!(version_kind("5.2.2").unwrap(), Sp1ProofVersion::V5);
    assert_eq!(version_kind("v4.1.0").unwrap(), Sp1ProofVersion::V5);
}

#[test]
fn parses_v6_major() {
    assert_eq!(version_kind("v6.1.0").unwrap(), Sp1ProofVersion::V6);
    assert_eq!(version_kind("6.1.0").unwrap(), Sp1ProofVersion::V6);
}

#[test]
fn rejects_invalid_version() {
    assert!(matches!(
        version_kind("").unwrap_err(),
        ProofError::InvalidSp1Version { .. }
    ));
    assert!(matches!(
        version_kind("abc").unwrap_err(),
        ProofError::InvalidSp1Version { .. }
    ));
}

#[test]
fn rejects_unsupported_major() {
    assert!(matches!(
        version_kind("v7.0.0").unwrap_err(),
        ProofError::UnsupportedSp1VersionMajor { .. }
    ));
}
