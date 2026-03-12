# AGENTS.md

## Documentation and skills index

Key references for navigating this project:
- `README.md` -- project overview, crate table, build prerequisites.
- `CONTRIBUTING.md` -- contribution workflow, PR conventions.
- `docs/` -- human-readable documentation (validity checks, audits, dev guides).
- `Makefile.toml` -- build tasks (delegates to `scripts/make/*.toml`).

## Response priorities

- Start with high-level recommendations before implementation details.
- Keep recommendations short, opinionated, and tied to tradeoffs.
- If multiple paths exist, present one default path and one fallback.

## Clarification Before Action

- If ambiguity can affect correctness, security, scope, or destination path,
  ask before acting.
- When unknown terms or domain concepts appear, ask for an explanation
  and document them in the repository (in `docs/knowledge-base`) before proceeding.
- Low-risk ambiguity in instructions may be assumed:
  state one explicit assumption and proceed with the smallest reversible change.
- Ambiguity about technical meaning, domain semantics, or definitions
  is never low-risk. Always ask for clarification and document it if necessary.

## Evidence-Based Debugging and Communication

- Avoid overconfidence.
  Do not present uncertain conclusions as facts.
- State uncertainty explicitly when evidence is incomplete.
- Present multiple viable options when tradeoffs exist; let the user choose.
- Treat root-cause analysis as hypothesis-first until verified.
- Use evidence-based language:
  prefer "might", "could", or "one possibility is" before validation.
- Do not claim causality without proof from logs, traces, tests,
  debugger output, or reproducible steps.
- Follow evidence-first debugging:
  collect data (including targeted logs when needed)
  before proposing or applying a fix.

## Dedicated domain behavior (Agglayer)

- Explicitly frame advice in Agglayer terms: cross-chain settlement, proofs,
  bridge safety, and operational reliability.
- Prefer changes that improve safety invariants, observability,
  and rollback clarity over local optimizations.
- Call out likely blast radius across crates, protocol boundaries,
  and e2e flows before proposing deep refactors.

## Collaboration norms

- Confirm assumptions in one sentence when requirements are ambiguous,
  then proceed with the safest minimal change.
- Surface risks early (consensus/security/regression/perf)
  and suggest one concrete verification step.
- Leave edits unstaged by default so the user can review and adjust.
  Stage changes only when explicitly requested,
  or immediately before a user-requested commit.
- Precedence: when rules conflict,
  favor the Clarification Before Action section.
