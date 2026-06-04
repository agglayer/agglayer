# Ensure `ensure_cfs` Migration Recording Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task.
> Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `Builder::ensure_cfs()` always persist a migration record,
including the no-op path, so future migration steps cannot create a
`MigrationRecordGap`.

**Architecture:** Keep `ensure_cfs()` as a normal numbered migration step.
Route all paths through `perform_step()`, including the already-present-CF
case.
Prove the invariant with a focused migration test, then update the rustdoc and
existing state-store comments to match the new recorded no-op behavior.

**Tech Stack:** Rust, RocksDB, `cargo test`, existing `agglayer-storage`
migration test helpers.

---

## File Map

- Modify: `crates/agglayer-storage/src/storage/migration/mod.rs`
  Purpose: make `ensure_cfs()` always execute via `perform_step()` and update
  rustdoc.
- Modify: `crates/agglayer-storage/src/storage/migration/test/corrupt.rs`
  Purpose: add the regression test that proves a no-op `ensure_cfs()` followed
  by a later step reopens cleanly.
- Modify: `crates/agglayer-storage/src/storage/migration/test/sample.rs`
  Purpose: add a tiny helper migration that uses `ensure_cfs()` and then a later
  step so the regression test can express the scenario with existing sample
  schema types.
- Modify: `crates/agglayer-storage/src/stores/state/tests.rs`
  Purpose: update comments so the idempotence test describes a recorded no-op,
  not an unrecorded one.

### Task 1: Add the Regression Test First

**Files:**
- Modify: `crates/agglayer-storage/src/storage/migration/test/sample.rs`
- Modify: `crates/agglayer-storage/src/storage/migration/test/corrupt.rs`
- Test: `crates/agglayer-storage/src/storage/migration/test/corrupt.rs`

- [ ] **Step 1: Write the failing test helper migration in `sample.rs`**

Add a helper that first runs `ensure_cfs(CFS_V1)` and then adds a distinct later
step.
Use a new sample CF so the second step is definitely recorded after the no-op.

```rust
pub struct EnsureCfsSentinelColumn;

impl ColumnSchema for EnsureCfsSentinelColumn {
    type Key = KeyV1;
    type Value = NetworkInfoV1;

    const COLUMN_FAMILY_NAME: &'static str = "ensure_cfs_sentinel";
}

pub const CFS_SENTINEL: &[ColumnDescriptor] =
    &[ColumnDescriptor::new::<EnsureCfsSentinelColumn>()];

impl Builder {
    pub fn sample_noop_ensure_cfs_then_add_step(self) -> Result<Self, DBOpenError> {
        self.ensure_cfs(CFS_V1)?.add_cfs(CFS_SENTINEL, |_db| Ok(()))
    }
}
```

- [ ] **Step 2: Write the failing regression test in `corrupt.rs`**

Add a test that creates a DB whose schema already includes `CFS_V1`, then reopens
through the helper migration above.
Before the fix, reopen after recording the later step should fail with
`MigrationRecordGap(1)` because `ensure_cfs(CFS_V1)` consumed step 1 without a
record.

```rust
#[test_log::test]
fn noop_ensure_cfs_does_not_create_migration_gap() -> Result<(), eyre::Error> {
    let temp_dir = TempDBDir::new();
    let db_path = &temp_dir.path;

    {
        let db = Builder::open_sample(db_path)?
            .sample_migrate_v0_v1()?
            .finalize(CFS_V1)?;
        drop(db);
    }

    {
        let db = Builder::open_sample(db_path)?
            .sample_migrate_v0_v1()?
            .sample_noop_ensure_cfs_then_add_step()?
            .finalize(&[CFS_V1, CFS_SENTINEL].concat())?;
        drop(db);
    }

    let reopen = Builder::open_sample(db_path)?
        .sample_migrate_v0_v1()?
        .sample_noop_ensure_cfs_then_add_step()?
        .finalize(&[CFS_V1, CFS_SENTINEL].concat());

    assert!(reopen.is_ok());
    Ok(())
}
```

- [ ] **Step 3: Run the new regression test and confirm it fails first**

Run:

```bash
cargo test -p agglayer-storage noop_ensure_cfs_does_not_create_migration_gap -- --nocapture
```

Expected before the fix: FAIL with `MigrationRecordGap(1)` during the final
reopen.

- [ ] **Step 4: Commit the failing test scaffold**

```bash
git add crates/agglayer-storage/src/storage/migration/test/sample.rs \
  crates/agglayer-storage/src/storage/migration/test/corrupt.rs
git commit -m "test: cover ensure_cfs no-op migration gaps"
```

### Task 2: Fix `ensure_cfs` and Align Docs

**Files:**
- Modify: `crates/agglayer-storage/src/storage/migration/mod.rs`
- Modify: `crates/agglayer-storage/src/stores/state/tests.rs`
- Test: `crates/agglayer-storage/src/storage/migration/test/corrupt.rs`

- [ ] **Step 1: Route `ensure_cfs()` through `perform_step()` for all paths**

Replace the early return with a single `perform_step()` closure.
Compute `missing_cfs` inside the closure against the migration builder state,
then either log-and-return or create the missing CFs.

```rust
pub fn ensure_cfs(self, cfs: &[ColumnDescriptor]) -> Result<Self, DBOpenError> {
    Ok(self.perform_step(move |db| {
        let missing_cfs: Vec<_> = cfs
            .iter()
            .filter(|descriptor| !db.cf_exists(descriptor.name()))
            .cloned()
            .collect();

        if missing_cfs.is_empty() {
            debug!("All requested column families already exist");
            return Ok(());
        }

        for descriptor in &missing_cfs {
            let cf = descriptor.name();
            let opts = DB::options(descriptor.options());
            debug!("Creating missing column family {cf:?}");
            db.db.rocksdb.create_cf(cf, &opts).map_err(DBError::from)?;
        }

        Ok(())
    })?)
}
```

- [ ] **Step 2: Update the rustdoc and state-store comment text**

Adjust the `ensure_cfs` rustdoc so it documents a recorded no-op migration step.
Update the state-store idempotence test comment so it says the second open is a
recorded no-op rather than a skipped record.

```rust
/// When all requested CFs are already present, the step becomes an idempotent
/// no-op that is still recorded in migration history.
/// Recording the no-op preserves contiguous migration records for future steps.
```

```rust
// Opening init_db twice on a fresh DB must succeed: the second call
// sees every CF already present, records the ensure_cfs step, and
// leaves the schema unchanged.
```

- [ ] **Step 3: Run the targeted tests and verify they pass**

Run:

```bash
cargo test -p agglayer-storage noop_ensure_cfs_does_not_create_migration_gap -- --nocapture
cargo test -p agglayer-storage init_db_is_idempotent_on_current_schema -- --nocapture
```

Expected after the fix: PASS for both tests.

- [ ] **Step 4: Run the full migration test module as a safety check**

Run:

```bash
cargo test -p agglayer-storage storage::migration::test -- --nocapture
```

Expected: PASS with no `MigrationRecordGap` regressions.

- [ ] **Step 5: Commit the implementation**

```bash
git add crates/agglayer-storage/src/storage/migration/mod.rs \
  crates/agglayer-storage/src/storage/migration/test/sample.rs \
  crates/agglayer-storage/src/storage/migration/test/corrupt.rs \
  crates/agglayer-storage/src/stores/state/tests.rs
git commit -m "fix: record no-op ensure_cfs migrations"
```
