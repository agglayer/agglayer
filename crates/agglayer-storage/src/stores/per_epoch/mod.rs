use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::{
    error::Error,
    storage::{epochs_db_cf_definitions, DB},
};

/// A logical store for an Epoch.
pub struct PerEpochStore<P> {
    #[allow(dead_code)]
    db: ArcSwap<DB>,
    _phantom: std::marker::PhantomData<P>,
}

impl<P> PerEpochStore<P> {
    pub fn open(config: Arc<agglayer_config::Config>, epoch_number: u64) -> Result<Self, Error> {
        // TODO: refactor this
        let path = config.storage.epochs_path.join(format!("{}", epoch_number));

        let db = ArcSwap::new(Arc::new(DB::open_cf(&path, epochs_db_cf_definitions())?));

        Ok(Self {
            db,
            _phantom: std::marker::PhantomData,
        })
    }
}
