use std::{path::Path, sync::Arc};

use agglayer_types::{Certificate, CertificateId};

use super::interfaces::{reader::DebugReader, writer::DebugWriter};
use crate::{columns::debug_certificates::DebugCertificatesColumn, error::Error, storage::DB};

pub enum DebugStore {
    Enabled(EnabledDebugStore),
    Disabled,
}

/// A logical store for debug.
#[derive(Debug, Clone)]
pub struct EnabledDebugStore {
    db: Arc<DB>,
}

impl DebugStore {
    pub fn new(db: Arc<DB>) -> Self {
        Self::Enabled(EnabledDebugStore { db })
    }

    pub fn new_with_path(path: &Path) -> Result<Self, Error> {
        let db = Arc::new(DB::open_cf(
            path,
            crate::storage::debug_db_cf_definitions(),
        )?);

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
