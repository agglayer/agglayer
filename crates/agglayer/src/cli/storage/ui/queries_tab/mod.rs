use std::{path::PathBuf, sync::Arc};

#[derive(Default)]
pub(crate) struct QueriesTab {
    storage_path: Arc<PathBuf>,
}

impl QueriesTab {
    pub(crate) fn new(storage_path: Arc<PathBuf>) -> Self {
        Self {
            storage_path,
            ..Default::default()
        }
    }
}
