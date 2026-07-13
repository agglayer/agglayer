# Settlement admin floor: reads and task controls (issue 1675) — design

- Issue: <https://github.com/agglayer/agglayer/issues/1675>
- Related: PR 1663 (settlement admin mutations, WIP draft), and the
  mutations design (`2026-07-07-settlement-admin-mutations-design.md`)
  plus full admin API design (`2026-06-08-settlement-admin-api-design.md`),
  both currently only on branch `design/split-settlement-admin-mutations`.
- Status: approved design, pre-implementation
- Date: 2026-07-13

## Context

The settlement service (`crates/agglayer-settlement-service`) drives one L1
contract call per `SettlementJob` until it lands.
A job is pending while storage holds no `SettlementJobResult` row,
and completed once that row exists.
Every pending job is supposed to have one live `SettlementTask` driving it.

When a job misbehaves on a live network,
an operator today has no way to see or influence it
short of grepping logs and restarting the node.
Issue 1675 defines the admin surface and its priority order:
the reads and the task controls are the floor,
the mutation and undo methods come after.
PR 1663 already drafts the mutation slice;
this document covers only the floor, sliced for review efficiency.

The internal hooks for the controls already exist:
`SettlementService::admin_abort_task` and
`SettlementService::admin_reload_and_restart_task`
(`crates/agglayer-settlement-service/src/settlement_service.rs`).
The reads can be served from `StateStore`,
which already implements `SettlementReader`,
and which the admin RPC already holds.

## Goals

- Ship `admin_listSettlementJobs`, `admin_getSettlementJob`,
  `admin_abortSettlementTask`, and `admin_reloadAndRestartSettlementTask`
  as JSON-RPC methods on the private admin listener (`admin_rpc_addr`).
- Give abort a recovery story:
  reload-and-restart must revive a pending job whose task is dead.
- Keep each PR small and single-purpose; three PRs total.
- Land plumbing in the exact shape of PR 1663
  so the mutation slice rebases cleanly on top.

## Non-goals

- The mutation and undo methods
  (`admin_insertSettlementAttempt`, `admin_markSettlementAttemptDefinitelyFailed`,
  `admin_removeSettlementAttemptResult`, `admin_forceRemoveSettlementJobResult`).
  They stay with PR 1663.
- Pause/resume, quiesce, audit logging, public read exposure,
  and any auth on the admin listener
  (network placement gates it, as for the certificate admin methods).
- Pagination or filtering on the list call.
  Job counts are expected to stay small; noted as future work.

## API surface

All methods live in the `admin` JSON-RPC namespace,
served only on the private admin listener,
next to `admin_forceEditCertificate` and friends.

| Method | Semantics |
|---|---|
| `admin_listSettlementJobs()` | One summary per stored job. |
| `admin_getSettlementJob(job_id)` | Full detail for one job, attempts included. Errors if unknown. |
| `admin_abortSettlementTask(job_id)` | Cancel the in-memory task. Errors if no task runs. The job stays pending in storage. |
| `admin_reloadAndRestartSettlementTask(job_id)` | Live task: send the existing `ReloadAndRestart` command. Dead task on a pending job: load from storage and respawn. Errors if the job is completed or unknown. |

`job_id` is the ULID string form of `SettlementJobId`
(already `serde(transparent)`).

### Response shapes

`admin_listSettlementJobs` returns an array of job summaries:

```json
{
  "jobId": "01JZX...",
  "certificateId": "0x... | null",
  "status": "pending | completed",
  "hasLiveTask": true,
  "attemptCount": 3,
  "latestAttempt": {
    "attemptNumber": 2,
    "senderWallet": "0x...",
    "nonce": 41,
    "txHash": "0x..."
  },
  "lastError": "string | null"
}
```

`lastError` is a human-readable rendering of the most recent attempt result
that represents a failure (client error or on-chain revert),
and null when the latest recorded state carries no failure.

