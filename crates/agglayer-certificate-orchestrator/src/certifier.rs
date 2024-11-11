use agglayer_types::LocalNetworkStateData;
use agglayer_types::{Certificate, Height, NetworkId};
use futures_util::future::BoxFuture;

use crate::error::{CertificationError, PreCertificationError};

pub trait CertificateInput: Clone {
    fn network_id(&self) -> NetworkId;
}

impl CertificateInput for Certificate {
    fn network_id(&self) -> NetworkId {
        self.network_id
    }
}

#[derive(Debug, Clone)]
pub struct CertifierOutput {
    pub certificate: Certificate,
    pub height: Height,
    pub new_state: LocalNetworkStateData,
    pub network: NetworkId,
}

pub type CertifierResult =
    Result<BoxFuture<'static, Result<CertifierOutput, CertificationError>>, PreCertificationError>;

/// Apply one Certificate on top of a local state and computes one proof.
pub trait Certifier: Unpin + Send + Sync + 'static {
    fn certify(
        &self,
        full_state: LocalNetworkStateData,
        network_id: NetworkId,
        height: Height,
    ) -> CertifierResult;
}
