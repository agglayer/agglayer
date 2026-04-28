use std::{path::Path, sync::Arc};

use agglayer_types::{Certificate, CertificateId};

use super::interfaces::{reader::DebugReader, writer::DebugWriter};
use crate::{
    columns::debug_certificates::{DebugCertificatesColumn, DebugCertificatesProtoColumn},
    error::Error,
    storage::DB,
};

mod cf_definitions;

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
        DB::builder(path, cf_definitions::DEBUG_DB_V0)?
            .add_cfs(
                cf_definitions::DEBUG_CERTIFICATE_PROTO_CFS,
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

/// Migration step for the certificate serialization switch from legacy bincode
/// rows to the proto-backed debug CF.
///
/// Keep the source CF intact and copy every existing row into the new CF so the
/// rollout remains reversible until the legacy family is intentionally dropped.
fn backfill_debug_certificates_proto_from_legacy_bincode(
    db: &crate::storage::DbAccess,
) -> Result<(), crate::storage::DBMigrationErrorDetails> {
    let keys = db
        .keys::<DebugCertificatesColumn>()?
        .collect::<Result<Vec<_>, _>>()?;

    for key in keys {
        if let Some(certificate) = db.get::<DebugCertificatesColumn>(&key)? {
            db.put::<DebugCertificatesProtoColumn>(&key, &certificate)?;
        }
    }

    Ok(())
}

impl DebugReader for DebugStore {
    fn get_certificate(
        &self,
        certificate_id: &CertificateId,
    ) -> Result<Option<Certificate>, Error> {
        match self {
            DebugStore::Enabled(store) => {
                if let Some(certificate) = store
                    .db
                    .get::<DebugCertificatesProtoColumn>(certificate_id)?
                {
                    return Ok(Some(certificate));
                }

                Ok(store.db.get::<DebugCertificatesColumn>(certificate_id)?)
            }
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
