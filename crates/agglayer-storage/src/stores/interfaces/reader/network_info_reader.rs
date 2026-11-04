use std::collections::BTreeMap;

use agglayer_types::{CertificateHeader, NetworkId, NetworkInfo};

use crate::error::Error;

/// The headers of a network's latest pending, proven, and settled
/// certificates, read from one database snapshot.
///
/// A cached pointer always resolves to its header: a pointer without one is
/// inconsistent storage and fails the read, it never reads as absent.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LatestCertificateHeaders {
    pub pending: Option<CertificateHeader>,
    pub proven: Option<CertificateHeader>,
    pub settled: Option<CertificateHeader>,
}

impl LatestCertificateHeaders {
    /// The header at the greatest height, the later lifecycle stage winning
    /// a tie.
    pub fn latest_known(&self) -> Option<&CertificateHeader> {
        [&self.pending, &self.proven, &self.settled]
            .into_iter()
            .flatten()
            .max_by_key(|header| header.height)
    }
}

pub trait NetworkInfoReader: Send + Sync {
    /// Read every field of a network's info from one database snapshot: the
    /// cached rows, and the headers of the pending and settled certificates
    /// they point to. The proven header is not read, the info serves no
    /// proven field.
    ///
    /// `None` is a network without any cached row, which is unknown.
    /// A pending or settled pointer without its header is inconsistent
    /// storage and fails the read. The pending fields are the referenced
    /// header's, even once that certificate has settled: a caller serving
    /// them as pending has to filter that out.
    /// Optional aggregates absent from the settled row stay absent; the live
    /// network state may have advanced already.
    ///
    /// A settled claim the record does not hold reads as no claim. That is only
    /// the truth for a network that never settled an imported bridge exit, so
    /// the record has to be complete: settlement keeps it complete, and a
    /// database migration is what makes it complete to begin with.
    fn get_network_info(&self, network_id: NetworkId) -> Result<Option<NetworkInfo>, Error>;

    /// The headers of the network's latest pending, proven, and settled
    /// certificates, from one database snapshot. A pointer without its header
    /// fails the read, whichever of the three it is. The pending header may
    /// be a certificate that has settled since: a caller serving it as
    /// pending has to filter that out.
    fn get_latest_certificate_headers(
        &self,
        network_id: NetworkId,
    ) -> Result<LatestCertificateHeaders, Error>;

    /// The same for every network with a cached row this binary knows, from
    /// one snapshot.
    ///
    /// One scan of the cache and the headers it points to: O(number of
    /// networks). Fails on the first undecodable pending, proven, or settled
    /// pointer row, or pointer without a header: this scan seeds startup
    /// state, where a skipped entry would stay missing until the next
    /// restart. A row of a kind this binary does not know is skipped with a
    /// warning, and the type and claim values are not interpreted.
    fn get_all_latest_certificate_headers(
        &self,
    ) -> Result<BTreeMap<NetworkId, LatestCertificateHeaders>, Error>;
}
