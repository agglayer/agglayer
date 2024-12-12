use std::sync::Arc;

use agglayer_config::Config;
use agglayer_storage::{
    storage::{pending_db_cf_definitions, state_db_cf_definitions, DB},
    stores::{debug::DebugStore, pending::PendingStore, state::StateStore},
};

pub struct StorageContext {
    pub state: Arc<StateStore>,
    pub pending: Arc<PendingStore>,
    pub debug: Arc<DebugStore>,
}

impl StorageContext {
    pub fn new_with_config(config: Arc<Config>) -> Self {
        let state_db = Arc::new(
            DB::open_cf(&config.storage.state_db_path, state_db_cf_definitions()).unwrap(),
        );
        let pending_db = Arc::new(
            DB::open_cf(&config.storage.pending_db_path, pending_db_cf_definitions()).unwrap(),
        );

        let state = Arc::new(StateStore::new(state_db));
        let pending = Arc::new(PendingStore::new(pending_db));
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
