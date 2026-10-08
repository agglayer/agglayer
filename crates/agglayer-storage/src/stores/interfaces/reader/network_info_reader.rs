use agglayer_types::{CertificateId, Height, NetworkId, NetworkInfo};

use crate::error::Error;

pub trait NetworkInfoReader: Send + Sync {
    /// Read all settled fields, including legacy cursor/header fallbacks, from
    /// one database snapshot. Optional aggregates absent from that settlement's
    /// record stay absent; the live network state may have advanced already.
    ///
    /// A settled claim the record does not hold reads as no claim. That is only
    /// the truth for a network that never settled an imported bridge exit, so
    /// the record has to be complete: settlement keeps it complete, and a
    /// database migration is what makes it complete to begin with.
    fn get_network_info(&self, network_id: NetworkId) -> Result<NetworkInfo, Error>;

    fn get_latest_pending_height(&self, network_id: NetworkId) -> Result<Option<Height>, Error>;

    fn get_latest_pending_certificate_id(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<CertificateId>, Error>;

    fn get_latest_proven_certificate_id(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<CertificateId>, Error>;

    fn get_latest_settled_certificate_id(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<CertificateId>, Error>;
}
