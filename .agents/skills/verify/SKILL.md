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
python3 "$SKILL_DIR/scripts/blast_radius.py"
```

The script reads `.blast-radius.yaml` from the repo root
for project-specific configuration (core crates, risk areas, commands).
Without that file it still works using built-in Rust workspace defaults.

See `blast-radius.example.yaml` in this skill's directory
for the full config schema with comments.

The script returns JSON with these fields:

- `changed_files`
- `affected_modules`
- `risk_flags`
- `docs_only`
- `recommended_scopes`
- `recommended_commands`
- `broad_impact`

Use `recommended_commands` as the source of truth.
Run them in the order listed.

If the script is unavailable or fails,
use this fallback procedure:

1. List changed files (staged and unstaged).
   If there are no local changes,
   compare against `main` if available.
2. Map changed paths to affected areas:
   - `proto/`
   - `crates/<crate-name>/`
   - docs-only changes (`docs/`, `README.md`, markdown/adoc/rst/txt prose)
3. Run `cargo check --workspace --tests --all-features` (always).
4. If runtime code changed, run `cargo nextest run --workspace`.

## Docs-only branch

If blast-radius reports `docs_only: true`:

- Run `recommended_commands` from blast-radius output.
- Skip runtime-heavy scopes (`code`, `proof`, `proto`)
  unless the user explicitly requested them via `$ARGUMENTS`.

## Scopes

Run **all** matching scopes (scopes are cumulative).
`minimal` always runs.
Additional scopes come from blast-radius output,
optionally constrained by `$ARGUMENTS`.

The exact commands for each scope are determined by blast-radius
based on the project's `.blast-radius.yaml` configuration.
Always prefer `recommended_commands` from the JSON output
over manually constructing commands.

If `$ARGUMENTS` explicitly requests additional scopes
beyond what blast-radius recommended,
run the standard Rust verification for those scopes:

- **minimal**: `cargo check --workspace --tests --all-features`
- **code**: `cargo nextest run --workspace`
- **full**: all of the above plus any scope-specific commands from blast-radius

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
