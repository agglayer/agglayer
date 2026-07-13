# Settlement admin floor (issue 1675) — implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task.
> Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship the settlement admin floor from issue 1675
(`admin_listSettlementJobs`, `admin_getSettlementJob`,
`admin_abortSettlementTask`, `admin_reloadAndRestartSettlementTask`)
as three small stacked PRs.

**Architecture:** PR 1 adds service and storage groundwork
(typed admin errors, `has_live_task`, a respawn-capable
reload-and-restart, the job-to-certificate reverse link dual write and
reader). PR 2 wires the concrete `SettlementService` into
`AdminAgglayerImpl` (byte-for-byte the PR 1663 plumbing shape) and
exposes the two task controls. PR 3 adds the two reads with DTOs at the
RPC boundary plus the operator documentation.
Spec: `docs/knowledge-base/src/ai-agents/specs/2026-07-13-settlement-admin-floor-design.md`.

**Tech Stack:** Rust, jsonrpsee (`#[rpc]` proc macro), tokio, thiserror,
mockall (`MockStateStore`), rstest, insta snapshots, RocksDB stores,
alloy mock providers, `cargo nextest`.

**Conventions for every task:**

- Run tests with `cargo nextest run -p <crate>`
  (the jsonrpc-api tests assert they run under nextest).
- Before each commit: `cargo +nightly fmt` on touched crates and
  `cargo clippy -p <touched crates> --all-targets -- -D warnings`.
- Git commits and pushes require explicit user approval per AGENTS.md;
  each "Commit" step below assumes that approval was given for the
  session.
- All three branches stack: PR 2 branches from PR 1's branch,
  PR 3 from PR 2's.

---

## PR 1 — service and storage groundwork

Branch: `feat/1675-settlement-admin-groundwork` (from `main`).
No RPC changes. Two crates touched: `agglayer-settlement-service`,
`agglayer-storage`.

### Task 1: `SettlementAdminError` type

**Files:**
- Create: `crates/agglayer-settlement-service/src/error.rs`
- Modify: `crates/agglayer-settlement-service/src/lib.rs`
- Modify: `crates/agglayer-settlement-service/Cargo.toml`
  (only if `thiserror` is not already a dependency; check first)

- [ ] **Step 1: Check the thiserror dependency**

Run: `grep -n "thiserror" crates/agglayer-settlement-service/Cargo.toml`
If absent, add `thiserror.workspace = true` under `[dependencies]`.

- [ ] **Step 2: Create the error module**

Create `crates/agglayer-settlement-service/src/error.rs`:

```rust
use agglayer_types::SettlementJobId;

/// Errors returned by the admin surface of the settlement service.
///
/// The variants distinguish the cases an operator must react to
/// differently: a job that does not exist, a job that is already
/// completed, and a pending job whose in-memory task is dead.
#[derive(Debug, thiserror::Error)]
pub enum SettlementAdminError {
    /// No settlement job with this id exists in storage.
    #[error("No settlement job found for id {0}")]
    JobNotFound(SettlementJobId),

    /// The job already has a terminal result recorded in storage.
    #[error("Settlement job {0} is already completed")]
    JobCompleted(SettlementJobId),

    /// The job is pending in storage but no in-memory task is running.
    /// Recover with reload-and-restart.
    #[error("No live settlement task for job {0}")]
    NoLiveTask(SettlementJobId),

    /// The live task did not accept the admin command
    /// (admin channel full or closed).
    #[error("Settlement task for job {job_id} did not accept the admin command: {reason}")]
    TaskNotResponding {
        job_id: SettlementJobId,
        reason: String,
    },

    /// Reloading the task state from storage failed.
    #[error("Failed to reload settlement task for job {job_id}: {reason}")]
    ReloadFailed {
        job_id: SettlementJobId,
        reason: String,
    },

    /// A storage read failed while classifying the job state.
    #[error("Storage error while handling settlement admin command for job {job_id}")]
    Storage {
        job_id: SettlementJobId,
        #[source]
        source: agglayer_storage::error::Error,
    },
}
```

- [ ] **Step 3: Export it from lib.rs**

In `crates/agglayer-settlement-service/src/lib.rs`, extend the module
list and re-exports:

```rust
pub mod error;
pub mod settlement_service;
pub mod settlement_service_trait;
mod settlement_task;
mod utils;

pub use error::SettlementAdminError;
pub use settlement_service::SettlementService;
#[cfg(feature = "testutils")]
pub use settlement_service_trait::MockSettlementServiceTrait;
pub use settlement_service_trait::SettlementServiceTrait;
```

- [ ] **Step 4: Compile**

Run: `cargo check -p agglayer-settlement-service`
Expected: clean build (the type is exported but unused so far;
the crate has `#![allow(dead_code)]`).

- [ ] **Step 5: Commit**

```bash
git add crates/agglayer-settlement-service
git commit -m "feat(settlement-service): add typed settlement admin error"
```

### Task 2: typed abort with storage-backed error classification

**Files:**
- Modify: `crates/agglayer-settlement-service/src/settlement_service.rs`
  (methods around lines 232-269, tower impl at 393-425,
  inline `mod tests` at the end)

- [ ] **Step 1: Write the failing tests**

Append to the inline `mod tests` in `settlement_service.rs`
(after `request_new_settlement_records_certificate_link_before_job`).
The new tests use tolerant `.returning()` mocks because the spawned
task's exact storage call sequence is an implementation detail:

```rust
    #[tokio::test]
    async fn admin_abort_task_unknown_job_returns_job_not_found() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(20);
        store
            .expect_get_settlement_job_result()
            .returning(|_| Ok(None));
        store.expect_get_settlement_job().returning(|_| Ok(None));

        let service = mk_service(Arc::new(store)).await;

        let error = service
            .admin_abort_task(job_id)
            .await
            .expect_err("abort of an unknown job must fail");
        assert!(matches!(
            error,
            crate::SettlementAdminError::JobNotFound(id) if id == job_id
        ));
    }

    #[tokio::test]
    async fn admin_abort_task_completed_job_returns_job_completed() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(21);
        let result = mk_result(21, ContractCallOutcome::Success);
        store
            .expect_get_settlement_job_result()
            .returning(move |_| Ok(Some(result.clone())));

        let service = mk_service(Arc::new(store)).await;

        let error = service
            .admin_abort_task(job_id)
            .await
            .expect_err("abort of a completed job must fail");
        assert!(matches!(
            error,
            crate::SettlementAdminError::JobCompleted(id) if id == job_id
        ));
    }

    #[tokio::test]
    async fn admin_abort_task_pending_job_without_task_returns_no_live_task() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(22);
        let job = mk_job(22);
        store
            .expect_get_settlement_job_result()
            .returning(|_| Ok(None));
        store
            .expect_get_settlement_job()
            .returning(move |_| Ok(Some(job.clone())));

        let service = mk_service(Arc::new(store)).await;

        let error = service
            .admin_abort_task(job_id)
            .await
            .expect_err("abort without a live task must fail");
        assert!(matches!(
            error,
            crate::SettlementAdminError::NoLiveTask(id) if id == job_id
        ));
    }

    /// Loads a pending job through `SettlementTask::load` and spawns its
    /// task, mirroring the reload test above. The caller provides the
    /// storage expectations for the load (job, no result, no attempts).
    async fn load_and_spawn_pending_task(
        service: &SettlementService<
            impl Provider + WalletProvider + 'static,
            MockStateStore,
        >,
        job_id: SettlementJobId,
    ) {
        let (task_control_handle, task_control) =
            TaskControlHandle::new(&service.cancellation_token);
        let task = match SettlementTask::load(
            job_id,
            service.tx_config.clone(),
            service.provider.clone(),
            service.store.clone(),
            task_control,
        )
        .await
        .expect("settlement task should load")
        {
            StoredSettlementJob::Pending(task) => task,
            StoredSettlementJob::Completed(_, _) => {
                panic!("load should find a pending job")
            }
        };
        service
            .spawn_settlement_task(job_id, task, task_control_handle)
            .await;
    }

    /// Storage expectations for loading one pending job, tolerant of the
    /// extra reads the spawned task performs before it gets cancelled.
    fn expect_pending_job_reads(store: &mut MockStateStore, seed: u8) {
        let job = mk_job(seed);
        store
            .expect_get_settlement_job()
            .returning(move |_| Ok(Some(job.clone())));
        store
            .expect_get_settlement_job_result()
            .returning(|_| Ok(None));
        store
            .expect_list_settlement_attempts()
            .returning(|_| Ok(Vec::new()));
        store
            .expect_list_settlement_attempt_results()
            .returning(|_| Ok(Vec::new()));
        store
            .expect_max_settlement_nonce_for_wallet()
            .returning(|_| Ok(None));
    }

    #[tokio::test]
    async fn admin_abort_task_cancels_live_task() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(23);
        expect_pending_job_reads(&mut store, 23);

        let service = mk_service(Arc::new(store)).await;
        load_and_spawn_pending_task(&service, job_id).await;
        assert!(service.has_live_task(job_id).await);

        service
            .admin_abort_task(job_id)
            .await
            .expect("abort of a live task must succeed");

        // The task observes the cancellation asynchronously.
        tokio::time::timeout(std::time::Duration::from_secs(10), async {
            while service.has_live_task(job_id).await {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("aborted task should deregister its control handle");
    }
```

