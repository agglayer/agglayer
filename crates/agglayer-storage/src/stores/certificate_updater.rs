//! Publish legacy and cache pointers using trusted storage and proof inputs.
//! Admin callers must stop normal writes before using these helpers.

use agglayer_types::{Certificate, CertificateId, CertificateStatus, Height, NetworkId};
use eyre::WrapErr;

use super::{PendingCertificateWriter, StateReader, StateWriter};
use crate::error::Error;

#[cfg(test)]
mod tests;

/// Persist the body and legacy pointer, then publish the header and cache.
/// Normal submission uses `StateWriter::submit_pending_certificate`.
pub fn submit(
    pending: &impl PendingCertificateWriter,
    state: &(impl StateReader + StateWriter),
    certificate: &Certificate,
    status: &CertificateStatus,
) -> eyre::Result<()> {
    if state
        .get_certificate_header(&certificate.hash())
        .wrap_err("certificate header")?
        .is_some_and(|header| header.status == CertificateStatus::Settled)
    {
        return Err(Error::UnprocessedAction(
            "Cannot change status of a settled certificate".into(),
        )
        .into());
    }
    pending
        .insert_pending_certificate(certificate.network_id, certificate.height, certificate)
        .wrap_err("legacy pending pointer and certificate body")?;
    state
        .publish_pending_certificate(certificate, status)
        .wrap_err("state pending cache")
}

/// Publish an administrative pending override without changing header status.
pub fn set_pending(
    pending: &impl PendingCertificateWriter,
    state: &impl StateWriter,
    network_id: NetworkId,
    height: Height,
    id: CertificateId,
) -> eyre::Result<()> {
    state
        .set_latest_pending_certificate_pointer(network_id, height, id)
        .wrap_err("state pending cache")?;
    pending
        .set_latest_pending_certificate_per_network(&network_id, &height, &id)
        .wrap_err("legacy pending pointer")
}

/// Publish a proven pointer without changing header status.
pub fn set_proven(
    pending: &impl PendingCertificateWriter,
    state: &impl StateWriter,
    network_id: NetworkId,
    height: Height,
    id: CertificateId,
) -> eyre::Result<()> {
    state
        .set_latest_proven_certificate_pointer(network_id, id)
        .wrap_err("state proven cache")?;
    pending
        .set_latest_proven_certificate_per_network(&network_id, &height, &id)
        .wrap_err("legacy proven pointer")
}
