# Settlement admin mutations (issue 1254) — design

- Issue: <https://github.com/agglayer/agglayer/issues/1254>
- Related: PR 1562 (full settlement admin and observability API design).
  That design is being split in two:
  this document records the **mutation surface**, which is implemented;
  the [PR 1562 document](2026-06-08-settlement-admin-api-design.md)
  keeps the parts that are not implemented yet
  (pause/resume with full quiesce, observability reads, durable audit log).
- Status: implemented
- Date: 2026-07-07

## Context

The settlement service (`crates/agglayer-settlement-service`) drives one L1
contract call per `SettlementJob` until it lands, retrying across nonces and
wallets.
A job is intentionally **infallible**: it has no terminal-failure state,
so the certificate orchestrator can safely iterate on subsequent certificates.
Job state is derived from row presence in storage:
a job is pending while it has no `SettlementJobResult` row,
and completed once that row is written.
Every pending job has one live `SettlementTask` driving it
(spawned at intake or by startup recovery).

Operators need a small set of manual interventions on stored settlement state:

- **Register an attempt the service does not know about** — a settlement
  transaction submitted out-of-band, or ported from the legacy settlement path
  (a pre-existing settlement tx hash recorded before the settlement service
  owned submission).
- **Assert that an attempt will never land** — e.g. the wallet was rotated
  away and the nonce is dead, or an RPC outage prevents the service from ever
  deriving the conclusion itself.
- **Undo a recorded attempt result** — hand the attempt back to the task so
  it re-derives the outcome from L1, e.g. after a wrong assertion.
- **Un-complete a job** — remove a terminal result that should not have been
  recorded, so the job is driven again.

This document explains the API that covers these four needs, and why it is
shaped the way it is.
It deliberately implements the minimum:
no pause/quiesce, no list/inspect observability, no audit log
(see Non-goals).

## Goals

- Ship the four mutations above as JSON-RPC methods on the private admin
  listener, plus the pre-existing abort/reload task controls.
- Preserve the infallible-`SettlementJob` invariant:
  no admin operation produces a terminal job failure.
- Keep storage the single source of truth:
  admin operations edit stored state and tell the live task to reload;
  they never patch task memory.
- Do no harm: precondition-checked, per-job-locked, insert-only where
  overwriting could destroy evidence.

## Non-goals

- Pause/resume (global or per-wallet), quiesce, and drain reporting.
- The observability read surface (`listJobs` / `getJob` / status views).
- The durable admin audit log.
  The mandatory `reason` of mark-definitely-failed is persisted only inside
  the attempt result's error message.
- A job-level terminal-failure state (contradicts the infallible-job
  invariant; unchanged from the PR 1562 design).

These stay covered by the PR 1562 document.

## API surface

All methods live in the `admin` JSON-RPC namespace, served only on the
private admin listener (`admin_rpc_addr`), next to the certificate admin
methods (`admin_forceEditCertificate` and friends).
Implementation: `crates/agglayer-jsonrpc-api/src/settlement_admin.rs`,
thin adapters over `SettlementService::admin_*` methods.

| Method | Semantics |
|---|---|
| `admin_insertSettlementAttempt(job_id, attempt, force?)` | Append one new attempt to the job; returns the assigned attempt number. Never overwrites. |
| `admin_markSettlementAttemptDefinitelyFailed(job_id, attempt_number, reason, force?)` | Overwrite the attempt's result with an operator assertion that it will never land. |
| `admin_removeSettlementAttemptResult(job_id, attempt_number, force?)` | Delete the attempt's result; the task re-derives it from L1. |
| `admin_forceRemoveSettlementJobResult(job_id)` | Delete the job's terminal result and immediately re-drive the job. |
| `admin_abortSettlementTask(job_id)` | Stop the in-memory task (runtime-only; not a terminal state). |
| `admin_reloadSettlementTask(job_id)` | Make the live task drop its memory and reload from storage. |

`attempt` for insert takes `txHash` (mandatory) and optional `senderWallet`,
`nonce`, `submissionTimeUnixSecs`, `maxFeePerGas`, `maxPriorityFeePerGas`.

The three attempt mutations respond with
`{ attemptNumber, liveTask: "notified" | "absent" | "notify-failed" }`;
`liveTask` reports whether the running task was told to reload
(see the mutation model below).
Their optional trailing `force` parameter takes the literal string
`"force=true"` or `"force=false"` (the `admin_forceEditCertificate` style;
omitted means false) and gates edits on completed jobs (see D6).

## Design decisions and rationale

### D1 — Insert is append-only, with store-assigned attempt numbers

`admin_insertSettlementAttempt` always adds one new attempt under the next
unused attempt sequence number (max existing + 1) and returns it.
It is **not** an upsert:
when porting a pre-existing settlement tx hash, overwriting an
already-recorded attempt would silently destroy evidence the task relies on.
Assigning the number in the store (under the per-job write lock) keeps the
operation race-free and spares the operator from picking a slot.
This diverges from the PR 1562 draft, which sketched `admin_upsert_attempt`
with an edit capability; editing is covered by remove-result + insert instead.

