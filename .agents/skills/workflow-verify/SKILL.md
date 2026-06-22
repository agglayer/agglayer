---
name: verify
description: >
  **Mandatory** before any commit or push.
  Run Definition-of-Done checks from change scope
  and report exact pass/fail per command.
argument-hint: auto, minimal, code, proof, proto, full
---

Every change must be verified before declaring completion,
including documentation-only changes.

If `$ARGUMENTS` is provided (`auto`, `minimal`, `code`, `proof`, `proto`, `full`),
use it as a hint to prioritize relevant scopes.
If `$ARGUMENTS` is absent or ambiguous, use automatic scope detection.

## Blast-radius analysis (always first)

Before selecting test commands,
run the blast-radius detector script:

```bash
cargo make blast-radius
```

The script returns JSON with these fields:

- `changed_files`
- `affected_crates`
- `risk_flags`
- `docs_only`
- `recommended_scopes`
- `recommended_commands`

Use this output as the source of truth for scope selection
and command execution order.

If the script is unavailable or fails,
use this fallback procedure:

1. List changed files (staged and unstaged).
   If there are no local changes,
   compare against `main` if available.
2. Map changed paths to affected areas:
   - `proto/`
   - `crates/pessimistic-proof*`
   - `crates/<crate-name>/`
   - docs-only changes (`docs/`, `README.md`, markdown/adoc/rst/txt prose)
3. Determine `affected_crates`.
   Use `docs/knowledge-base/src/architecture.md` as the ownership map.
4. Detect `risk_flags`:
   - proof pipeline changes
   - protobuf schema changes
   - storage schema/migration changes
   - settlement/signer/contract changes
   - configuration schema changes
5. Derive `docs_only`:
   - `true` only when all changed files are documentation/prose and
     no runtime code/config/proto files changed
   - `false` otherwise
6. Derive `recommended_scopes`:
   - `minimal` always
   - `code` when runtime behavior may change
   - `proof` when proof crates changed
   - `proto` when protobuf schema changed
7. Derive `recommended_commands` as exact commands in execution order.

If `$ARGUMENTS` explicitly requests additional scopes,
append the missing scope commands.
`minimal` always remains required.

## Docs-only branch

If blast-radius reports `docs_only: true`:

- Always run `recommended_commands` from blast-radius.
- Ensure this command is included:

  ```bash
  mdbook build docs/knowledge-base/
  ```

- Skip runtime-heavy scopes (`code`, `proof`, `proto`)
  unless the user explicitly requested them via `$ARGUMENTS`.

## Scopes

Run **all** matching scopes (scopes are cumulative).
`minimal` always runs.
Additional scopes come from blast-radius output,
optionally constrained by `$ARGUMENTS`.
When available,
run `recommended_commands` directly in the provided order.

### Minimal (always runs)

```bash
cargo check --workspace --tests --all-features
```

`cargo check` only type-checks; it does **not** execute tests.
Never treat a passing `cargo check` as proof that changes work.
When any scope below matches the changed files, it must also run.

### Code behavior (features, bug fixes, refactors)

```bash
cargo make ci-all
cargo nextest run --workspace
```

`cargo make ci-all` runs: format check, clippy, typos,
and clippy on the PP program.

Test selection rules for `cargo nextest run` (fallback only,
when blast-radius did not provide `recommended_commands`):

- If blast-radius reports broad impact
  (core types/storage/rpc/proto boundaries or many crates),
  run:

  ```bash
  cargo nextest run --workspace
  ```

- Otherwise run package-targeted nextest for affected crates first.
- If package-targeted tests fail in a way that suggests transitive impact,
  escalate to `cargo nextest run --workspace`.

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
- Blast-radius result summary and chosen scope rationale.
