use agglayer_config::TlsConfig;

#[test]
fn tls_config_serialization() {
    let config = TlsConfig {
        certificate: "/path/to/cert.pem".into(),
        key: "/path/to/key.pem".into(),
    };

    insta::assert_toml_snapshot!(config);
}
