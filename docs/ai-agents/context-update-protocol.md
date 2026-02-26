# Context update protocol

When the user says `update context`:

1. Read `AGENTS.md`.
2. Read the `docs/ai-agents/` or `docs/dev/` module(s) relevant to the requested change.
3. If the user provided explicit deltas, apply those changes and ask any missing high-value clarifications.
4. If the user says only `update context`, review the full conversation and identify what changes would have made the interaction smoother.
5. Propose or apply durable edits to the relevant file; ask clarifying questions only when needed to avoid encoding incorrect facts.
6. If unclear terms, crate names, design decisions, or domain concepts appear, ask before writing.
7. Edit concisely. Prefer targeted edits over full rewrites.
8. Do not remove sections without asking, unless the user explicitly requested structural refactoring.
9. If no durable change exists, the update is a no-op.

## File placement

- Use the context map table in `AGENTS.md` to identify which file owns the change.
- Domain or architecture knowledge useful to humans goes in `docs/dev/<topic>.md`.
- Contribution rules go in `CONTRIBUTING.md`; project overview goes in `README.md`.

## No-assumptions rule

Only write facts supported by the codebase, CI configuration, or explicit user statements. If uncertain, ask.

Checklist before saving a context update:
- Non-trivial claims are supported by code, config, or conversation evidence.
- New commands are verified against the relevant `Makefile.*.toml` or CI workflow.
- Unknown terms or design decisions were clarified with the user before writing.
- No inferred certainty wording without explicit support.