`admin_getSettlementJob` returns the summary fields plus:

- job parameters: `contractAddress`, `ethValue`, `gasLimit`,
  `calldata` as hex (admin-only listener, so size is acceptable);
- `attempts`: every attempt with `attemptNumber`, `senderWallet`, `nonce`,
  `txHash`, `submissionTimeUnixSecs`, `maxFeePerGas`, `maxPriorityFeePerGas`,
  and its recorded result when present;
- `jobResult` when completed: `wallet`, `nonce`, `attemptNumber`,
  and the contract call outcome.

Timestamps are unix seconds
(`submissionTimeUnixSecs`, matching the PR 1663 insert parameters),
and `U256` values (`ethValue`, fees) use their alloy JSON form,
hex quantity strings.

## Design decisions

### D1 — Task-centric names for the controls

Issue 1675 writes `admin_abortSettlementJob` and
`admin_reloadAndRestartSettlementJob`.
Abort only stops the in-memory task; the job stays pending in storage,
and jobs are intentionally infallible.
The names `admin_abortSettlementTask` and
`admin_reloadAndRestartSettlementTask` say what actually happens,
match the service methods on main
(`admin_abort_task`, `admin_reload_and_restart_task`),
and match the naming already drafted in PR 1663.
`reloadAndRestart` is kept over PR 1663's shorter `reloadSettlementTask`
because the method now also restarts dead tasks (D2).

### D2 — Reload-and-restart revives dead tasks

On main, `admin_reload_and_restart_task` only signals a live task.
After an abort, or after a task crash,
a pending job has no task and nothing short of a node restart revives it.
The service doc comment already flags this gap.

The extended semantics:

- live task: send `TaskAdminCommand::ReloadAndRestart`, as today;
- no live task, job pending in storage:
  `SettlementTask::load` plus `spawn_settlement_task`;
- job completed or unknown: typed error.

This makes abort, inspect or fix, then reload-and-restart
a complete unstick cycle.
One subtlety to pin down with a test:
a waiter that grabbed its watch receiver before the abort is stranded
when the aborted task drops its sender (pre-existing abort behavior,
watch channels cannot be re-attached).
The respawn registers a fresh watcher,
so `retrieve_settlement_result` called after the respawn
returns a functioning watcher;
stranded waiters recover by retrieving again.
Concurrent respawns are serialized by a service-level admin operation lock
so two reload calls cannot spawn two tasks for the same job.

### D3 — Status from storage, liveness from the service

`status` is derived from storage row presence
(`pending` without a `SettlementJobResult` row, `completed` with one),
which keeps storage the single source of truth.
`hasLiveTask` comes from the service's `task_controls` registry
through a new `has_live_task(job_id)` accessor.
A pending job with `hasLiveTask: false` is exactly the wedged case
operators need to spot (issue scenarios 1 and 2).
The flag is advisory: it can race with task completion,
which is acceptable for a diagnostic surface.

### D4 — Concrete service handle, PR 1663-shaped plumbing

`AdminAgglayerImpl` gains an `L1Provider` generic parameter and a
`settlement_service: SettlementService<L1Provider, StateStore>` field,
`node.rs` keeps a clone of the service before the orchestrator consumes
the `Arc`, and `agglayer-jsonrpc-api` gains the
`agglayer-settlement-service` dependency.
This is byte-for-byte the plumbing PR 1663 uses,
so the mutation slice rebases onto identical scaffolding.
A trait abstraction was considered and rejected:
one production implementation, and divergence from the takeover branch
would create rebase friction for no testing benefit
(the test harness already builds a real service on a mock alloy provider).

### D5 — DTOs live at the RPC boundary

Settlement domain types in `agglayer-types` keep no serde derives
(only `SettlementJobId` has them, and stays as is).
The JSON representation is owned by a new `settlement_admin` module in
`agglayer-jsonrpc-api`, following the `TokenBalanceEntry` pattern:
camelCase serde structs converted from the domain types.
This keeps wire compatibility concerns out of the domain layer
and gives the RPC surface freedom to render
(hex encoding, human-readable `lastError`).

