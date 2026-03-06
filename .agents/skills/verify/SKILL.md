---
name: verify
description: >
  Run verification checks after completing a task.
  Use when declaring work done, after implementing a feature, fixing a bug,
  or making any code or documentation change.
  Picks the right check level based on what changed.
---

# Verification (Definition of Done)

Every change must be verified before declaring completion,
including documentation-only changes.

## Choose the right check level

1. **Default** (docs-only, config, minor code changes):
   ```bash
   cargo check --workspace --tests --all-features
   ```

2. **Code behavior changes** (features, bug fixes, refactors):
   ```bash
   cargo make ci-all
   ```
   This runs: format check, clippy, typos, and clippy on the PP program.

3. **Pessimistic proof crate changes** (`crates/pessimistic-proof*`):
   Also run:
   ```bash
   cargo make pp-check-vkey-change
   ```
   If the vkey changed intentionally:
   ```bash
   cargo make pp-accept-vkey-change
   ```

4. **Protobuf changes** (`proto/`):
   ```bash
   cargo make generate-proto
   ```
   Then verify no uncommitted diffs in generated code.

## Fix-and-rerun protocol

- If checks fail, attempt focused fixes for failures plausibly caused
  by your changes, then rerun checks.
- **Stop after 2 fix-and-rerun cycles**,
  or if failures appear unrelated to your changes.
- Hand control back with a brief summary of what passed, what failed,
  and what you tried.

## Reporting

Always report:
- Exact command(s) run.
- Whether each passed or failed.
- If failed: the relevant error output.
