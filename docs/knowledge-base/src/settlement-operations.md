# Settlement operations

This chapter is a runbook for the settlement admin methods of the Agglayer node.

A settlement job is the stored description of one L1 contract call
that settles a [certificate](glossary.md#certificate).
An in-memory settlement task drives the job:
it submits transaction attempts
and records a terminal result once [settlement](glossary.md#settlement) concludes.

The settlement admin methods live on the private admin JSON-RPC listener,
bound to `rpc.host`:`rpc.admin_port`.
The port defaults to 9091 and the `AGGLAYER_ADMIN_PORT` environment variable
overrides it (`crates/agglayer-config/src/rpc.rs`).
The listener has no authentication beyond network placement:
anyone who can reach the port can call every admin method.
This is the same stance as the certificate admin methods,
which share the listener.

## The settlement admin surface

| Method | Semantics |
|---|---|
| `admin_listSettlementJobs` | One summary row per job known to storage: status, live-task flag, attempt count, latest attempt, and latest error. |
| `admin_getSettlementJob` | Full detail for one job, including the complete attempt history and the terminal result if any. |
| `admin_abortSettlementTask` | Stop the in-memory task of a job; storage is untouched. |
| `admin_reloadAndRestartSettlementTask` | Reload a job from storage and restart its task, or spawn a fresh task when none is live. |

The method definitions live in `crates/agglayer-jsonrpc-api/src/admin.rs`
and the wire types in `crates/agglayer-jsonrpc-api/src/settlement_admin.rs`.

The mutation methods (insert an attempt, mark an attempt failed,
and remove recorded results) are a separate work stream
([PR 1663](https://github.com/agglayer/agglayer/pull/1663))
and are not available yet.

## Unstick a settlement job

The scenarios below come from
[issue 1675](https://github.com/agglayer/agglayer/issues/1675)
and map each situation to the calls that resolve it.

### A job looks stuck

Start with `admin_listSettlementJobs` to see every job,
then call `admin_getSettlementJob` on the suspicious one.

Read a row as follows.
`status` derives from storage:
`pending` while no terminal result row exists, `completed` once one does.
`hasLiveTask` reports whether an in-memory task currently drives the job.
A `pending` job with `hasLiveTask: false` is wedged:
no task is working on it and it will not make progress on its own.
`lastError` renders the most recent attempt failure,
and is null when the latest recorded attempt state is not a failure.

Fields are read point-in-time, not transactionally,
so a job completing concurrently can briefly appear pending without a live task;
the mutation methods re-classify authoritatively,
so acting on a stale row is safe.

### Wedged for a transient reason

Call `admin_reloadAndRestartSettlementTask` on the job.
A live task drops its in-memory state and reloads the job from storage.
A pending job without a live task, for example after an admin abort
or a failed in-task reload, gets a fresh task spawned from storage.
Repeating the call is harmless: reload-and-restart is idempotent recovery.

### A job must stop now

Call `admin_abortSettlementTask` to stop the in-memory task.
The abort is runtime-only.
The job stays pending in storage and nothing is recorded,
so the certificate waiting on the job stays blocked
until a later `admin_reloadAndRestartSettlementTask`.

A reload chained immediately after an abort can be accepted and then dropped,
because the task can exit on the cancellation
without draining its command queue.
Verify with `admin_getSettlementJob` that `hasLiveTask` is false
before reloading.

### External transaction or wrong recorded result

Two scenarios are not covered yet:
settling a job through a transaction sent outside the node,
and correcting a wrong recorded result.
Both need the mutation methods from the separate work stream
([PR 1663](https://github.com/agglayer/agglayer/pull/1663)).

### The abort and reload cycle

1. Inspect the job with `admin_getSettlementJob`.
2. Abort the task with `admin_abortSettlementTask`.
3. Re-inspect until `hasLiveTask` is false.
4. Restart with `admin_reloadAndRestartSettlementTask`.
5. Re-inspect and confirm `hasLiveTask` is true.

## Worked examples

All examples target the default admin listener on port 9091.
Job ids are ULID strings and parameters are positional arrays.
Replace the example job id with a real one from the list output.

List every job:

```bash
curl -s -X POST http://127.0.0.1:9091/ \
  -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"admin_listSettlementJobs","params":[]}'
```

Get one job with its attempt history:

```bash
curl -s -X POST http://127.0.0.1:9091/ \
  -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"admin_getSettlementJob","params":["01K1ZDGVR1V0Q2EXAMPLEULID0"]}'
```

Abort the task of a job:

```bash
curl -s -X POST http://127.0.0.1:9091/ \
  -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"admin_abortSettlementTask","params":["01K1ZDGVR1V0Q2EXAMPLEULID0"]}'
```

Reload a job from storage and restart its task:

```bash
curl -s -X POST http://127.0.0.1:9091/ \
  -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"admin_reloadAndRestartSettlementTask","params":["01K1ZDGVR1V0Q2EXAMPLEULID0"]}'
```

## Error responses

An unknown job id returns code `-10008` (resource not found).
Every other settlement admin failure returns code `-10010`
with a machine-readable variant tag nested under `data.settlement-admin`:
`job-completed`, `no-live-task`, `task-not-responding`,
`reload-failed`, or `storage`.
Scripts should branch on the tag, not on the message,
which is free text and can change.
`crates/agglayer-jsonrpc-api/src/error.rs` defines the codes.

An abort without a live task, for example, fails with:

```json
{
  "code": -10010,
  "data": {
    "settlement-admin": {
      "no-live-task": {
        "job-id": "01K1ZDGVR1V0Q2EXAMPLEULID0"
      }
    }
  },
  "message": "Settlement admin error: No live settlement task for job ..."
}
```
