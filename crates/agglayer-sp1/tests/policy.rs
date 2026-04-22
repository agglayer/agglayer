use agglayer_sp1::{AcceptancePolicy, ProofError, Sp1ProofVersion};

#[test]
fn default_reads_v5_and_v6() {
    AcceptancePolicy::DEFAULT
        .ensure_readable(Sp1ProofVersion::V5, "v5.2.2")
        .unwrap();
    AcceptancePolicy::DEFAULT
        .ensure_readable(Sp1ProofVersion::V6, "v6.0.1")
        .unwrap();
}

#[test]
fn default_rejects_v6_writes() {
    let err = AcceptancePolicy::DEFAULT
        .ensure_writable(Sp1ProofVersion::V6, "v6.0.1")
        .unwrap_err();
    assert!(matches!(
        err,
        ProofError::UnsupportedWritableSp1Version { .. }
    ));
}

#[test]
fn default_rejects_v6_executes() {
    let err = AcceptancePolicy::DEFAULT
        .ensure_executable(Sp1ProofVersion::V6, "v6.0.1")
        .unwrap_err();
    assert!(matches!(
        err,
        ProofError::UnsupportedExecutableSp1Version { .. }
    ));
}
