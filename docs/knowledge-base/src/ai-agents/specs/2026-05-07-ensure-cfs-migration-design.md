# Ensure `ensure_cfs` Migration Recording

## Context

`agglayer-storage` recently introduced `Builder::ensure_cfs()` as a migration
helper for state-store column-family normalization.
The current implementation increments the in-memory migration step when the
requested CFs already exist, but it does not write a migration record for that
no-op path.

The migration framework treats migration history as a contiguous sequence of
step records starting at `0`.
On open, it scans the migration record column family and fails with
`MigrationRecordGap` as soon as any step is missing.

## Problem

An unrecorded no-op `ensure_cfs` step creates a forward-compatibility hazard.
If a future release adds another migration step after `ensure_cfs`, that later
step will be recorded while the earlier no-op step remains absent.
The resulting history contains a hole, and the next reopen fails the gap check.

This behavior makes migration numbering unsafe because the ledger of completed
steps is no longer monotonic or auditable.

## Non-Goal

Supporting reopen or downgrade with older binaries that declare fewer migration
steps is not a requirement for this change.

## Options Considered

### Option 1: Always record `ensure_cfs`

Make `ensure_cfs()` always execute through `perform_step()`.
When every requested CF already exists, the closure returns `Ok(())` without
schema changes, but the migration framework still persists the step record.

Pros:

- Smallest code change.
- Preserves contiguous migration history.
- Keeps `ensure_cfs` semantics explicit in the migration ledger.
- Maintains idempotence for already-current databases.

Cons:

- Older binaries that declare fewer steps may fail with
  `FewerStepsDeclared` after this migration has been recorded.

### Option 2: Treat schema normalization outside the migration ledger

Move the "create missing CFs if absent" logic into open-time schema
normalization before numbered migration-step handling.
`ensure_cfs` would no longer be a normal migration step.

Pros:

- Avoids both migration gaps and downgrade incompatibility.
- Separates repair-style schema normalization from numbered migrations.

Cons:

- Larger conceptual and code-path change.
- Less explicit audit trail for this schema transition.
- Broader blast radius in migration framework behavior.

## Decision

Choose Option 1.

`ensure_cfs()` will always be a real migration step.
It will always run through `perform_step()` and therefore always write a
migration record when the step is active.
If all requested CFs already exist, the step becomes an idempotent recorded
no-op.

This keeps the migration ledger contiguous and makes future step numbering
safe.
Since downgrade compatibility is not required, the stricter recorded-step model
is the safer operational choice.

## Design Details

### `ensure_cfs` behavior

- Remove the early return that manually increments `self.step` without calling
  `perform_step()`.
- Move missing-CF detection inside the `perform_step()` closure.
- If the filtered list is empty, log that all requested CFs already exist and
  return `Ok(())`.
- If any CFs are missing, create them in the same closure.
- Let `perform_step()` write the migration record in both cases.

### Documentation

Update the `ensure_cfs` rustdoc to say:

- the step is idempotent,
- no-op executions are still recorded,
- recorded no-op steps preserve contiguous migration history for future
  releases.

Remove wording that describes advancing the step counter without persisting a
record.

### Tests

Keep the existing state-store idempotence coverage, but update its comments and
expectations so it no longer relies on an unrecorded no-op interpretation.

Add a focused migration test that proves this sequence works:

1. Open a database and run an `ensure_cfs()` step that becomes a no-op.
2. Run a later migration step.
3. Reopen the database.
4. Confirm reopen succeeds without `MigrationRecordGap`.

## Verification Target

The change is correct when:

- a no-op `ensure_cfs` run still leaves a contiguous migration record,
- a later migration step does not create a hole after that no-op,
- reopening the database succeeds under the updated migration sequence.
