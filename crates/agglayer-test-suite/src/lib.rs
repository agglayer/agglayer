mod storage;

use std::{path::Path, sync::Arc};

use agglayer_config::Config;
pub use pessimistic_proof_test_suite::forest::Forest;

pub mod sample_data {
    pub use pessimistic_proof_test_suite::sample_data::*;
}
pub use storage::StorageContext;

pub async fn new_node() {
    todo!()
}

pub fn new_storage(path: &Path) -> StorageContext {
    let config = get_default_config(path);

    StorageContext::new_with_config(Arc::new(config))
}

pub fn get_default_config(path: &Path) -> Config {
    Config::new(path)
}
