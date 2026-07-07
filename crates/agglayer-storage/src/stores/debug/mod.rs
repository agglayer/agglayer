use std::{path::Path, sync::Arc};

use agglayer_types::{Certificate, CertificateId};

use super::interfaces::{reader::DebugReader, writer::DebugWriter};
use crate::{
    columns::debug_certificates::{DebugCertificatesColumn, DebugCertificatesProtoColumn},
    error::Error,
    schema::ColumnDescriptor,
    storage::DB,
};

pub(crate) mod cf_definitions;

#[cfg(test)]
mod tests;

pub enum DebugStore {
    Enabled(EnabledDebugStore),
    Disabled,
}

/// A logical store for debug.
#[derive(Clone)]
pub struct EnabledDebugStore {
    db: Arc<DB>,
}

impl DebugStore {
    pub fn init_db(path: &Path) -> Result<DB, crate::storage::DBOpenError> {
        DB::builder(path, cf_definitions::DEBUG_DB_V0, cf_definitions::DEBUG_DB)?
            .add_cfs(
                &[ColumnDescriptor::new::<DebugCertificatesProtoColumn>()],
                backfill_debug_certificates_proto_from_legacy_bincode,
            )?
            .finalize(cf_definitions::DEBUG_DB)
    }

    pub fn new(db: Arc<DB>) -> Self {
        Self::Enabled(EnabledDebugStore { db })
    }

    pub fn new_with_path(path: &Path) -> Result<Self, crate::storage::DBOpenError> {
        let db = Arc::new(Self::init_db(path)?);
        Ok(Self::new(db))
    }
}

/// Migration step for the certificate serialization switch from the legacy
/// debug CF to the proto-backed CF.
///
/// Delegates to
/// [`super::migration_helpers::copy_legacy_certificate_cf_into_proto`],
/// which streams the legacy keyspace, skips and logs rows whose bytes cannot
/// be decoded as a certificate, and copies the rest into the proto CF. The
/// source CF is left intact so the rollout remains reversible until the
/// legacy family is intentionally dropped.
fn backfill_debug_certificates_proto_from_legacy_bincode(
    db: &crate::storage::DbAccess,
) -> Result<(), crate::storage::DBMigrationErrorDetails> {
    super::migration_helpers::copy_legacy_certificate_cf_into_proto::<
        DebugCertificatesColumn,
        DebugCertificatesProtoColumn,
    >(db, "debug")
}

impl DebugReader for DebugStore {
    fn get_certificate(
        &self,
        certificate_id: &CertificateId,
    ) -> Result<Option<Certificate>, Error> {
        match self {
            DebugStore::Enabled(store) => Ok(store
                .db
                .get::<DebugCertificatesProtoColumn>(certificate_id)?),
            DebugStore::Disabled => Ok(None),
        }
    }
}

impl DebugWriter for DebugStore {
    fn add_certificate(&self, certificate: &Certificate) -> Result<(), Error> {
        match self {
            DebugStore::Enabled(store) => Ok(store
                .db
                .put::<DebugCertificatesProtoColumn>(&certificate.hash(), certificate)?),
            DebugStore::Disabled => Ok(()),
        }
    }
}
