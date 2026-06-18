use std::sync::{Mutex, MutexGuard};

mod corrupt;
mod happy_path;
mod recovery;
pub mod sample;
mod schema_checks;

static SAMPLE_MIGRATION_TEST_LOCK: Mutex<()> = Mutex::new(());

fn lock_sample_migration_tests() -> MutexGuard<'static, ()> {
    match SAMPLE_MIGRATION_TEST_LOCK.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