### D2 — Missing attempt fields are resolved from L1 by tx hash

Only the transaction hash is mandatory.
A missing sender or nonce is resolved by fetching the transaction from the
L1 RPC (`eth_getTransactionByHash`); an unknown transaction is rejected with
instructions to pass the fields explicitly.
Missing fees fall back to the fetched transaction's fees
(an accurate baseline for the task's fee-bumping retries),
or 0 when the transaction was not fetched
(a retry then starts over from freshly estimated fees).
A missing submission time defaults to now;
it only seeds the retry backoff.
Rationale: the porting flow starts from a tx hash;
requiring the operator to copy sender/nonce/fees by hand is error-prone
for no gain.

### D3 — "Definitely failed" is an attempt-result override, never a job state

A human asserting that an attempt will never land is a terminal client-side
outcome for that attempt, so it maps onto the existing
`SettlementAttemptResult::ClientError` with one new variant:
`ClientErrorType::AbandonedByAdmin`
(proto: `CLIENT_ERROR_TYPE_ABANDONED_BY_ADMIN = 3`).
The write bypasses the regular upgrade-only rule (`can_be_replaced_by`)
through a dedicated storage method.

Re-drive is **emergent**, not a separate command:
once the attempt is no longer pending, the reloaded run loop either
re-drives the same nonce (wallet still under the service's control)
or falls through to a fresh nonce on the default wallet
(wallet rotated away, private key unknown).
The job never fails — it is freed to settle elsewhere,
preserving the infallible-job invariant.

This is a trusted operator assertion:
if the abandoned transaction can still land,
only the settlement contract's replay protection prevents a double
settlement (unchanged assumption from the PR 1562 design).
Real on-chain evidence observed later still supersedes the assertion,
because `ClientError -> ContractCall` remains a legal upgrade.

### D4 — Remove-result is the undo

`admin_removeSettlementAttemptResult` deletes the result row and nothing
else, handing the attempt back to the task as pending.
It exists because admin assertions must be reversible:
a wrong mark-definitely-failed (or any wrongly recorded client error)
is corrected by removing the result and letting the task re-derive the truth
from L1, rather than by hand-writing a replacement result.
It was not in the PR 1562 draft.

### D5 — Force-remove of a job's terminal result

`admin_forceRemoveSettlementJobResult` un-completes a job:
it deletes the `SettlementJobResult` row, drops the in-memory watcher that
still broadcasts the removed result, and spawns a fresh task that re-drives
the job from stored state.
Attempts and their results are untouched;
correct them **first**, with the attempt mutations' `force` parameter,
while the terminal result still blocks the job from being re-driven
(see D6).

Guards, in order:

1. refuse while a live task exists for the job
   (a completed job has none; this blocks mid-completion races
   and misuse on pending jobs);
2. refuse when the job does not exist or has no terminal result;
3. after the removal, respawn the task immediately,
   so the "every pending job has a running task" invariant holds without a
   node restart.

The `force` prefix follows `admin_forceEditCertificate`:
this is the one operation that moves a job backwards,
and if the removed result was real,
double-settle safety again rests on the contract.
It still does not violate the infallible-job direction —
it turns a completed job into a pending one, never the reverse.

### D6 — Mutation model: declarative over stored state, reload to apply

All mutations follow the model locked in the PR 1562 design:

1. the storage method takes the per-job write lock
   (`StateStore::with_settlement_write_lock`),
   checks its preconditions, and writes;
2. the service then tells the live task, if any, to drop its in-memory
   `attempts` map and reload from storage
   (`TaskAdminCommand::ReloadAndRestart`);
3. tasks only ever read storage (start/reload),
   so there is no bespoke in-memory edit path to diverge.

The notification is best-effort, and the response says so honestly:
`liveTask` is `notified` (task will reload), `absent` (no live task;
the edit is picked up whenever one starts), or `notify-failed`
(task still acts on stale memory; `admin_reloadSettlementTask` is the
escape hatch).
For a pending job, anything but `notified` is an anomaly worth
investigating — after node integration (PR 1393),
every pending job normally has a live task.

Every attempt mutation refuses a job that already has a terminal result,
unless its `force` parameter is `"force=true"`:
a completed job is never re-driven,
so editing its attempts could normally only create inconsistencies.
The forced variant exists to prepare
`admin_forceRemoveSettlementJobResult`:
attempt-result corrections must land while the job still has its terminal
result, because removing the result immediately respawns the task,
which could re-derive and re-record the job result from the uncorrected
attempts before the operator gets a chance to adjust them.
On a completed job there is no live task,
so a forced edit reporting `liveTask: "absent"` is the expected state;
the correction is observed by the task that
`admin_forceRemoveSettlementJobResult` spawns.

**Accepted race**: between an admin write and the task observing its reload,
the task can act on pre-edit memory.
For insert, the task may collide with the admin-assigned attempt number and
panic on the insert-only storage write, wedging the job in memory until a
reload or restart; the window is one loop iteration and the failure is loud.
The clean fix is the pause/quiesce mechanism from the PR 1562 design;
until then the race is documented rather than closed.
The other mutations are race-benign (see D7).

### D7 — An admin abandon outranks client-side notes, but yields to L1

`record_settlement_attempt_result` (the regular, task-facing writer) keeps
an `AbandonedByAdmin` result and **reports success** when asked to overwrite
it with another client error (nonce-used / settled-elsewhere notes,
submit failures).
Rationale: a task that has not observed the override yet may legitimately
try to record such a note; refusing would make the task panic on a conflict
it cannot resolve — the panic-loop family fixed by PR 1617 —
and the admin assertion is semantically at least as strong as any
client-side note.
On-chain evidence (`ContractCall`) still replaces the assertion through the
normal upgrade rule, so a wrong assertion self-heals when the transaction
lands after all.

### D8 — Simplicity choices (deviations from the PR 1562 draft)

Reviewed and chosen deliberately when the surface shrank to mutations-only:

- **No separate `SettlementAdminWriter` trait, no `settlement-admin` cargo
  feature.** The four bypass methods live on `SettlementWriter` with an
  `admin_` prefix and a documented contract that the settlement task must
  never call them.
  This trades the draft's compile-time capability containment for one fewer
  trait; the containment was hygiene, not a security boundary
  (cargo features are additive anyway).
- **No tower `AdminCommand` extension.** The JSON-RPC adapters call the
  service's inherent `admin_*` methods directly.
  The tower command-as-data indirection only pays for itself when a
  cross-cutting layer (the audit log) needs a single choke point;
  it should return together with the audit work.
- **`admin` namespace, not a `settlement` namespace.**
  The methods join the existing admin RPC surface
  (one listener, one namespace, one operator entry point)
  as a second jsonrpsee trait merged into the same server.
- **No audit rows.** Deferred with the audit design in PR 1562;
  `reason` survives only inside the abandoned result's message.

## Safety invariants

1. **Infallible job preserved** — no operation adds a terminal job failure;
   force-remove only moves a job from completed back to pending.
2. **Storage is the single source of truth** — tasks apply admin edits by
   reloading; nothing patches task memory.
3. **Per-job atomicity** — every mutation checks and writes under the job's
   settlement write lock.
4. **Evidence is never silently overwritten** — insert is append-only;
   only the explicit override and the two explicit removals touch existing
   rows, each behind its own preconditions.
5. **Double-settle backstop is the contract** — mark-definitely-failed and
   force-remove are trusted operator assertions;
   their safety rests on the L1 settlement contract being replay-safe.
6. **Capability containment is by convention now** — the bypass writers are
   reachable from settlement code as a type-system matter;
   the contract lives in the trait docs (weaker than the draft, see D8).

## Runbooks

- **Port an externally-submitted settlement tx** (legacy path, out-of-band
  replacement): `admin_insertSettlementAttempt(job_id, { txHash })` —
  sender, nonce, and fees resolve from L1; the task tracks the attempt like
  its own.
- **Dead nonce after wallet rotation**: confirm the transaction cannot land,
  `admin_markSettlementAttemptDefinitelyFailed(job_id, n, reason)`;
  the reloaded task re-drives on a fresh nonce/wallet.
- **Wrong assertion**: `admin_removeSettlementAttemptResult(job_id, n)`;
  the task re-derives the attempt's outcome from L1.
- **Wrongly recorded terminal result**: first correct the misleading
  attempt results with `"force=true"` (mark abandoned, remove, or insert),
  then `admin_forceRemoveSettlementJobResult(job_id)`;
  a fresh task re-drives the job immediately from the corrected attempts.
- **Recovery when `liveTask != "notified"`**:
  `admin_reloadSettlementTask(job_id)`, or `admin_abortSettlementTask` and
  let the next restart respawn from storage.

## Key code references

- RPC surface: `crates/agglayer-jsonrpc-api/src/settlement_admin.rs`,
  merged into the admin router in `crates/agglayer-jsonrpc-api/src/admin.rs`
  and wired in `crates/agglayer-node/src/node.rs`.
- Service methods and mutation model:
  `crates/agglayer-settlement-service/src/settlement_service.rs`
  (`NewSettlementAttempt`, `LiveTaskNotification`, `admin_*` methods).
- Storage writers and guards:
  `crates/agglayer-storage/src/stores/interfaces/writer/settlement_writer.rs`
  (trait contract) and
  `crates/agglayer-storage/src/stores/state/settlement/mod.rs`
  (implementation, per-job lock, D7 conflict rule).
- Domain type: `ClientErrorType::AbandonedByAdmin` in
  `crates/agglayer-types/src/settlement.rs`;
  proto in `proto/agglayer/storage/v0/settlement.proto`.
