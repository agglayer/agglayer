use agglayer_types::{Height, NetworkId, NetworkInfo};

use crate::{error::Error, types::BasicPendingCertificateInfo};

pub trait NetworkInfoReader: Send + Sync {
    fn get_network_info(&self, network_id: NetworkId) -> Result<NetworkInfo, Error>;

    fn get_latest_pending_height(&self, network_id: NetworkId) -> Result<Option<Height>, Error>;

    fn get_latest_pending_certificate_info(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<BasicPendingCertificateInfo>, Error>;
}
