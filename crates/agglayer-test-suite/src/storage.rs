use std::sync::Arc;

use agglayer_config::Config;
use agglayer_storage::{
    backup::BackupClient,
    stores::{debug::DebugStore, pending::PendingStore, state::StateStore},
};

pub struct StorageContext {
    pub state: Arc<StateStore>,
    pub pending: Arc<PendingStore>,
    pub debug: Arc<DebugStore>,
}

impl StorageContext {
    pub fn new_with_config(config: Arc<Config>) -> Self {
        let state = Arc::new(
            StateStore::new_with_path(&config.storage.state_db_path, BackupClient::noop()).unwrap(),
        );
        let pending =
            Arc::new(PendingStore::new_with_path(&config.storage.pending_db_path).unwrap());
        let debug = if config.debug_mode {
            Arc::new(DebugStore::new_with_path(&config.storage.debug_db_path).unwrap())
        } else {
            Arc::new(DebugStore::Disabled)
        };

        Self {
            state,
            pending,
            debug,
        }
    }
}
