use agglayer_types::{CertificateId, Height, NetworkId, NetworkInfo};

use crate::error::Error;

pub trait NetworkInfoReader: Send + Sync {
    fn get_network_info(&self, network_id: NetworkId) -> Result<NetworkInfo, Error>;

    fn get_latest_pending_height(&self, network_id: NetworkId) -> Result<Option<Height>, Error>;

    fn get_latest_settled_certificate_id(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<CertificateId>, Error>;
}
