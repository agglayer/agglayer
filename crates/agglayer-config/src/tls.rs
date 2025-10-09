use std::{io, path::PathBuf};

use axum_server::tls_rustls::RustlsConfig;

/// Configuration of TLS certificates.
#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TlsConfig {
    pub certificate: PathBuf,
    pub key: PathBuf,
}

impl TlsConfig {
    pub async fn to_rustls_config(&self) -> io::Result<RustlsConfig> {
        RustlsConfig::from_pem_file(&self.certificate, &self.key).await
    }
}
