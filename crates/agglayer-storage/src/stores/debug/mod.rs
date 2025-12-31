use std::{path::Path, sync::Arc};

use agglayer_types::{Certificate, CertificateId};

use super::interfaces::{reader::DebugReader, writer::DebugWriter};
use crate::{columns::debug_certificates::DebugCertificatesColumn, error::Error, storage::Db};

mod cf_definitions;

pub enum DebugStore {
    Enabled(EnabledDebugStore),
    Disabled,
}

/// A logical store for debug.
#[derive(Clone)]
pub struct EnabledDebugStore {
    db: Arc<Db>,
}

impl DebugStore {
    pub fn init_db(path: &Path) -> Result<Db, crate::storage::DbOpenError> {
        Db::open_cf(path, cf_definitions::debug_db_cf_definitions())
    }

    pub fn new(db: Arc<Db>) -> Self {
        Self::Enabled(EnabledDebugStore { db })
    }

    pub fn new_with_path(path: &Path) -> Result<Self, crate::storage::DbOpenError> {
        let db = Arc::new(Self::init_db(path)?);
        Ok(Self::new(db))
    }
}

impl DebugReader for DebugStore {
    fn get_certificate(
        &self,
        certificate_id: &CertificateId,
    ) -> Result<Option<Certificate>, Error> {
        match self {
            DebugStore::Enabled(store) => {
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
                .put::<DebugCertificatesColumn>(&certificate.hash(), certificate)?),
            DebugStore::Disabled => Ok(()),
        }
    }
}