### D6 — Typed errors on the service admin methods

The two admin methods on `SettlementService` currently return
`eyre::Result`, which the RPC layer could only map to one opaque
internal error.
A small `SettlementAdminError` enum (thiserror) with variants such as
`JobNotFound`, `JobCompleted`, `NoLiveTask`, and `Storage`
lets the RPC layer answer operators precisely:
"no task is running" and "no such job" need different responses.
New RPC error variants get snapshot coverage in the existing
`tests/errors.rs` style.

### D7 — Reverse link reader in storage, and its missing writer

The `certificate_id_per_settlement_job_id_cf` column exists in the schema
but nothing writes it and no reader exposes it:
`insert_certificate_settlement_job_id` only populates the forward
(certificate to job) column.
Two changes:

- `StateWriter::insert_certificate_settlement_job_id` writes both columns
  in one RocksDB write batch;
- `SettlementReader` gains
  `get_settlement_job_certificate_id(job_id) -> Option<CertificateId>`,
  implemented on `StateStore`, mocked in `MockStateStore`,
  and covered by store tests.

Jobs linked before this change lack the reverse row,
so their `certificateId` reads as null; no backfill in the floor
(the forward column could seed one later if operators need it).
The list and detail responses use the reader to show `certificateId`.

## Delivery plan: three PRs

Sizes include tests.

1. **Groundwork** (service and storage, no RPC changes, ~300-400 lines):
   `SettlementAdminError`, `has_live_task`,
   the respawn path in `admin_reload_and_restart_task` (D2, D6),
   the reverse link dual write and reader (D7), mocks,
   unit and store tests.
2. **Controls** (~350-450 lines):
   RPC plumbing (D4) in `admin.rs`, `node.rs`, and `testutils.rs`;
   `admin_abortSettlementTask` and `admin_reloadAndRestartSettlementTask`
   as thin adapters; error mapping and snapshots;
   API-level tests through `TestContext::admin_client`
   covering abort, reload of a live task, and respawn of a dead one.
3. **Reads and ops doc** (~500-650 lines):
   `settlement_admin` DTO module (D5), `admin_listSettlementJobs`,
   `admin_getSettlementJob`, API-level tests
   (empty list, pending with live task, aborted without task,
   completed, unknown id),
   and an "unstick a settlement job" ops section mapping issue
   scenarios 1-3 to these calls and pointing at the mutation slice
   for scenarios 4-5.

Each PR compiles, passes tests, and carries no dead code:
the groundwork PR is consumed by its own layer's tests,
and the RPC handle lands together with its first two consumers.

## Testing

- Unit tests at the service layer for `has_live_task`,
  the respawn path, and the watcher behavior across respawn (D2).
- Store tests for the reverse link reader.
- API-level tests in `agglayer-jsonrpc-api` via the existing
  `TestContext` harness (real RocksDB temp stores, mock alloy provider,
  real HTTP admin server), matching the issue's "covered by API-level
  tests" requirement.
- Snapshot tests for every new RPC error encoding.

## Risks and open points

- Respawn versus in-flight waiters (D2) is the one piece of genuinely new
  concurrency behavior; it gets a dedicated test before the RPC exposes it.
- `admin_listSettlementJobs` scans all job ids and performs per-job
  lookups across several column families.
  Fine at expected volumes; pagination is future work.
- The admin listener has no auth.
  Unchanged from the existing `admin_force*` methods, and the floor adds
  no mutation beyond task lifecycle, but worth restating in the ops doc.
- PR 1663 renames on rebase: its `admin_reloadSettlementTask` becomes
  `admin_reloadAndRestartSettlementTask` (D1), and its plumbing commit
  drops out as already landed.
