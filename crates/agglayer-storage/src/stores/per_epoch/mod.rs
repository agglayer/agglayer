use arc_swap::ArcSwap;

use crate::storage::DB;

/// A logical store for an Epoch.
pub struct PerEpochStore<P> {
    #[allow(dead_code)]
    db: ArcSwap<DB>,
    _phantom: std::marker::PhantomData<P>,
}
