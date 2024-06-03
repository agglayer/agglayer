use std::sync::Arc;

use super::{PendingCertificateReader, PendingCertificateWriter};
use crate::storage::DB;

/// A logical store for pending.
#[derive(Debug, Clone)]
pub struct PendingStore {
    #[allow(unused)]
    db: Arc<DB>,
}

impl PendingStore {
    pub fn new(db: Arc<DB>) -> Self {
        Self { db }
    }
}

impl PendingCertificateWriter for PendingStore {}
impl PendingCertificateReader for PendingStore {}
