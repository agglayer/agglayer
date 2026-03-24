---
name: verify
description: >
  Run Definition-of-Done checks from change scope
  and report exact pass/fail per command.
argument-hint: auto, minimal, full, proof, proto
---

Every change must be verified before declaring completion,
including documentation-only changes.

If `$ARGUMENTS` is provided (`auto`, `minimal`, `full`, `proof`, `proto`),
use it as a hint to prioritize relevant scopes.
If `$ARGUMENTS` is absent or ambiguous, use automatic scope detection.

## Scopes

Detect which scopes apply based on the files changed,
then run **all** matching commands (scopes are cumulative).

### Minimal (always runs)

```bash
cargo check --workspace --tests --all-features
```

### Code behavior (features, bug fixes, refactors)

```bash
cargo make ci-all
cargo nextest run --workspace
```
`cargo make ci-all` runs: format check, clippy, typos,
and clippy on the PP program.

### Pessimistic proof (`crates/pessimistic-proof*`)

```bash
cargo make pp-check-vkey-change
```
If the vkey changed, **ask the user for explicit confirmation**
before running:
```bash
cargo make pp-accept-vkey-change
```

### Protobuf (`proto/`)

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