Note: `has_live_task` is written in Task 3; these tests compile only
once Tasks 2 and 3 are both in. Write the tests for both tasks first,
then implement both, then run.

- [ ] **Step 2: Run to verify failure**

Run: `cargo nextest run -p agglayer-settlement-service admin_abort`
Expected: compile error (`admin_abort_task` returns `eyre::Result`,
`SettlementAdminError` and `has_live_task` unknown).

- [ ] **Step 3: Implement the typed abort**

In `settlement_service.rs`:

Add to the imports:

```rust
use crate::error::SettlementAdminError;
```

Delete the `task_control` and `admin_task` private methods
(lines 232-257); the reworked admin methods below replace their only
uses.

Replace `admin_abort_task` (lines 259-263) with:

```rust
    /// Classify why no live task exists for `job_id` by consulting
    /// storage: completed job, pending job with a dead task, or no such
    /// job at all.
    async fn no_live_task_error(&self, job_id: SettlementJobId) -> SettlementAdminError {
        match self.store.get_settlement_job_result(&job_id) {
            Err(source) => SettlementAdminError::Storage { job_id, source },
            Ok(Some(_)) => SettlementAdminError::JobCompleted(job_id),
            Ok(None) => match self.store.get_settlement_job(&job_id) {
                Err(source) => SettlementAdminError::Storage { job_id, source },
                Ok(Some(_)) => SettlementAdminError::NoLiveTask(job_id),
                Ok(None) => SettlementAdminError::JobNotFound(job_id),
            },
        }
    }

    /// Stop the in-memory task of `job_id`. Runtime-only: the job stays
    /// pending in storage and no terminal result is recorded; restart it
    /// with [`Self::admin_reload_and_restart_task`].
    #[tracing::instrument(skip(self))]
    pub async fn admin_abort_task(
        &self,
        job_id: SettlementJobId,
    ) -> Result<(), SettlementAdminError> {
        let control = self.task_controls.lock().await.get(&job_id).cloned();
        match control {
            Some(control) => {
                control.cancel();
                Ok(())
            }
            None => Err(self.no_live_task_error(job_id).await),
        }
    }
```

Update the `tower::Service<AdminCommand>` impl (`call`, lines 414-424)
to convert the typed error:

```rust
    fn call(&mut self, req: AdminCommand) -> Self::Future {
        let this = self.clone();
        Box::pin(async move {
            match req {
                AdminCommand::AbortTask(job_id) => this
                    .admin_abort_task(job_id)
                    .await
                    .map_err(eyre::Report::new),
                AdminCommand::ReloadAndRestartTask(job_id) => this
                    .admin_reload_and_restart_task(job_id)
                    .await
                    .map_err(eyre::Report::new),
            }
        })
    }
```

