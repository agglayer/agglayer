---
name: docs-knowledge-base
description: >
  Create or update docs/knowledge-base/ chapters
  in mdbook format for human-first technical documentation.
disable-model-invocation: true
---

Use this workflow when creating or editing files under `docs/knowledge-base/`.

## Audience and tone

- Primary audience: human maintainers and contributors.
- Secondary audience: AI agents reading project documentation.
- Write concise, factual prose.
- Avoid AI-specific annotations or prompt-oriented formatting.

## mdbook structure

- Keep chapter sources under `docs/knowledge-base/src/`.
- Keep `docs/knowledge-base/src/SUMMARY.md` as the navigation source of truth.
- When adding a new chapter,
  update `SUMMARY.md` in the same change.

## Chapter format

- One top-level heading (`#`) per chapter.
- Prefer short sections with explicit responsibilities,
  invariants,
  and workflows.
- Link related terms to `glossary.md`.
- Link to relevant code paths when making concrete claims.

## Writing conventions

- Follow semantic line breaks for prose.
- Prefer active voice and operational wording.
- Keep guidance implementation-neutral unless a path is repository-specific.
- Clearly separate facts,
  constraints,
  and recommendations.

## Verification

Before finishing,
build the book and report the exact command and result:

```bash
mdbook build docs/knowledge-base/
```
