# AGENTS.md

## Response priorities
- Start with high-level recommendations before implementation details.
- Keep recommendations short, opinionated, and tied to tradeoffs.
- If multiple paths exist, present one default path and one fallback.

## Dedicated domain behavior (Agglayer)
- Explicitly frame advice in Agglayer terms: cross-chain settlement, proofs, bridge safety, and operational reliability.
- Prefer changes that improve safety invariants, observability, and rollback clarity over local optimizations.
- Call out likely blast radius across crates, protocol boundaries, and e2e flows before proposing deep refactors.

## Collaboration norms
- Confirm assumptions in one sentence when requirements are ambiguous, then proceed with the safest minimal change.
- Surface risks early (consensus/security/regression/perf) and suggest one concrete verification step.

## Commit convention
- Use Conventional Commits exactly: `<type>(<optional-scope>): <description>`.
- Keep subject lines imperative and concise, for example: `fix(prover): reject malformed proof payload`.
- Keep commit subject lines at 72 characters max.
- Wrap commit body lines at 72 characters max, except long URLs, code blocks, or stack traces.

## PR creation and PR description
- When running `gh pr create` or when drafting a PR description, always use `.github/pull_request_template.md`.
- Follow merge-queue mapping: PR title equals commit title, PR description equals commit body.
- Keep PR titles at 72 characters max.
- Keep lines short and commit-friendly so the PR text can be reused directly as commit message content.
- Wrap PR description lines at 72 characters max, except long URLs, code blocks, or stack traces.
- Do not add `## Summary` or any other heading in the PR description.
- The first line of the PR description must be plain context text.
- Include context succinctly in the commit body/PR description.
- Fill template sections when applicable, including `CONFIG-CHANGE:` and `BREAKING-CHANGE:`.
- If there is no config change or breaking change, remove that section from the PR description.
