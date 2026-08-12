# Settlement operations

This runbook maps the five settlement-job recovery scenarios from
[#1675](https://github.com/agglayer/agglayer/issues/1675) to the private
`admin` JSON-RPC methods that resolve them.
These methods inspect or edit stored settlement state, or control a live settlement task.
They can cause an L1 transaction to be stopped, replaced, or re-driven.

The admin listener binds to `rpc.host` and `rpc.admin-port` and defaults to port `9091`.
`AGGLAYER_ADMIN_PORT` overrides the port.
The listener has no authentication beyond network placement, so do not expose it publicly.
Before overriding an outcome, verify the relevant transaction and nonce on L1.
The settlement contract's replay protection is the final double-settlement backstop for an
incorrect operator assertion.

The examples use positional parameters in their wire order and target
`http://127.0.0.1:9091/` by default.

## Unstick a settlement job

### Choose the recovery path

| Scenario from #1675 | Method | Effect and recovery |
|---|---|---|
| 1. A job looks stuck and must be inspected | `admin_listSettlementJobs`, then `admin_getSettlementJob(job_id)` | Find the job and inspect its task liveness, attempts, errors, and result. A pending job with `hasLiveTask: false` is wedged; call reload. |
| 2. A job is wedged for a transient reason | `admin_abortSettlementTask(job_id)`, then `admin_reloadSettlementTask(job_id)` | Stop the stale task, wait for teardown, and respawn it from storage. |
| 3. A job blocks a wallet's nonce pipeline | `admin_abortSettlementTask(job_id)` | Stop the in-memory task without changing stored state. Reload it when it is safe to continue. |
| 4. A transaction was handled outside the node | `admin_insertSettlementAttempt(job_id, attempt, force?)` or `admin_markSettlementAttemptDefinitelyFailed(job_id, attempt_number, reason, force?)` | Register the external transaction, or record the trusted assertion that an existing attempt cannot land. |
| 5. An attempt or completed-job result is wrong | `admin_removeSettlementAttemptResult(job_id, attempt_number, force?)` or `admin_forceRemoveSettlementJobResult(job_id)` | Remove the wrong attempt result, or un-complete and immediately re-drive the whole job. Correct attempt rows before force-removing a completed-job result. |

### Scenario 1: find and inspect a job

Call `admin_listSettlementJobs` first.
Each readable summary carries:

- `jobId`, the optional `certificateId`, and storage-derived `status`;
- `hasLiveTask`, `attemptCount`, and the latest attempt's number, wallet, nonce, and transaction
  hash;
- `lastError`, a human-readable rendering of the latest recorded attempt result when it is a
  client error or L1 revert.

If one job's related storage records cannot be read, the list keeps the other jobs visible.
The failed row instead carries `jobId`, `status: "unreadable"`, and `error` with the full
contextual storage error.

Then call `admin_getSettlementJob` for full detail.
It carries `jobId`, `certificateId`, `status`, `hasLiveTask`, the contract address, ETH value,
gas limit, calldata, every attempt and recorded attempt result, the optional terminal
`jobResult`, and `lastError`.
An attempt includes `attemptNumber`, `senderWallet`, `nonce`, `txHash`,
`submissionTimeUnixSecs`, `maxFeePerGas`, `maxPriorityFeePerGas`, and `result`.

A `pending` job with `hasLiveTask: false` has durable state but no registered in-memory task.
It is wedged and needs `admin_reloadSettlementTask`.
A completed job normally has no live task, so `hasLiveTask: false` is expected there.

Both reads are point-in-time and not transactional.
A job that completes during a read can briefly appear pending without a live task, and task
liveness can change immediately after it is sampled.
Re-read the detail before acting on a surprising value.
The list call scans every settlement job and performs per-job lookups; use it for operator
diagnosis rather than high-frequency polling.

### Scenario 2: abort, inspect, and respawn a wedged task

The following sequence is copy-pasteable after replacing `JOB_ID` with a value returned by the
list call:

```bash
ADMIN_RPC_URL="${ADMIN_RPC_URL:-http://127.0.0.1:9091/}"
JOB_ID='<job-id-from-list>'

curl -sS -X POST "$ADMIN_RPC_URL" \
  -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"admin_listSettlementJobs","params":[]}'

curl -sS -X POST "$ADMIN_RPC_URL" \
  -H 'content-type: application/json' \
  -d "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"admin_getSettlementJob\",\"params\":[\"$JOB_ID\"]}"

curl -sS -X POST "$ADMIN_RPC_URL" \
  -H 'content-type: application/json' \
  -d "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"admin_abortSettlementTask\",\"params\":[\"$JOB_ID\"]}"

curl -sS -X POST "$ADMIN_RPC_URL" \
  -H 'content-type: application/json' \
  -d "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"admin_getSettlementJob\",\"params\":[\"$JOB_ID\"]}"

curl -sS -X POST "$ADMIN_RPC_URL" \
  -H 'content-type: application/json' \
  -d "{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"admin_reloadSettlementTask\",\"params\":[\"$JOB_ID\"]}"

curl -sS -X POST "$ADMIN_RPC_URL" \
  -H 'content-type: application/json' \
  -d "{\"jsonrpc\":\"2.0\",\"id\":6,\"method\":\"admin_getSettlementJob\",\"params\":[\"$JOB_ID\"]}"
```

The abort response is JSON `null` on success.
The first detail read after abort must show `status: "pending"` and
`hasLiveTask: false` before reload can safely respawn the task.
If it still shows a live task, teardown is in progress; wait and repeat the detail read.

Reload creates a fresh task and result watcher when no task is registered.
If reload overlaps abort teardown, it returns `Unavailable`; retry after the detail read reports
`hasLiveTask: false`.
The final detail read should show `hasLiveTask: true`, unless the respawned task already completed
the job, in which case `status: "completed"` is the successful outcome.

When a task is still live, reload instead queues an in-task reload command.
It does not interrupt an L1 wait, so a successful response is not a promptness guarantee.
Unknown job IDs return `NotFound`; completed jobs return `AlreadyCompleted` because their stored
terminal results remain authoritative.

### Scenario 3: release a wallet nonce pipeline

Call `admin_abortSettlementTask` with the job ID:

```json
["<job-id>"]
```

Abort is runtime-only and leaves the pending job in storage.
After the cause of the blockage is safe or corrected, wait for `hasLiveTask: false` and call
`admin_reloadSettlementTask` to spawn a fresh task without restarting the node.

A live certificate waiter observes the abort as a closed watcher and errors.
The certificate can therefore move to `InError` even though a later respawn settles the stored job
successfully on L1.
After recovery, compare the certificate state, stored settlement result, and L1 outcome, then
reconcile that divergence manually.

### Scenario 4: register or abandon an externally handled transaction

To register a transaction submitted outside the node, call
`admin_insertSettlementAttempt`:

```json
[
  "<job-id>",
  { "txHash": "0x<32-byte-transaction-hash>" }
]
```

`txHash` is the only always-required request field.
When the transaction is available from the configured L1 RPC, the service uses its sender and
nonce as authoritative values and resolves omitted fees.
If L1 does not know the transaction, pass both `senderWallet` and `nonce`; otherwise the request
returns `NotFound`.
Optional fields are `submissionTimeUnixSecs`, `maxFeePerGas`, and
`maxPriorityFeePerGas`.
The store appends the attempt and assigns its `attemptNumber`; it never overwrites an existing
attempt.

To assert that a recorded attempt will never land, call
`admin_markSettlementAttemptDefinitelyFailed`:

```json
[
  "<job-id>",
  7,
  "wallet rotated and nonce is no longer usable"
]
```

The `reason` is mandatory and is stored in an `abandonedByAdmin` client-error result.
Make this assertion only after confirming that the transaction cannot land.
Once the task reloads the edit, it can stop waiting on that attempt and drive settlement
elsewhere.

### Scenario 5: correct a recorded result

To undo an attempt result, call `admin_removeSettlementAttemptResult`:

```json
[
  "<job-id>",
  7
]
```

The task treats that attempt as pending again and re-derives its outcome from L1.

#### Correct, then remove

To fix a wrongly completed job, first correct its attempt rows with the trailing literal
`"force=true"` while the terminal job result still blocks re-driving.
Only then call `admin_forceRemoveSettlementJobResult`.
That call removes the job result and immediately spawns a task, which re-derives the outcome from
the stored attempts.

Use the forced form appropriate to the correction.

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

That last call is `admin_forceRemoveSettlementJobResult` and returns JSON `null`.
Unlike the three attempt mutations, it does not return an `attemptNumber` or `liveTask` field.

Do not reverse this order.
Once the job result is removed, the new task can immediately re-record a result from the still
incorrect attempt rows.
Outstanding callers that already hold the completed job's result watcher are not revoked.
Ensure certificate processing for the associated job is quiesced before force-removing the result,
or a certificate task can act on the removed result while the fresh settlement task re-drives the
job.

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
| `queued` | A reload command was queued for the live task. This is not a wake-up or promptness guarantee; the task handles it at a later run-loop control check. | Verify the job's subsequent behavior before relying on the edit. Use abort, wait for teardown, then reload when prompt application matters. |
| `absent` | The edit is durable, but no live task exists. This is expected for a forced edit of a completed job. | For a pending job, call `admin_reloadSettlementTask`, which now respawns dead tasks. For a completed job, make all corrections first, then call `admin_forceRemoveSettlementJobResult`. |
| `notify-failed` | The edit is durable, but the registered task was cancelled or could not accept the reload command and can act on stale memory. | Call `admin_reloadSettlementTask`; it respawns a dead task. If a live task is wedged, abort it, wait for `hasLiveTask: false`, then reload. |

`admin_abortSettlementTask`, `admin_reloadSettlementTask`, and
`admin_forceRemoveSettlementJobResult` return JSON `null` on success.

## Error contract for automation

Branch on the top-level numeric JSON-RPC error `code`.
Do not branch on `message`: it contains human-readable report context and is not a stable
interface.
The serialized tag in `data.classified.code` is useful for logs, but the numeric code is the
script contract.

`RpcErrorCode` in `agglayer-types` is the sole allocator for application error codes across the
node.
Each variant owns its numeric code and kebab-case tag.
An unclassified error fails closed as the standard JSON-RPC internal error `-32603`.

This table is rendered from `crates/agglayer-types/src/rpc_error_code.rs` and includes the
pre-existing public-API allocations because the enum is node-wide.

| Variant | `code()` | `tag()` | Meaning |
|---|---:|---|---|
| `InvalidParams` | `-32602` | `invalid-params` | A supplied value conflicts with authoritative state. Structurally invalid JSON-RPC parameters also use standard code `-32602`. |
| `RollupNotRegistered` | `-10001` | `rollup-not-registered` | The rollup is not registered. |
| `SignatureMismatch` | `-10002` | `signature-mismatch` | Rollup signature verification failed. |
| `ValidationFailure` | `-10003` | `validation-failure` | Proof or state validation failed. |
| `SettlementError` | `-10004` | `settlement-error` | L1 settlement failed. |
| `StatusError` | `-10005` | `status-error` | Transaction status retrieval failed. |
| `SendCertificate` | `-10006` | `send-certificate` | Certificate submission failed. |
| `RateLimited` | `-10007` | `rate-limited` | Transaction settlement was rate-limited. |
| `NotFound` | `-10008` | `not-found` | The referenced job, attempt, attempt result, L1 transaction, or certificate header does not exist. |
| `MethodDisabled` | `-10009` | `method-disabled` | The method is permanently disabled. |
| `AlreadyCompleted` | `-10010` | `already-completed` | The job has a terminal result and the operation was not forced, or reload/abort targeted a completed job. |
| `NotCompleted` | `-10011` | `not-completed` | The operation requires a terminal job result, but none exists. |
| `NoLiveTask` | `-10012` | `no-live-task` | Abort targeted a pending job with no registered task. Use reload to respawn it. |
| `TaskStillLive` | `-10013` | `task-still-live` | The operation requires the task to be gone, but it is still live. Abort or wait for teardown first. |
| `Unavailable` | `-10014` | `unavailable` | A transient L1, storage-reload, task-teardown, or command-queue condition occurred; retry later. |

## Follow-on operations

Pause/resume and full quiesce, a durable admin audit log, and public read exposure remain out of
scope for now.
Per-job keying of the service admin-operation lock and list pagination are also possible
follow-ups as the operator surface grows.