(`admin_reload_and_restart_task` changes type in Task 4; until then,
leave its old body but change its signature in Task 4, not here.
If the compiler complains about the mixed types in this step, do
Task 4's signature change together with this one.)

- [ ] **Step 4: Continue with Task 3 before running the tests**

### Task 3: `has_live_task`

**Files:**
- Modify: `crates/agglayer-settlement-service/src/settlement_service.rs`

- [ ] **Step 1: Implement**

Add next to `admin_abort_task`:

```rust
    /// Whether an in-memory task is currently registered for `job_id`.
    ///
    /// Advisory: the answer can race with task completion. A `pending`
    /// job without a live task is wedged and needs
    /// [`Self::admin_reload_and_restart_task`].
    pub async fn has_live_task(&self, job_id: SettlementJobId) -> bool {
        self.task_controls.lock().await.contains_key(&job_id)
    }
```

- [ ] **Step 2: Run the Task 2 + 3 tests**

Run: `cargo nextest run -p agglayer-settlement-service admin_abort`
Expected: the four new tests PASS
(if `admin_reload_and_restart_task` still has its eyre signature and
that blocks compilation of the tower impl, jump to Task 4 Step 3 and
come back).

- [ ] **Step 3: Commit**

```bash
git add crates/agglayer-settlement-service
git commit -m "feat(settlement-service): typed admin abort and has_live_task"
```

### Task 4: respawn-capable reload-and-restart

**Files:**
- Modify: `crates/agglayer-settlement-service/src/settlement_service.rs`
  (struct fields lines 28-36, `start` lines 77-94,
  `admin_reload_and_restart_task` lines 265-269, tests)

- [ ] **Step 1: Write the failing tests**

Append to the inline `mod tests`:

```rust
    #[tokio::test]
    async fn admin_reload_and_restart_respawns_dead_task() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(24);
        expect_pending_job_reads(&mut store, 24);

        let cancellation_token = CancellationToken::new();
        let service =
            mk_service_with_token(Arc::new(store), cancellation_token.clone()).await;
        assert!(!service.has_live_task(job_id).await);

        service
            .admin_reload_and_restart_task(job_id)
            .await
            .expect("reload of a pending job without a task must respawn it");

        assert!(service.has_live_task(job_id).await);

        // A retrieval after the respawn gets a functioning watcher.
        let retrieved = service
            .retrieve_settlement_result(job_id)
            .await
            .expect("retrieval after respawn should succeed");
        assert!(matches!(retrieved, RetrievedSettlementResult::Pending(_)));

        cancellation_token.cancel();
    }

    #[tokio::test]
    async fn admin_reload_and_restart_completed_job_returns_job_completed() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(25);
        let job = mk_job(25);
        let result = mk_result(25, ContractCallOutcome::Success);
        store
            .expect_get_settlement_job()
            .returning(move |_| Ok(Some(job.clone())));
        store
            .expect_get_settlement_job_result()
            .returning(move |_| Ok(Some(result.clone())));

        let service = mk_service(Arc::new(store)).await;

        let error = service
            .admin_reload_and_restart_task(job_id)
            .await
            .expect_err("reload of a completed job must fail");
        assert!(matches!(
            error,
            crate::SettlementAdminError::JobCompleted(id) if id == job_id
        ));
    }

    #[tokio::test]
    async fn admin_reload_and_restart_unknown_job_returns_job_not_found() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(26);
        store.expect_get_settlement_job().returning(|_| Ok(None));
        store
            .expect_get_settlement_job_result()
            .returning(|_| Ok(None));

        let service = mk_service(Arc::new(store)).await;

        let error = service
            .admin_reload_and_restart_task(job_id)
            .await
            .expect_err("reload of an unknown job must fail");
        assert!(matches!(
            error,
            crate::SettlementAdminError::JobNotFound(id) if id == job_id
        ));
    }

    #[tokio::test]
    async fn admin_reload_and_restart_live_task_sends_command() {
        let mut store = MockStateStore::new();
        expect_empty_startup_recovery(&mut store);
        let job_id = mk_job_id(27);
        expect_pending_job_reads(&mut store, 27);

        let cancellation_token = CancellationToken::new();
        let service =
            mk_service_with_token(Arc::new(store), cancellation_token.clone()).await;
        load_and_spawn_pending_task(&service, job_id).await;

        service
            .admin_reload_and_restart_task(job_id)
            .await
            .expect("reload of a live task must be accepted");
        // The task stays registered: the reload command re-enters the
        // run loop rather than tearing the task down.
        assert!(service.has_live_task(job_id).await);

        cancellation_token.cancel();
    }
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo nextest run -p agglayer-settlement-service admin_reload`
Expected: FAIL. `admin_reload_and_restart_task` still returns
`eyre::Result` and errors on a dead task instead of respawning.

- [ ] **Step 3: Implement**

Add the admin operation lock to the struct (fields, lines 28-36):

```rust
    task_controls: Arc<Mutex<HashMap<SettlementJobId, TaskControlHandle>>>,
    result_watchers:
        Arc<Mutex<HashMap<SettlementJobId, watch::Receiver<Option<SettlementJobResult>>>>>,
    /// Serializes admin respawn operations so two concurrent
    /// reload-and-restart calls cannot spawn two tasks for one job.
    admin_operation_lock: Arc<Mutex<()>>,
```

Initialize it in `start` (lines 84-91):

```rust
        let this = Self {
            tx_config,
            provider,
            store,
            cancellation_token,
            task_controls: Arc::new(Mutex::new(HashMap::new())),
            result_watchers: Arc::new(Mutex::new(HashMap::new())),
            admin_operation_lock: Arc::new(Mutex::new(())),
        };
```

Replace `admin_reload_and_restart_task` with:

```rust
    /// Make the task of `job_id` drop its in-memory state and reload
    /// from storage. A live task gets the reload command; a pending job
    /// without a live task (aborted or crashed) gets a fresh task
    /// spawned from storage.
    #[tracing::instrument(skip(self))]
    pub async fn admin_reload_and_restart_task(
        &self,
        job_id: SettlementJobId,
    ) -> Result<(), SettlementAdminError> {
        let _admin_op = self.admin_operation_lock.lock().await;

        // Fast path: a live task processes the reload itself.
        let control = self.task_controls.lock().await.get(&job_id).cloned();
        if let Some(control) = control {
            return control
                .try_send(TaskAdminCommand::ReloadAndRestart)
                .map_err(|error| SettlementAdminError::TaskNotResponding {
                    job_id,
                    reason: error.to_string(),
                });
        }

        // No live task: respawn from storage if the job is still pending.
        let (task_control_handle, task_control) =
            TaskControlHandle::new(&self.cancellation_token);
        match SettlementTask::load(
            job_id,
            self.tx_config.clone(),
            self.provider.clone(),
            self.store.clone(),
            task_control,
        )
        .await
        {
            Ok(StoredSettlementJob::Pending(task)) => {
                self.spawn_settlement_task(job_id, task, task_control_handle)
                    .await;
                info!(%job_id, "Respawned settlement task via admin reload-and-restart");
                Ok(())
            }
            Ok(StoredSettlementJob::Completed(_, _)) => {
                Err(SettlementAdminError::JobCompleted(job_id))
            }
            Err(error) => match self.store.get_settlement_job(&job_id) {
                Ok(None) => Err(SettlementAdminError::JobNotFound(job_id)),
                _ => Err(SettlementAdminError::ReloadFailed {
                    job_id,
                    reason: format!("{error:#}"),
                }),
            },
        }
    }
```

- [ ] **Step 4: Run the full crate test suite**

Run: `cargo nextest run -p agglayer-settlement-service`
Expected: all tests PASS (old and new).

- [ ] **Step 5: Update the service doc comment**

The struct doc (lines 18-25) says abort has no recovery path.
Replace its last sentence with:

```rust
/// The admin abort escape hatch is the exception: it stops a task
/// without recording a terminal result, leaving the job pending with no
/// live task until `admin_reload_and_restart_task` respawns it.
```

- [ ] **Step 6: Commit**

```bash
git add crates/agglayer-settlement-service
git commit -m "feat(settlement-service): respawn dead tasks in admin reload-and-restart"
```

### Task 5: reverse link dual write and reader

**Files:**
- Modify: `crates/agglayer-storage/src/stores/interfaces/reader/settlement_reader.rs`
- Modify: `crates/agglayer-storage/src/stores/state/settlement/mod.rs`
- Modify: `crates/agglayer-storage/src/stores/state/mod.rs`
  (`insert_certificate_settlement_job_id`, around line 212)
- Modify: `crates/agglayer-storage/src/tests/mocks/state_store.rs`
- Modify: `crates/agglayer-storage/src/stores/state/tests/settlement.rs`

- [ ] **Step 1: Write the failing store tests**

Append to `crates/agglayer-storage/src/stores/state/tests/settlement.rs`:

```rust
#[test]
fn certificate_settlement_job_link_is_readable_in_both_directions() {
    let (_tmp, _db, store) = setup_store();
    let certificate_id = mk_certificate_id(40);
    let job_id = mk_job_id(40);

    store
        .insert_certificate_settlement_job_id(&certificate_id, &job_id)
        .expect("link insert must succeed");

    assert_eq!(
        store
            .get_certificate_settlement_job_id(&certificate_id)
            .expect("forward read must succeed"),
        Some(job_id),
    );
    assert_eq!(
        store
            .get_settlement_job_certificate_id(&job_id)
            .expect("reverse read must succeed"),
        Some(certificate_id),
    );
}

#[test]
fn get_settlement_job_certificate_id_returns_none_for_unlinked_job() {
    let (_tmp, _db, store) = setup_store();
    assert_eq!(
        store
            .get_settlement_job_certificate_id(&mk_job_id(41))
            .expect("reverse read must succeed"),
        None,
    );
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo nextest run -p agglayer-storage certificate_settlement_job_link`
Expected: compile error, `get_settlement_job_certificate_id` not found.

- [ ] **Step 3: Add the reader trait method**

In `settlement_reader.rs`, extend the imports with `CertificateId` and
append to the trait:

```rust
    /// Returns the certificate linked to `settlement_job_id`, if any.
    ///
    /// Jobs created without a certificate, and jobs linked before the
    /// reverse column was populated, return `None`.
    fn get_settlement_job_certificate_id(
        &self,
        settlement_job_id: &SettlementJobId,
    ) -> Result<Option<CertificateId>, Error>;
```

- [ ] **Step 4: Implement on StateStore**

In `crates/agglayer-storage/src/stores/state/settlement/mod.rs`, add
`CertificateId` to the `agglayer_types` import,
`certificate_id_per_settlement_job_id::CertificateIdPerSettlementJobIdColumn`
to the `columns` import, and append to the `SettlementReader` impl:

```rust
    fn get_settlement_job_certificate_id(
        &self,
        settlement_job_id: &SettlementJobId,
    ) -> Result<Option<CertificateId>, Error> {
        Ok(self
            .db
            .get::<CertificateIdPerSettlementJobIdColumn>(settlement_job_id)?)
    }
```

- [ ] **Step 5: Write the reverse column**

In `crates/agglayer-storage/src/stores/state/mod.rs`, replace the write
in `insert_certificate_settlement_job_id` (the final `Ok(self.db.put...)`
around line 225) with a batch covering both columns
(add `CertificateIdPerSettlementJobIdColumn` to the columns import;
`WriteBatch` is already used in this file):

```rust
        let mut batch = rocksdb::WriteBatch::default();
        self.db.multi_insert_batch::<SettlementJobIdPerCertificateIdColumn>(
            [(certificate_id, settlement_job_id)],
            &mut batch,
        )?;
        self.db.multi_insert_batch::<CertificateIdPerSettlementJobIdColumn>(
            [(settlement_job_id, certificate_id)],
            &mut batch,
        )?;
        Ok(self.db.write_batch(batch)?)
```

- [ ] **Step 6: Extend the mock**

In `crates/agglayer-storage/src/tests/mocks/state_store.rs`, append to
the `impl SettlementReader for StateStore` mock block:

```rust
        fn get_settlement_job_certificate_id(
            &self,
            settlement_job_id: &SettlementJobId,
        ) -> Result<Option<CertificateId>, Error>;
```

- [ ] **Step 7: Run the tests**

Run: `cargo nextest run -p agglayer-storage`
Expected: all PASS, including the two new tests
(existing link tests keep passing: the forward column behavior and the
duplicate check are unchanged).
Then: `cargo nextest run -p agglayer-settlement-service`
Expected: PASS (the settlement service consumes the grown trait via the
regenerated mock).

- [ ] **Step 8: Commit**

```bash
git add crates/agglayer-storage
git commit -m "feat(storage): populate and expose the settlement job to certificate link"
```

### Task 6: PR 1 verification and creation

- [ ] **Step 1: Definition-of-done checks**

```bash
cargo +nightly fmt --check
cargo clippy -p agglayer-settlement-service -p agglayer-storage --all-targets -- -D warnings
cargo nextest run -p agglayer-settlement-service -p agglayer-storage
```
Expected: all clean. Fix and amend nothing; add fixes as new commits.

- [ ] **Step 2: Push and open PR 1** (with user approval)

Title: `feat(settlement): admin groundwork for task controls and job reads`
Body: link issue 1675 and the spec; state the three behavior changes
(typed admin errors, respawn-capable reload, reverse link dual write)
and the known limitation (rows linked before this change read a null
reverse link).

---

## PR 2 — admin RPC plumbing and task controls

Branch: `feat/1675-settlement-admin-task-controls`
(from `feat/1675-settlement-admin-groundwork`).
Crates: `agglayer-jsonrpc-api`, `agglayer-node`.

### Task 7: thread the settlement service into the admin RPC

**Files:**
- Modify: `crates/agglayer-jsonrpc-api/Cargo.toml`
- Modify: `crates/agglayer-jsonrpc-api/src/admin.rs`
  (struct lines 204-228, impl headers at 231, 297, 306)
- Modify: `crates/agglayer-jsonrpc-api/src/testutils.rs`
- Modify: `crates/agglayer-node/src/node.rs` (lines 269-324)

This is plumbing: correctness is "everything still compiles and all
existing tests pass". Keep it byte-compatible with PR 1663 so the
mutations branch rebases cleanly.

- [ ] **Step 1: Add the dependency**

In `crates/agglayer-jsonrpc-api/Cargo.toml` under `[dependencies]`
(alphabetical order):

```toml
agglayer-settlement-service.workspace = true
```

- [ ] **Step 2: Extend `AdminAgglayerImpl`**

In `admin.rs`, add imports:

```rust
use agglayer_settlement_service::SettlementService;
use alloy::providers::{Provider, WalletProvider};
```

Change the struct and constructor:

```rust
/// The Admin RPC agglayer service implementation.
pub struct AdminAgglayerImpl<PendingStore, StateStore, DebugStore, L1Provider> {
    certificate_sender: mpsc::Sender<(NetworkId, Height, CertificateId)>,
    pending_store: Arc<PendingStore>,
    state: Arc<StateStore>,
    debug_store: Arc<DebugStore>,
    config: Arc<Config>,
    settlement_service: SettlementService<L1Provider, StateStore>,
}

impl<PendingStore, StateStore, DebugStore, L1Provider>
    AdminAgglayerImpl<PendingStore, StateStore, DebugStore, L1Provider>
{
    /// Create an instance of the admin RPC agglayer service.
    pub fn new(
        certificate_sender: mpsc::Sender<(NetworkId, Height, CertificateId)>,
        pending_store: Arc<PendingStore>,
        state: Arc<StateStore>,
        debug_store: Arc<DebugStore>,
        config: Arc<Config>,
        settlement_service: SettlementService<L1Provider, StateStore>,
    ) -> Self {
        Self {
            certificate_sender,
            pending_store,
            state,
            debug_store,
            config,
            settlement_service,
        }
    }
}
```

Propagate the parameter to the other three impl blocks
(`start`, `Drop`, `AdminAgglayerServer`), matching PR 1663:

```rust
impl<PendingStore, StateStore, DebugStore, L1Provider>
    AdminAgglayerImpl<PendingStore, StateStore, DebugStore, L1Provider>
where
    PendingStore: PendingCertificateWriter + PendingCertificateReader + 'static,
    StateStore: StateReader + StateWriter + 'static,
    DebugStore: DebugReader + DebugWriter + 'static,
    L1Provider: Provider + WalletProvider + 'static,
{
    pub async fn start(self) -> eyre::Result<axum::Router> {
```

```rust
impl<PendingStore, StateStore, DebugStore, L1Provider> Drop
    for AdminAgglayerImpl<PendingStore, StateStore, DebugStore, L1Provider>
{
```

```rust
impl<PendingStore, StateStore, DebugStore, L1Provider> AdminAgglayerServer
    for AdminAgglayerImpl<PendingStore, StateStore, DebugStore, L1Provider>
where
    PendingStore: PendingCertificateWriter + PendingCertificateReader + 'static,
    StateStore: StateReader + StateWriter + 'static,
    DebugStore: DebugReader + DebugWriter + 'static,
    L1Provider: Provider + WalletProvider + 'static,
{
```

(PR 1663 also adds `SettlementReader + SettlementWriter` bounds on
`StateStore`; PR 3 needs `SettlementReader` for the reads and the
service's own bounds require the rest, so add
`+ SettlementReader + SettlementWriter` to both `where` clauses now,
importing them from `agglayer_storage::stores`. `SettlementService`
methods used here also require the store `Send + Sync`, which the
concrete `StateStore` satisfies; add those bounds if the compiler asks.)

- [ ] **Step 3: Wire node.rs**

After the settlement service creation (line 279, before the
`data_sender` channel), keep a clone:

```rust
        let settlement_service_for_admin = (*settlement_service).clone();
```

And extend the `AdminAgglayerImpl::new` call (lines 315-324):

```rust
        let admin_router = AdminAgglayerImpl::new(
            data_sender,
            pending_store.clone(),
            state_store.clone(),
            debug_store.clone(),
            config.clone(),
            settlement_service_for_admin,
        )
        .start()
        .await
        .context("Failed starting admin router")?;
```

- [ ] **Step 4: Wire testutils**

In `crates/agglayer-jsonrpc-api/src/testutils.rs`:

Add imports:

```rust
use agglayer_config::settlement_service::{
    SettlementServiceConfig, SettlementTransactionConfig,
};
use agglayer_settlement_service::SettlementService;
use alloy::{
    network::EthereumWallet,
    providers::fillers::WalletFiller,
    signers::local::PrivateKeySigner,
};
```

Add a concrete provider alias next to `MockProvider` and expose the
service on the context:

```rust
/// Provider used by the test settlement service: wallet-carrying, with
/// a dead HTTP endpoint. Startup recovery does no L1 calls, spawned
/// tasks block retrying against L1 until cancelled.
pub type SettlementTestProvider = FillProvider<
    JoinFill<
        JoinFill<
            Identity,
            JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
        >,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider,
    alloy::network::Ethereum,
>;
```

(If the compiler disagrees with the filler nesting, take the exact type
from the mismatch in the error message; the alias mirrors
`ProviderBuilder::new().wallet(...).connect_http(...)`.)

```rust
pub struct TestContext {
    pub cancellation_token: CancellationToken,
    pub state_store: Arc<StateStore>,
    pub pending_store: Arc<PendingStore>,
    pub settlement_service: SettlementService<SettlementTestProvider, StateStore>,
    pub api_client: HttpClient,
    pub admin_client: HttpClient,
    pub config: Arc<Config>,
    pub certificate_receiver: tokio::sync::mpsc::Receiver<(NetworkId, Height, CertificateId)>,
}
```

In `new_with_provider`, before the routers are created (line 151),
build the service (PR 1663 shape) and pass a clone to the admin impl:

```rust
        // A settlement service over the (empty) state store, with its own
        // wallet-carrying provider pointed at a dead endpoint: startup
        // recovery finds no jobs, and spawned tasks retry against L1
        // until cancelled instead of completing.
        let settlement_provider = ProviderBuilder::new()
            .wallet(EthereumWallet::from(
                PrivateKeySigner::from_slice(&[0x11; 32]).expect("valid test signing key"),
            ))
            .connect_http(
                "http://127.0.0.1:0"
                    .parse()
                    .expect("test provider URL should parse"),
            );
        let settlement_service = SettlementService::start(
            SettlementServiceConfig::default(),
            Arc::new(SettlementTransactionConfig::default()),
            Arc::new(settlement_provider),
            state_store.clone(),
            cancellation_token.clone(),
        )
        .await
        .expect("settlement service should start");
```

```rust
        let admin_router = AdminAgglayerImpl::new(
            certificate_sender,
            pending_store.clone(),
            state_store.clone(),
            debug_store.clone(),
            config.clone(),
            settlement_service.clone(),
        )
        .start()
        .await
        .unwrap();
```

And add `settlement_service` to the `Self { ... }` constructor at the
end of the function.

- [ ] **Step 5: Compile and run existing tests**

```bash
cargo check -p agglayer-jsonrpc-api -p agglayer-node
cargo nextest run -p agglayer-jsonrpc-api
```
Expected: clean compile, all existing tests PASS (no behavior change).

- [ ] **Step 6: Commit**

```bash
git add crates/agglayer-jsonrpc-api crates/agglayer-node Cargo.lock
git commit -m "feat(jsonrpc-api): thread the settlement service into the admin RPC"
```

### Task 8: RPC error mapping for settlement admin

**Files:**
- Modify: `crates/agglayer-jsonrpc-api/src/error.rs`
- Modify: `crates/agglayer-jsonrpc-api/src/tests/errors.rs`

- [ ] **Step 1: Write the failing snapshot cases**

In `tests/errors.rs`, add two `#[case]`s to `rpc_error_rendering`
(next to the existing cases) and the import
`use agglayer_types::SettlementJobId;` plus
`use agglayer_settlement_service::SettlementAdminError;`:

```rust
#[case(
    "settlement_admin_job_not_found",
    SettlementAdminError::JobNotFound(SettlementJobId::from(7u128))
)]
#[case(
    "settlement_admin_no_live_task",
    SettlementAdminError::NoLiveTask(SettlementJobId::from(8u128))
)]
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo nextest run -p agglayer-jsonrpc-api rpc_error_rendering`
Expected: compile error, no `From<SettlementAdminError> for Error`.

- [ ] **Step 3: Implement the mapping**

In `error.rs`:

Add to the `code` module:

```rust
    /// Settlement admin operation failure.
    pub const SETTLEMENT_ADMIN: i32 = -10010;
```

Add a variant to `Error` (after `MethodDisabled`):

```rust
    #[error("Settlement admin error: {0}")]
    SettlementAdmin(String),
```

Map it in `Error::code`:

```rust
            Self::SettlementAdmin(_) => code::SETTLEMENT_ADMIN,
```

Add the conversion (job-not-found reuses the resource-not-found code,
matching the certificate retrieval mapping above it):

```rust
impl From<agglayer_settlement_service::SettlementAdminError> for Error {
    fn from(error: agglayer_settlement_service::SettlementAdminError) -> Self {
        use agglayer_settlement_service::SettlementAdminError as E;
        match error {
            E::JobNotFound(job_id) => {
                Self::ResourceNotFound(format!("SettlementJob({job_id})"))
            }
            error => Self::SettlementAdmin(error.to_string()),
        }
    }
}
```

- [ ] **Step 4: Run and accept snapshots**

```bash
INSTA_UPDATE=always cargo nextest run -p agglayer-jsonrpc-api rpc_error_rendering
cargo nextest run -p agglayer-jsonrpc-api rpc_error_rendering
```
Expected: first run writes the two new snapshots, second run PASSES.
Review the two new `.snap` files by hand before committing.

- [ ] **Step 5: Commit**

```bash
git add crates/agglayer-jsonrpc-api
git commit -m "feat(jsonrpc-api): map settlement admin errors to RPC errors"
```

### Task 9: the two control methods and their API tests

**Files:**
- Modify: `crates/agglayer-jsonrpc-api/src/admin.rs`
- Modify: `crates/agglayer-jsonrpc-api/src/tests.rs`
- Create: `crates/agglayer-jsonrpc-api/src/tests/settlement_admin.rs`

- [ ] **Step 1: Write the failing API tests**

Register the module in `src/tests.rs` (alphabetical):

```rust
mod settlement_admin;
```

Create `src/tests/settlement_admin.rs`:

```rust
use agglayer_storage::stores::SettlementWriter;
use agglayer_types::{SettlementJob, SettlementJobId, U256};
use jsonrpsee::{core::client::ClientT, rpc_params};

use crate::testutils::TestContext;

fn mk_job_id(seed: u128) -> SettlementJobId {
    SettlementJobId::from(seed)
}

fn mk_job(seed: u8) -> SettlementJob {
    SettlementJob {
        contract_address: agglayer_types::Address::from([seed; 20]),
        calldata: vec![seed, seed.wrapping_add(1)].into(),
        eth_value: U256::from(seed),
        gas_limit: seed as u128 + 100_000,
    }
}

/// Seed one pending settlement job directly in storage. The settlement
/// service does not know about it until reload-and-restart spawns its
/// task.
fn seed_pending_job(context: &TestContext, seed: u8) -> SettlementJobId {
    let job_id = mk_job_id(seed as u128);
    context
        .state_store
        .insert_settlement_job(&job_id, &mk_job(seed))
        .expect("job insert must succeed");
    job_id
}

async fn wait_until_task_gone(context: &TestContext, job_id: SettlementJobId) {
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        while context.settlement_service.has_live_task(job_id).await {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("aborted task should deregister");
}

fn assert_error_code(error: jsonrpsee::core::client::Error, code: i32) {
    match error {
        jsonrpsee::core::client::Error::Call(call_error) => {
            assert_eq!(call_error.code(), code)
        }
        other => panic!("expected a call error, got {other:?}"),
    }
}

#[test_log::test(tokio::test)]
async fn abort_and_reload_settlement_task_round_trip() {
    let context = TestContext::new_with_config(TestContext::get_default_config()).await;
    let job_id = seed_pending_job(&context, 1);

    // Dead-task path: reload-and-restart spawns the task from storage.
    let () = context
        .admin_client
        .request("admin_reloadAndRestartSettlementTask", rpc_params![job_id])
        .await
        .expect("reload of a pending job must respawn its task");
    assert!(context.settlement_service.has_live_task(job_id).await);

    // Live-task path: a second reload is accepted by the running task.
    let () = context
        .admin_client
        .request("admin_reloadAndRestartSettlementTask", rpc_params![job_id])
        .await
        .expect("reload of a live task must be accepted");

    // Abort stops it.
    let () = context
        .admin_client
        .request("admin_abortSettlementTask", rpc_params![job_id])
        .await
        .expect("abort of a live task must succeed");
    wait_until_task_gone(&context, job_id).await;

    // A second abort reports the dead task.
    let error = context
        .admin_client
        .request::<(), _>("admin_abortSettlementTask", rpc_params![job_id])
        .await
        .expect_err("abort without a live task must fail");
    assert_error_code(error, crate::error::code::SETTLEMENT_ADMIN);

    // And reload-and-restart revives it: the full unstick cycle.
    let () = context
        .admin_client
        .request("admin_reloadAndRestartSettlementTask", rpc_params![job_id])
        .await
        .expect("reload after abort must respawn the task");
    assert!(context.settlement_service.has_live_task(job_id).await);
}

#[test_log::test(tokio::test)]
async fn settlement_task_controls_report_unknown_job() {
    let context = TestContext::new_with_config(TestContext::get_default_config()).await;
    let job_id = mk_job_id(99);

    for method in ["admin_abortSettlementTask", "admin_reloadAndRestartSettlementTask"] {
        let error = context
            .admin_client
            .request::<(), _>(method, rpc_params![job_id])
            .await
            .expect_err("unknown job must fail");
        assert_error_code(error, crate::error::code::RESOURCE_NOT_FOUND);
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo nextest run -p agglayer-jsonrpc-api settlement_admin`
Expected: FAIL with "Method not found" errors.

- [ ] **Step 3: Add the trait methods**

In `admin.rs`, append to the `AdminAgglayer` trait (after
`enable_network`), importing `SettlementJobId` from `agglayer_types`:

```rust
    /// Stop the in-memory settlement task of a job.
    ///
    /// **JSON-RPC method:** `admin_abortSettlementTask`
    ///
    /// Cancels the task driving `job_id`. Runtime-only: the job stays
    /// pending in storage and no terminal result is recorded, so the
    /// certificate waiting on it stays blocked until the task is
    /// restarted with `admin_reloadAndRestartSettlementTask`. Fails when
    /// no task is running (job unknown, completed, or already aborted).
    #[method(name = "abortSettlementTask")]
    async fn abort_settlement_task(&self, job_id: SettlementJobId) -> RpcResult<()>;

    /// Reload a settlement job from storage and (re)start its task.
    ///
    /// **JSON-RPC method:** `admin_reloadAndRestartSettlementTask`
    ///
    /// A live task drops its in-memory state and reloads from storage.
    /// A pending job without a live task (aborted, or its task crashed)
    /// gets a fresh task spawned from storage: this is the recovery step
    /// after `admin_abortSettlementTask`. Fails if the job is unknown or
    /// already completed.
    #[method(name = "reloadAndRestartSettlementTask")]
    async fn reload_and_restart_settlement_task(
        &self,
        job_id: SettlementJobId,
    ) -> RpcResult<()>;
```

- [ ] **Step 4: Implement the handlers**

Append to the `AdminAgglayerServer` impl block:

```rust
    #[instrument(skip(self))]
    async fn abort_settlement_task(&self, job_id: SettlementJobId) -> RpcResult<()> {
        warn!(%job_id, "Aborting settlement task via admin RPC");
        Ok(self.settlement_service.admin_abort_task(job_id).await?)
    }

    #[instrument(skip(self))]
    async fn reload_and_restart_settlement_task(
        &self,
        job_id: SettlementJobId,
    ) -> RpcResult<()> {
        warn!(%job_id, "Reloading and restarting settlement task via admin RPC");
        Ok(self
            .settlement_service
            .admin_reload_and_restart_task(job_id)
            .await?)
    }
```

- [ ] **Step 5: Run the tests**

Run: `cargo nextest run -p agglayer-jsonrpc-api`
Expected: all PASS, including the two new settlement_admin tests.

- [ ] **Step 6: Commit, verify, open PR 2** (with user approval)

```bash
git add crates/agglayer-jsonrpc-api
git commit -m "feat(jsonrpc-api): admin abort and reload-and-restart for settlement tasks"
cargo +nightly fmt --check
cargo clippy -p agglayer-jsonrpc-api -p agglayer-node --all-targets -- -D warnings
cargo nextest run -p agglayer-jsonrpc-api
```

Title: `feat(settlement): admin RPC task controls for settlement jobs`
Base: `feat/1675-settlement-admin-groundwork`.

---

## PR 3 — reads and operator documentation

Branch: `feat/1675-settlement-admin-reads`
(from `feat/1675-settlement-admin-task-controls`).
Crates: `agglayer-jsonrpc-api`, plus `docs/knowledge-base`.

### Task 10: DTO module

**Files:**
- Create: `crates/agglayer-jsonrpc-api/src/settlement_admin.rs`
- Create: `crates/agglayer-jsonrpc-api/src/settlement_admin/tests.rs`
- Modify: `crates/agglayer-jsonrpc-api/src/lib.rs` (module list)

- [ ] **Step 1: Write the failing unit tests**

Create `src/settlement_admin/tests.rs`:

```rust
use std::time::{Duration, SystemTime};

use agglayer_types::{
    Address, ClientError, ClientErrorType, ContractCallOutcome, ContractCallResult,
    Digest, Nonce, SettlementAttempt, SettlementAttemptResult, SettlementTxHash, B256,
};

use super::*;

fn attempt(seed: u64) -> SettlementAttempt {
    SettlementAttempt {
        sender_wallet: Address::from([seed as u8; 20]),
        nonce: Nonce(seed),
        hash: SettlementTxHash::new(Digest::from([seed as u8; 32])),
        submission_time: SystemTime::UNIX_EPOCH + Duration::from_secs(seed),
        max_fee_per_gas: 30_000_000_000,
        max_priority_fee_per_gas: 1_000_000_000,
    }
}

fn client_error_result(message: &str) -> SettlementAttemptResult {
    SettlementAttemptResult::ClientError(ClientError {
        kind: ClientErrorType::Unknown,
        message: message.to_string(),
    })
}

fn contract_call_result(outcome: ContractCallOutcome) -> SettlementAttemptResult {
    SettlementAttemptResult::ContractCall(ContractCallResult {
        outcome,
        metadata: Vec::new().into(),
        block_hash: B256::from([9u8; 32]),
        block_number: 9,
        tx_hash: SettlementTxHash::new(Digest::from([9u8; 32])),
    })
}

#[test]
fn last_error_is_none_without_results() {
    assert_eq!(render_last_error(&[]), None);
}

#[test]
fn last_error_renders_the_latest_client_error() {
    let results = vec![
        (0, client_error_result("older error")),
        (1, client_error_result("newer error")),
    ];
    let rendered = render_last_error(&results).expect("latest failure must render");
    assert!(rendered.contains("newer error"), "got: {rendered}");
}

#[test]
fn last_error_is_none_when_latest_result_is_a_success() {
    let results = vec![
        (0, client_error_result("older error")),
        (1, contract_call_result(ContractCallOutcome::Success)),
    ];
    assert_eq!(render_last_error(&results), None);
}

#[test]
fn last_error_renders_the_latest_revert() {
    let results = vec![(0, contract_call_result(ContractCallOutcome::Revert))];
    let rendered = render_last_error(&results).expect("revert must render");
    assert!(rendered.contains("Reverted"), "got: {rendered}");
}

#[test]
fn job_summary_serializes_camel_case() {
    let summary = SettlementJobSummary {
        job_id: agglayer_types::SettlementJobId::from(1u128),
        certificate_id: None,
        status: SettlementJobStatus::Pending,
        has_live_task: true,
        attempt_count: 1,
        latest_attempt: Some(SettlementAttemptSummary::from((0u64, &attempt(0)))),
        last_error: None,
    };
    let json = serde_json::to_value(&summary).expect("summary must serialize");
    assert!(json.get("hasLiveTask").is_some());
    assert!(json.get("attemptCount").is_some());
    assert_eq!(json["status"], "pending");
    assert!(json["latestAttempt"].get("senderWallet").is_some());
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo nextest run -p agglayer-jsonrpc-api settlement_admin::tests`
Expected: compile error, module does not exist yet.

- [ ] **Step 3: Implement the DTO module**

Add to `src/lib.rs` module list: `pub mod settlement_admin;`

Create `src/settlement_admin.rs`:

```rust
//! Wire types for the settlement admin read methods.
//!
//! The settlement domain types in `agglayer-types` carry no serde; the
//! JSON representation is owned here, at the RPC boundary (same pattern
//! as `TokenBalanceEntry` for `admin_getTokenBalance`).

use std::time::SystemTime;

use agglayer_types::{
    Address, CertificateId, ContractCallOutcome, SettlementAttempt,
    SettlementAttemptResult, SettlementJob, SettlementJobId, SettlementJobResult,
    SettlementTxHash, B256, U256,
};
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// Storage-derived job state: pending while no terminal result row
/// exists, completed once it does.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SettlementJobStatus {
    Pending,
    Completed,
}

/// One row of `admin_listSettlementJobs`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementJobSummary {
    pub job_id: SettlementJobId,
    /// Certificate linked to the job. Null for jobs created without a
    /// certificate and for jobs linked before the reverse link existed.
    pub certificate_id: Option<CertificateId>,
    pub status: SettlementJobStatus,
    /// Whether an in-memory task currently drives the job. A pending
    /// job without a live task is wedged:
    /// use `admin_reloadAndRestartSettlementTask`.
    pub has_live_task: bool,
    pub attempt_count: u64,
    pub latest_attempt: Option<SettlementAttemptSummary>,
    /// Human-readable rendering of the most recent attempt result when
    /// it is a failure (client error or on-chain revert), null
    /// otherwise.
    pub last_error: Option<String>,
}

/// Attempt identification fields shown in the job list.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementAttemptSummary {
    pub attempt_number: u64,
    pub sender_wallet: Address,
    pub nonce: u64,
    pub tx_hash: SettlementTxHash,
}

impl From<(u64, &SettlementAttempt)> for SettlementAttemptSummary {
    fn from((attempt_number, attempt): (u64, &SettlementAttempt)) -> Self {
        Self {
            attempt_number,
            sender_wallet: attempt.sender_wallet,
            nonce: attempt.nonce.0,
            tx_hash: attempt.hash,
        }
    }
}

/// Full job detail returned by `admin_getSettlementJob`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementJobDetail {
    pub job_id: SettlementJobId,
    pub certificate_id: Option<CertificateId>,
    pub status: SettlementJobStatus,
    pub has_live_task: bool,
    pub contract_address: Address,
    /// `U256`, rendered in alloy's JSON form (hex quantity string).
    pub eth_value: U256,
    pub gas_limit: u128,
    /// Full settlement calldata, hex-encoded. Admin-only surface, so
    /// the size is acceptable.
    pub calldata: alloy::primitives::Bytes,
    pub attempts: Vec<SettlementAttemptDetail>,
    pub job_result: Option<SettlementJobResultDto>,
    pub last_error: Option<String>,
}

/// One attempt with its recorded result, as returned in the job detail.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementAttemptDetail {
    pub attempt_number: u64,
    pub sender_wallet: Address,
    pub nonce: u64,
    pub tx_hash: SettlementTxHash,
    pub submission_time_unix_secs: u64,
    pub max_fee_per_gas: u128,
    pub max_priority_fee_per_gas: u128,
    pub result: Option<SettlementAttemptResultDto>,
}

impl SettlementAttemptDetail {
    pub fn new(
        attempt_number: u64,
        attempt: &SettlementAttempt,
        result: Option<&SettlementAttemptResult>,
    ) -> Self {
        Self {
            attempt_number,
            sender_wallet: attempt.sender_wallet,
            nonce: attempt.nonce.0,
            tx_hash: attempt.hash,
            submission_time_unix_secs: attempt
                .submission_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|duration| duration.as_secs())
                .unwrap_or(0),
            max_fee_per_gas: attempt.max_fee_per_gas,
            max_priority_fee_per_gas: attempt.max_priority_fee_per_gas,
            result: result.map(SettlementAttemptResultDto::from),
        }
    }
}

/// Recorded outcome of one attempt.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SettlementAttemptResultDto {
    #[serde(rename_all = "camelCase")]
    ClientError { kind: String, message: String },
    #[serde(rename_all = "camelCase")]
    ContractCall {
        outcome: String,
        tx_hash: SettlementTxHash,
        block_number: u64,
        block_hash: B256,
    },
}

impl From<&SettlementAttemptResult> for SettlementAttemptResultDto {
    fn from(result: &SettlementAttemptResult) -> Self {
        match result {
            SettlementAttemptResult::ClientError(client_error) => Self::ClientError {
                kind: format!("{:?}", client_error.kind),
                message: client_error.message.clone(),
            },
            SettlementAttemptResult::ContractCall(call) => Self::ContractCall {
                outcome: match call.outcome {
                    ContractCallOutcome::Success => "success".to_string(),
                    ContractCallOutcome::Revert => "revert".to_string(),
                },
                tx_hash: call.tx_hash,
                block_number: call.block_number,
                block_hash: call.block_hash,
            },
        }
    }
}

/// Terminal result of a completed job.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementJobResultDto {
    pub wallet: Address,
    pub nonce: u64,
    pub attempt_number: u64,
    pub outcome: String,
    pub tx_hash: SettlementTxHash,
    pub block_number: u64,
}

impl From<&SettlementJobResult> for SettlementJobResultDto {
    fn from(result: &SettlementJobResult) -> Self {
        Self {
            wallet: result.wallet,
            nonce: result.nonce.0,
            attempt_number: result.attempt_number.0,
            outcome: match result.contract_call_result.outcome {
                ContractCallOutcome::Success => "success".to_string(),
                ContractCallOutcome::Revert => "revert".to_string(),
            },
            tx_hash: result.contract_call_result.tx_hash,
            block_number: result.contract_call_result.block_number,
        }
    }
}

/// Render the most recent attempt result as an operator-facing error
/// string, or `None` when the latest recorded state is not a failure.
pub(crate) fn render_last_error(
    results: &[(u64, SettlementAttemptResult)],
) -> Option<String> {
    let (_, latest) = results.iter().max_by_key(|(number, _)| *number)?;
    match latest {
        SettlementAttemptResult::ClientError(client_error) => Some(format!(
            "{:?}: {}",
            client_error.kind, client_error.message
        )),
        SettlementAttemptResult::ContractCall(call) => match call.outcome {
            ContractCallOutcome::Revert => Some(format!(
                "Reverted on L1 in tx {} (block {})",
                call.tx_hash, call.block_number
            )),
            ContractCallOutcome::Success => None,
        },
    }
}

/// Build one list row from its storage and service inputs.
pub(crate) fn build_job_summary(
    job_id: SettlementJobId,
    certificate_id: Option<CertificateId>,
    has_live_task: bool,
    job_result: Option<&SettlementJobResult>,
    attempts: &[(u64, SettlementAttempt)],
    attempt_results: &[(u64, SettlementAttemptResult)],
) -> SettlementJobSummary {
    let latest_attempt = attempts
        .iter()
        .max_by_key(|(number, _)| *number)
        .map(|(number, attempt)| SettlementAttemptSummary::from((*number, attempt)));
    SettlementJobSummary {
        job_id,
        certificate_id,
        status: if job_result.is_some() {
            SettlementJobStatus::Completed
        } else {
            SettlementJobStatus::Pending
        },
        has_live_task,
        attempt_count: attempts.len() as u64,
        latest_attempt,
        last_error: render_last_error(attempt_results),
    }
}
```

(If `agglayer_types` does not re-export `ContractCallOutcome` or other
settlement types at the crate root, import them from
`agglayer_types::settlement::...` per the compiler's suggestion; the
storage tests import them from the root, so the root path is expected
to work.)

- [ ] **Step 4: Run the unit tests**

Run: `cargo nextest run -p agglayer-jsonrpc-api settlement_admin::tests`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/agglayer-jsonrpc-api
git commit -m "feat(jsonrpc-api): settlement admin read DTOs"
```

### Task 11: `admin_listSettlementJobs` and `admin_getSettlementJob`

**Files:**
- Modify: `crates/agglayer-jsonrpc-api/src/admin.rs`
- Modify: `crates/agglayer-jsonrpc-api/src/tests/settlement_admin.rs`

- [ ] **Step 1: Write the failing API tests**

Append to `src/tests/settlement_admin.rs`,
merging the new `use` items into the existing import block at the top
of the file:

```rust
use std::time::{Duration, SystemTime};

use agglayer_storage::stores::StateWriter;
use agglayer_types::{
    CertificateId, ClientError, ClientErrorType, ContractCallOutcome, ContractCallResult,
    Digest, Nonce, SettlementAttempt, SettlementAttemptNumber, SettlementAttemptResult,
    SettlementJobResult, SettlementTxHash, B256,
};

use crate::settlement_admin::{SettlementJobDetail, SettlementJobStatus, SettlementJobSummary};

fn mk_attempt(seed: u64) -> SettlementAttempt {
    SettlementAttempt {
        sender_wallet: agglayer_types::Address::from([seed as u8; 20]),
        nonce: Nonce(seed),
        hash: SettlementTxHash::new(Digest::from([seed as u8; 32])),
        submission_time: SystemTime::UNIX_EPOCH + Duration::from_secs(seed),
        max_fee_per_gas: 30_000_000_000,
        max_priority_fee_per_gas: 1_000_000_000,
    }
}

#[test_log::test(tokio::test)]
async fn list_settlement_jobs_returns_seeded_jobs() {
    let context = TestContext::new_with_config(TestContext::get_default_config()).await;

    let empty: Vec<SettlementJobSummary> = context
        .admin_client
        .request("admin_listSettlementJobs", rpc_params![])
        .await
        .expect("empty list must succeed");
    assert!(empty.is_empty());

    // A pending job with one errored attempt, linked to a certificate.
    let pending_id = seed_pending_job(&context, 2);
    let certificate_id = CertificateId::new(Digest::from([2u8; 32]));
    context
        .state_store
        .insert_certificate_settlement_job_id(&certificate_id, &pending_id)
        .expect("link insert must succeed");
    context
        .state_store
        .insert_settlement_attempt(&pending_id, 0, &mk_attempt(2))
        .expect("attempt insert must succeed");
    context
        .state_store
        .record_settlement_attempt_result(
            &pending_id,
            0,
            &SettlementAttemptResult::ClientError(ClientError {
                kind: ClientErrorType::Unknown,
                message: "rpc flake".to_string(),
            }),
        )
        .expect("attempt result insert must succeed");

    // A completed job without attempts.
    let completed_id = seed_pending_job(&context, 3);
    context
        .state_store
        .insert_settlement_job_result(
            &completed_id,
            &SettlementJobResult {
                wallet: agglayer_types::Address::from([3u8; 20]),
                nonce: Nonce(3),
                attempt_number: SettlementAttemptNumber(0),
                contract_call_result: ContractCallResult {
                    outcome: ContractCallOutcome::Success,
                    metadata: Vec::new().into(),
                    block_hash: B256::from([3u8; 32]),
                    block_number: 3,
                    tx_hash: SettlementTxHash::new(Digest::from([3u8; 32])),
                },
            },
        )
        .expect("job result insert must succeed");

    let jobs: Vec<SettlementJobSummary> = context
        .admin_client
        .request("admin_listSettlementJobs", rpc_params![])
        .await
        .expect("list must succeed");
    assert_eq!(jobs.len(), 2);

    let pending = jobs
        .iter()
        .find(|job| job.job_id == pending_id)
        .expect("pending job must be listed");
    assert_eq!(pending.status, SettlementJobStatus::Pending);
    assert_eq!(pending.certificate_id, Some(certificate_id));
    assert!(!pending.has_live_task);
    assert_eq!(pending.attempt_count, 1);
    assert_eq!(
        pending
            .latest_attempt
            .as_ref()
            .expect("latest attempt must be set")
            .attempt_number,
        0
    );
    assert!(pending
        .last_error
        .as_ref()
        .expect("last error must be set")
        .contains("rpc flake"));

    let completed = jobs
        .iter()
        .find(|job| job.job_id == completed_id)
        .expect("completed job must be listed");
    assert_eq!(completed.status, SettlementJobStatus::Completed);
    assert_eq!(completed.certificate_id, None);
    assert_eq!(completed.last_error, None);
}

#[test_log::test(tokio::test)]
async fn get_settlement_job_returns_detail_with_attempts() {
    let context = TestContext::new_with_config(TestContext::get_default_config()).await;
    let job_id = seed_pending_job(&context, 4);
    context
        .state_store
        .insert_settlement_attempt(&job_id, 0, &mk_attempt(4))
        .expect("attempt insert must succeed");

    let detail: SettlementJobDetail = context
        .admin_client
        .request("admin_getSettlementJob", rpc_params![job_id])
        .await
        .expect("get must succeed");
    assert_eq!(detail.job_id, job_id);
    assert_eq!(detail.status, SettlementJobStatus::Pending);
    assert_eq!(detail.attempts.len(), 1);
    assert_eq!(detail.attempts[0].nonce, 4);
    assert!(detail.attempts[0].result.is_none());
    assert!(detail.job_result.is_none());

    // Live-task flag through the respawn path.
    let () = context
        .admin_client
        .request("admin_reloadAndRestartSettlementTask", rpc_params![job_id])
        .await
        .expect("reload must respawn");
    let detail: SettlementJobDetail = context
        .admin_client
        .request("admin_getSettlementJob", rpc_params![job_id])
        .await
        .expect("get must succeed");
    assert!(detail.has_live_task);
}

#[test_log::test(tokio::test)]
async fn get_settlement_job_unknown_id_is_resource_not_found() {
    let context = TestContext::new_with_config(TestContext::get_default_config()).await;
    let error = context
        .admin_client
        .request::<SettlementJobDetail, _>(
            "admin_getSettlementJob",
            rpc_params![mk_job_id(98)],
        )
        .await
        .expect_err("unknown job must fail");
    assert_error_code(error, crate::error::code::RESOURCE_NOT_FOUND);
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo nextest run -p agglayer-jsonrpc-api settlement_admin`
Expected: the three new tests FAIL with "Method not found".

- [ ] **Step 3: Add trait methods and handlers**

In `admin.rs`, import the DTOs:

```rust
use crate::settlement_admin::{
    build_job_summary, render_last_error, SettlementAttemptDetail, SettlementJobDetail,
    SettlementJobStatus, SettlementJobSummary, SettlementJobResultDto,
};
```

Trait methods:

```rust
    /// List every settlement job known to storage.
    ///
    /// **JSON-RPC method:** `admin_listSettlementJobs`
    ///
    /// One summary per job: certificate link, storage-derived status,
    /// live-task flag, attempt count, latest attempt, and the latest
    /// error if any. A `pending` job with `hasLiveTask: false` is
    /// wedged and needs `admin_reloadAndRestartSettlementTask`.
    /// Full scan; intended for operator use on the admin listener.
    #[method(name = "listSettlementJobs")]
    async fn list_settlement_jobs(&self) -> RpcResult<Vec<SettlementJobSummary>>;

    /// Get one settlement job with its full attempt history.
    ///
    /// **JSON-RPC method:** `admin_getSettlementJob`
    #[method(name = "getSettlementJob")]
    async fn get_settlement_job(
        &self,
        job_id: SettlementJobId,
    ) -> RpcResult<SettlementJobDetail>;
```

Handlers in the `AdminAgglayerServer` impl (`storage_error` maps
`agglayer_storage::error::Error`; reuse `Error::internal`):

```rust
    #[instrument(skip(self))]
    async fn list_settlement_jobs(&self) -> RpcResult<Vec<SettlementJobSummary>> {
        let job_ids = self
            .state
            .list_settlement_job_ids()
            .map_err(|error| Error::internal(error.to_string()))?;
        let mut jobs = Vec::with_capacity(job_ids.len());
        for job_id in job_ids {
            jobs.push(self.read_job_summary(job_id).await?);
        }
        Ok(jobs)
    }

    #[instrument(skip(self))]
    async fn get_settlement_job(
        &self,
        job_id: SettlementJobId,
    ) -> RpcResult<SettlementJobDetail> {
        let job = self
            .state
            .get_settlement_job(&job_id)
            .map_err(|error| Error::internal(error.to_string()))?
            .ok_or_else(|| Error::ResourceNotFound(format!("SettlementJob({job_id})")))?;
        let job_result = self
            .state
            .get_settlement_job_result(&job_id)
            .map_err(|error| Error::internal(error.to_string()))?;
        let attempts = self
            .state
            .list_settlement_attempts(&job_id)
            .map_err(|error| Error::internal(error.to_string()))?;
        let attempt_results = self
            .state
            .list_settlement_attempt_results(&job_id)
            .map_err(|error| Error::internal(error.to_string()))?;
        let certificate_id = self
            .state
            .get_settlement_job_certificate_id(&job_id)
            .map_err(|error| Error::internal(error.to_string()))?;
        let has_live_task = self.settlement_service.has_live_task(job_id).await;

        let attempts = attempts
            .iter()
            .map(|(number, attempt)| {
                let result = attempt_results
                    .iter()
                    .find(|(result_number, _)| result_number == number)
                    .map(|(_, result)| result);
                SettlementAttemptDetail::new(*number, attempt, result)
            })
            .collect();

        Ok(SettlementJobDetail {
            job_id,
            certificate_id,
            status: if job_result.is_some() {
                SettlementJobStatus::Completed
            } else {
                SettlementJobStatus::Pending
            },
            has_live_task,
            contract_address: job.contract_address,
            eth_value: job.eth_value,
            gas_limit: job.gas_limit,
            calldata: job.calldata,
            attempts,
            job_result: job_result.as_ref().map(SettlementJobResultDto::from),
            last_error: render_last_error(&attempt_results),
        })
    }
```

And a private helper in the non-trait impl block of
`AdminAgglayerImpl` (the one with the `where` bounds, next to `start`):

```rust
    /// Read everything one list row needs from storage and the service.
    async fn read_job_summary(
        &self,
        job_id: SettlementJobId,
    ) -> Result<SettlementJobSummary, Error> {
        let job_result = self
            .state
            .get_settlement_job_result(&job_id)
            .map_err(|error| Error::internal(error.to_string()))?;
        let attempts = self
            .state
            .list_settlement_attempts(&job_id)
            .map_err(|error| Error::internal(error.to_string()))?;
        let attempt_results = self
            .state
            .list_settlement_attempt_results(&job_id)
            .map_err(|error| Error::internal(error.to_string()))?;
        let certificate_id = self
            .state
            .get_settlement_job_certificate_id(&job_id)
            .map_err(|error| Error::internal(error.to_string()))?;
        let has_live_task = self.settlement_service.has_live_task(job_id).await;
        Ok(build_job_summary(
            job_id,
            certificate_id,
            has_live_task,
            job_result.as_ref(),
            &attempts,
            &attempt_results,
        ))
    }
```

- [ ] **Step 4: Run the tests**

Run: `cargo nextest run -p agglayer-jsonrpc-api`
Expected: all PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/agglayer-jsonrpc-api
git commit -m "feat(jsonrpc-api): admin settlement job list and detail reads"
```

### Task 12: operator documentation and PR 3

**Files:**
- Create: `docs/knowledge-base/src/settlement-operations.md`
- Modify: `docs/knowledge-base/src/SUMMARY.md`

- [ ] **Step 1: Write the ops chapter**

Create `docs/knowledge-base/src/settlement-operations.md` with an
"unstick a settlement job" section. Content requirements
(write real prose with semantic line breaks, one `#` heading):

- Intro: the settlement admin methods live on the private admin
  listener (`admin_rpc_addr`, default port 9091, no auth beyond
  network placement).
- A table mapping issue 1675's scenarios to calls:
  scenario 1 (job looks stuck) to `admin_listSettlementJobs` and
  `admin_getSettlementJob` (a pending job with `hasLiveTask: false` is
  wedged); scenario 2 (transient wedge) to
  `admin_reloadAndRestartSettlementTask`; scenario 3 (stop now) to
  `admin_abortSettlementTask`, noting the job stays pending and must be
  reloaded later; scenarios 4 and 5 (external transaction, wrong
  result) marked "mutation methods, PR 1663, not yet available".
- One worked `curl` example per available method, e.g.:

```bash
curl -s -X POST http://127.0.0.1:9091/ \
  -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"admin_listSettlementJobs","params":[]}'
```

- The abort-then-reload unstick cycle spelled out as a numbered
  procedure (inspect, abort, verify with get, reload, verify
  `hasLiveTask` is true again).

Add to `docs/knowledge-base/src/SUMMARY.md` after the Storage line:

```markdown
- [Settlement Operations](settlement-operations.md)
```

- [ ] **Step 2: Build the book**

Run: `mdbook build docs/knowledge-base/`
Expected: clean build.

- [ ] **Step 3: Commit, verify, open PR 3** (with user approval)

```bash
git add docs/knowledge-base
git commit -m "docs(knowledge-base): settlement operations runbook for the admin floor"
cargo +nightly fmt --check
cargo clippy -p agglayer-jsonrpc-api --all-targets -- -D warnings
cargo nextest run -p agglayer-jsonrpc-api
```

Title: `feat(settlement): admin RPC reads for settlement jobs`
Base: `feat/1675-settlement-admin-task-controls`.
Body: link issue 1675 and the spec; call out that this closes the
floor of issue 1675 (reads + task controls) and that scenarios 4-5
stay with PR 1663.

---

## Post-plan notes for the executor

- **PR 1663 alignment:** if PR 1663 lands first, Tasks 7 and 8 mostly
  disappear (same plumbing); rebase and keep only the deltas
  (`reloadAndRestartSettlementTask` naming per spec D1, typed errors).
- **Flaky-risk spots:** the two API tests that poll `has_live_task`
  use a 10 s timeout; if CI is slow, raise the timeout, never sleep a
  fixed amount and assert.
- **Do not** move inline `mod tests` to sibling files in these PRs;
  that refactor belongs to PR 1663 and would inflate the diff.
