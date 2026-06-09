//! Shared building blocks for the certificate proto-CF migrations.
//!
//! The three certificate-bearing stores (pending, per-epoch, debug) all run
//! the same shape of backfill at startup: iterate the legacy CF, decode
//! each row through [`crate::types::LegacyCertificate`] (which accepts both
//! historical bincode and proto), and write the resulting
//! [`agglayer_types::Certificate`] into the proto CF.
//!
//! This module provides one helper, [`copy_legacy_certificate_cf_into_proto`],
//! that performs that loop with two operational properties:
//!
//! * **Streaming:** rows are read and written one at a time; no `Vec`
//!   accumulation, so migration memory stays bounded regardless of CF size.
//! * **Skip-with-log on codec failure:** if a single legacy row cannot be
//!   decoded as either bincode or proto, the row is logged via
//!   `tracing::error!` (with the CF name and key) and skipped, rather than
//!   aborting the migration step. The row is also unreadable through the
//!   current runtime codec, so skipping it does not make data inaccessible that
//!   was previously accessible. Non-codec errors (rocksdb I/O, schema problems)
//!   still propagate and abort the step.
//!
//! Per-CF and per-key details land in the structured `tracing` events; a
//! `tracing::warn!` summary is emitted at the end if any rows were skipped,
//! so operators see the impact even if they only have warn-level logging
//! enabled.

use agglayer_types::Certificate;
use tracing::{debug, error, info, warn};

use crate::{
    schema::ColumnSchema,
    storage::{DBError, DBMigrationErrorDetails, DbAccess},
    types::LegacyCertificate,
};

/// Stream every row in the legacy certificate CF `L` into the proto CF `P`,
/// skipping rows whose bytes cannot be decoded as a certificate.
///
/// `label` is a short human tag (e.g. `"pending"`, `"epoch"`, `"debug"`) used
/// in the structured log fields and the summary message.
pub(crate) fn copy_legacy_certificate_cf_into_proto<L, P>(
    db: &DbAccess,
    label: &str,
) -> Result<(), DBMigrationErrorDetails>
where
    L: ColumnSchema<Value = LegacyCertificate>,
    P: ColumnSchema<Value = Certificate, Key = L::Key>,
    L::Key: std::fmt::Debug,
{
    let mut migrated = 0_usize;
    let mut skipped = 0_usize;

    for key in db.keys::<L>()? {
        let key = key?;
        match db.get::<L>(&key) {
            Ok(Some(legacy)) => {
                let certificate = Certificate::from(legacy);
                db.put::<P>(&key, &certificate)?;
                migrated += 1;
            }
            Ok(None) => {
                // Race-y: the key was visible to the iterator but has gone
                // by the time we try to read it. Should not happen during
                // normal startup migrations, but log just in case.
                debug!(
                    cf = L::COLUMN_FAMILY_NAME,
                    label,
                    ?key,
                    "legacy CF key vanished between iterate and read"
                );
            }
            Err(DBMigrationErrorDetails::Database(DBError::CodecError(codec_error))) => {
                error!(
                    cf = L::COLUMN_FAMILY_NAME,
                    label,
                    ?key,
                    error = %codec_error,
                    "skipping unparsable legacy row during proto migration; this row is also \
                     unreadable through the current runtime codec",
                );
                skipped += 1;
            }
            Err(other) => return Err(other),
        }
    }

    if skipped > 0 {
        warn!(
            cf = L::COLUMN_FAMILY_NAME,
            label,
            migrated,
            skipped,
            "completed proto migration with skipped rows; investigate skipped rows in the legacy \
             CF before they are dropped in the follow-up cleanup",
        );
    } else {
        info!(
            cf = L::COLUMN_FAMILY_NAME,
            label, migrated, "completed proto migration",
        );
    }
    Ok(())
}
