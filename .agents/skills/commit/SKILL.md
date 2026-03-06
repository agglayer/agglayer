---
name: commit
description: Create a git commit following project conventions.
disable-model-invocation: true
---

# Commit workflow

Create commits using Conventional Commits format,
following the project's `commitlint.config.cjs` configuration.

## Format

```
<type>(<optional-scope>): <description>

[optional body]

[optional footer(s)]
```

## Rules

- **Subject line**: imperative mood, lowercase start, no trailing period.
- **Subject line max**: 72 characters.
- **Body lines max**: 72 characters,
  except long URLs, code blocks, or stack traces.
- **Allowed types**: `feat`, `fix`, `docs`, `chore`, `style`,
  `refactor`, `perf`, `test`, `build`, `ci`, `revert`.
- **Scope**: optional, should name the affected crate or area
  (e.g., `prover`, `grpc`, `config`).

## Examples

```
fix(prover): reject malformed proof payload
```

```
feat(grpc): add rate limiting to certificate submission

Introduces a token-bucket rate limiter per chain ID to prevent
excessive certificate submissions from overwhelming the pipeline.

CONFIG-CHANGE: New `rate_limit` section in agglayer.toml.
```

## Footer conventions

- `CONFIG-CHANGE:` for configuration changes (multi-line allowed).
- `BREAKING-CHANGE:` for breaking changes (multi-line allowed).

## Steps

1. Review staged changes with `git diff --cached`.
2. Draft a commit message following the rules above.
3. Present the message for approval before committing.
4. Create the commit.
   Do not push unless explicitly asked.
