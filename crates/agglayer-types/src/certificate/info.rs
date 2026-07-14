use std::ops::Range;

use unified_bridge::NetworkId;

use super::{CertificateId, CertificateStatus, Height};
use crate::U256;

/// Projection of a certificate binding it to its bridge exits and claims.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CertificateInfo {
    pub certificate_id: CertificateId,
    pub network_id: NetworkId,
    pub height: Height,
    /// Any status other than `Settled` means the binding is provisional: the
    /// certificate may still be replaced at the same height.
    pub status: CertificateStatus,
    /// Number of bridge exits, i.e. local exit tree leaves appended by this
    /// certificate.
    pub exit_count: u32,
    /// Absolute local exit tree leaf range covered by the bridge exits.
    pub leaf_range: Range<u32>,
    /// Global indexes of the imported bridge exits (claims), when requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claims: Option<Vec<U256>>,
}
