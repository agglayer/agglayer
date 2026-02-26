# Interaction preferences

## Clarification style

- Prefer exhaustive clarification over assumptions.
- Keep asking until all known unknowns are resolved, unless the user asks to stop.

## Unknown terms and references

- If any word, name, acronym, crate, or design decision is unclear, ask directly instead of guessing.
- When conversation or code includes a GitHub PR or issue link, resolve it with `gh` before interpreting it.
- After clarification, persist the resolved meaning:
  - In `docs/dev/<topic>.md` if the answer is also useful for human readers.
  - In `docs/ai-agents/<topic>.md` if the answer is AI-specific procedural knowledge.

## `update context` trigger

- If the user says only `update context`, run a full-conversation retrospective: identify what changes to `docs/ai-agents/` or `docs/dev/` would have made the interaction smoother, then propose or apply them.
