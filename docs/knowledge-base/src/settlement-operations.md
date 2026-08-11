# Settlement operations

This runbook maps the seven settlement-job recovery scenarios from
[#1675](https://github.com/agglayer/agglayer/issues/1675) to the operations
available in the current phase.
Scenario 1 still awaits discovery reads.
Scenarios 2–7 are covered by the shipped reload and abort controls and state
mutations.
Use these methods only through the private `admin` JSON-RPC listener.
They edit stored settlement state or control a live task and can cause an L1
transaction to be re-driven.
Before overriding an outcome, verify the relevant transaction and nonce on L1;
the settlement contract's replay protection is the final double-settlement
backstop for an incorrect operator assertion.

The examples below show positional parameters in their wire order.
The API does not yet provide settlement-job discovery reads, so obtain the job
ID and attempt number from existing logs or storage inspection.

## Unstick a settlement job

### Choose the recovery path

| Failure scenario | Method | Effect and recovery |
|---|---|---|
| 1. A job looks stuck and must be inspected | Not shipped: `admin_listSettlementJobs` and `admin_getSettlementJob` | Use existing logs or storage inspection until the read phase adds job discovery and detail. |
| 2. A task is wedged or missing | `admin_reloadSettlementTask(job_id)` | Reloads a live task from storage or respawns a missing task for a pending job. Retry if task teardown is still in progress. |
| 3. A job blocks a wallet's nonce pipeline | `admin_abortSettlementTask(job_id)` | Stops the in-memory task without changing stored state. Inspect or fix the job, then reload it to spawn a fresh task. |
| 4. A transaction was handled outside the node | `admin_insertSettlementAttempt(job_id, attempt, force?)` | Registers the external transaction as a stored attempt. Only `txHash` is required when L1 returns the transaction; the service resolves identity and available fees from L1. |
| 5. An attempt will never land | `admin_markSettlementAttemptDefinitelyFailed(job_id, attempt_number, reason, force?)` | Records a trusted terminal outcome for that attempt, then lets the job drive settlement elsewhere. |
| 6. An attempt result is wrong | `admin_removeSettlementAttemptResult(job_id, attempt_number, force?)` | Removes the result so the task re-derives it from L1. |
| 7. A completed job result is wrong | `admin_forceRemoveSettlementJobResult(job_id)` | Removes the terminal job result and immediately spawns a task that re-derives the result from stored attempts. |

### Scenario 1: find and inspect a job

The admin API cannot currently list settlement jobs or inspect one by ID.
Do not look for `admin_listSettlementJobs` or `admin_getSettlementJob` in this
phase; obtain the job ID and attempt details from existing logs or storage
inspection.

The follow-up read phase can replace that temporary discovery step with the
two admin reads without changing the recovery procedures below.

### Scenario 2: reload or respawn a task

Call `admin_reloadSettlementTask` with the job ID:

```json
["<job-id>"]
```

The method queues a command that makes an existing task drop its in-memory
state, reload from storage, and restart its run loop.
The command does not interrupt an L1 wait, so a successful RPC response is not
a promptness guarantee.

If no task is registered, the method loads the pending job from storage and
spawns a fresh task and result watcher.
If the call overlaps task teardown, it returns `Unavailable` rather than
spawning before the old task finishes cleanup.
Retry the call after teardown completes.
Unknown job IDs return `NotFound`, and completed jobs return
`AlreadyCompleted` because their stored results still stand.

### Scenario 3: release a wallet nonce pipeline

Call `admin_abortSettlementTask` with the job ID:

```json
["<job-id>"]
```

The abort is runtime-only.
It leaves the pending job in storage.
Inspect and fix the cause of the blockage, then call
`admin_reloadSettlementTask` to load the pending job and spawn a fresh task
without restarting the node.
An immediate reload can return `Unavailable` while abort teardown is still in
progress; retry after teardown completes.

A live certificate waiter observes the abort as an error.
The certificate can therefore move to `InError` even though the pending job
is respawned and later settles.
After recovery, compare the certificate state, stored settlement result, and
L1 outcome, then reconcile that divergence manually.

### Scenarios 4 and 5: register or abandon an externally handled transaction

To register a transaction submitted outside the node, call
`admin_insertSettlementAttempt`:

```json
[
  "<job-id>",
  { "txHash": "0x<32-byte-transaction-hash>" }
]
```

`txHash` is the only always-required request field.
When the transaction is available from the configured L1 RPC, the service
uses its sender and nonce as authoritative values and resolves omitted fees.
If L1 does not know the transaction, pass both `senderWallet` and `nonce`;
otherwise the request returns `NotFound`.
Optional fields are `submissionTimeUnixSecs`, `maxFeePerGas`, and
`maxPriorityFeePerGas`.
The store appends the attempt and assigns its `attemptNumber`; it never
overwrites an existing attempt.

To assert that a recorded attempt will never land, call
`admin_markSettlementAttemptDefinitelyFailed`:

```json
[
  "<job-id>",
  7,
  "wallet rotated and nonce is no longer usable"
]
```

The `reason` is mandatory and is stored in the attempt's client-error result.
Make this assertion only after confirming the transaction cannot land.
Once the task observes the edit, it can re-drive the job with another nonce or
wallet.

### Scenarios 6 and 7: correct a recorded result

To undo an attempt result, call `admin_removeSettlementAttemptResult`:

```json
[
  "<job-id>",
  7
]
```

The task treats that attempt as pending again and re-derives its outcome from
L1.

#### Correct, then remove

To fix a wrongly completed job, first correct its attempt results with the
attempt mutations using the trailing literal `"force=true"` **while the
terminal job result still blocks re-driving**.
Only then call `admin_forceRemoveSettlementJobResult`.
The removal immediately respawns the task, which re-derives the job result
from stored attempts.

Use the forced form appropriate to the correction:

Mark an attempt definitely failed:

```json
["<job-id>", 7, "attempt cannot land", "force=true"]
```

Remove an attempt result:

```json
["<job-id>", 7, "force=true"]
```

Insert a replacement attempt:

```json
[
  "<job-id>",
  { "txHash": "0x<32-byte-transaction-hash>" },
  "force=true"
]
```

Then un-complete and re-drive the job:

```json
["<job-id>"]
```

That last call is `admin_forceRemoveSettlementJobResult`.
It returns JSON `null`; unlike the three attempt mutations, it does not return
an `attemptNumber` or `liveTask` field.

Do not reverse this order.
Once the job result is removed, the new task can immediately re-record a
result from the still-incorrect attempt rows.

## Mutation response contract

The three attempt mutations return this shape:

```json
{
  "attemptNumber": 7,
  "liveTask": "queued"
}
```

`liveTask` has three wire values:

| Value | Meaning | Operator action |
|---|---|---|
| `queued` | A reload command was queued for the live task. This is not a wake-up or a promptness guarantee; the task handles it at a later run-loop control check. | Verify the job's subsequent behavior before relying on the edit. |
| `absent` | The edit is durable, but no live task exists. This is expected for a forced edit of a completed job. | Start the task through the matching recovery step: force-remove the job result, or restart the node for a pending job. |
| `notify-failed` | The edit is durable, but the live task could not be notified and can continue from stale memory. | Try `admin_reloadSettlementTask`; if it cannot queue or prompt application matters, call `admin_abortSettlementTask` and restart the node. |

`admin_abortSettlementTask`, `admin_reloadSettlementTask`, and
`admin_forceRemoveSettlementJobResult` return JSON `null` on success.

## Error contract for automation

Branch on the top-level numeric JSON-RPC error `code`.
Do not branch on `message`: it contains human-readable report context and is
not a stable interface.
The optional serialized tag in `data.classified.code` is useful for logs, but
the numeric code is the script contract.

`RpcErrorCode` in `agglayer-types` is the sole allocator for application error
codes across the node.
Every semantic condition has a dedicated code; there is no catch-all
`RpcErrorCode` variant.
An unclassified error fails closed as the standard JSON-RPC internal error
`-32603`.

This table is rendered from `crates/agglayer-types/src/rpc_error_code.rs`:

It includes the pre-existing public-API allocations because the enum is
node-wide, not settlement-specific.

| Variant | `code()` | `tag()` | Meaning |
|---|---:|---|---|
| `InvalidParams` | `-32602` | `invalid-params` | A supplied value conflicts with authoritative state. Structurally invalid JSON-RPC parameters also use the standard `-32602` code. |
| `RollupNotRegistered` | `-10001` | `rollup-not-registered` | The rollup is not registered. |
| `SignatureMismatch` | `-10002` | `signature-mismatch` | Rollup signature verification failed. |
| `ValidationFailure` | `-10003` | `validation-failure` | Proof or state validation failed. |
| `SettlementError` | `-10004` | `settlement-error` | L1 settlement failed. |
| `StatusError` | `-10005` | `status-error` | Transaction status retrieval failed. |
| `SendCertificate` | `-10006` | `send-certificate` | Certificate submission failed. |
| `RateLimited` | `-10007` | `rate-limited` | Transaction settlement was rate-limited. |
| `NotFound` | `-10008` | `not-found` | The referenced job, attempt, attempt result, L1 transaction, or certificate header does not exist. |
| `MethodDisabled` | `-10009` | `method-disabled` | The method is permanently disabled. |
| `AlreadyCompleted` | `-10010` | `already-completed` | The job has a terminal result and the attempt edit was not forced. |
| `NotCompleted` | `-10011` | `not-completed` | The operation requires a terminal job result, but none exists. |
| `NoLiveTask` | `-10012` | `no-live-task` | A pending job has no in-memory task; restart the node so startup recovery respawns it. |
| `TaskStillLive` | `-10013` | `task-still-live` | The operation requires the task to be gone, but it is still live. |
| `Unavailable` | `-10014` | `unavailable` | A transient dependency or task-command-queue failure occurred; retry later. |

## Follow-on operations

The mutation phase intentionally does not ship these operations yet:

- `admin_listSettlementJobs` and `admin_getSettlementJob` discovery reads;
- reload that can respawn a missing task;
- pause, quiesce, or drain controls;
- a durable admin audit log.

Future work can add those procedures to this chapter without changing the
mutation contracts above.
